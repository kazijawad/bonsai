use std::{mem, sync::Arc};

use itertools::partition;

use crate::{
    base::{
        material::{Material, TransportMode},
        primitive::Primitive,
    },
    geometries::{bounds3::Bounds3, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::surface::SurfaceInteraction,
};

pub struct BVH {
    primitives: Vec<Arc<dyn Primitive>>,
    max_primitives_in_node: u32,
    nodes: Vec<LinearBVHNode>,
}

#[derive(Clone, Copy)]
struct LinearBVHNode {
    bounds: Bounds3,
    // Leaf Node:
    prims_offset: usize,
    // Interior Node:
    second_child_offset: usize,
    // Zero for interior nodes:
    num_prims: usize,
    axis: usize,
}

#[derive(Clone, Copy)]
struct BVHPrimitiveInfo {
    prim_index: usize,
    bounds: Bounds3,
    centroid: Point3,
}

struct BVHBuildNode {
    bounds: Bounds3,
    children: Vec<BVHBuildNode>,
    split_axis: usize,
    prims_offset: usize,
    num_prims: usize,
}

#[derive(Clone, Copy)]
struct BucketInfo {
    count: u32,
    bounds: Bounds3,
}

impl BVH {
    pub fn new(primitives: Vec<Arc<dyn Primitive>>, max_primitives_in_node: u32) -> Self {
        let mut bvh = Self {
            primitives,
            max_primitives_in_node: max_primitives_in_node.min(255),
            nodes: vec![],
        };
        if bvh.primitives.is_empty() {
            return bvh;
        }

        // Initialize primitive into array from primitives.
        let mut prim_info: Vec<BVHPrimitiveInfo> = Vec::with_capacity(bvh.primitives.len());
        for (i, primitive) in bvh.primitives.iter().enumerate() {
            prim_info.push(BVHPrimitiveInfo::new(i, primitive.world_bound()));
        }

        // Build BVH tree for primitives.
        let mut total_nodes = 0;
        let mut ordered_prims: Vec<Arc<dyn Primitive>> = Vec::with_capacity(bvh.primitives.len());
        let root = bvh.recursive_build(
            &mut prim_info,
            0,
            bvh.primitives.len(),
            &mut total_nodes,
            &mut ordered_prims,
        );

        mem::swap(&mut bvh.primitives, &mut ordered_prims);

        bvh.nodes = vec![LinearBVHNode::default(); total_nodes as usize];

        let offset = &mut 0;
        bvh.flatten(&root, offset);
        debug_assert_eq!(total_nodes, *offset as u32);

        bvh
    }

    fn recursive_build(
        &self,
        prim_info: &mut [BVHPrimitiveInfo],
        start: usize,
        end: usize,
        total_nodes: &mut u32,
        ordered_prims: &mut Vec<Arc<dyn Primitive>>,
    ) -> BVHBuildNode {
        let mut node = BVHBuildNode::default();
        *total_nodes += 1;

        // Compute bounds of all primitives in BVH node.
        let mut bounds = Bounds3::default();
        for i in start..end {
            bounds = bounds.union(&prim_info[i].bounds);
        }

        let num_prims = end - start;
        if num_prims == 1 {
            // Create leaf node.
            let first_prim_offset = ordered_prims.len();
            for i in start..end {
                let prim_index = prim_info[i].prim_index;
                ordered_prims.push(self.primitives[prim_index].clone());
            }
            node.init_leaf(first_prim_offset, num_prims, &bounds);
            return node;
        } else {
            // Compute bound of primitive centroids, choose split dimension.
            let mut centroid_bounds = Bounds3::default();
            for i in start..end {
                centroid_bounds = centroid_bounds.union_point(&prim_info[i].centroid);
            }
            let dim = centroid_bounds.maximum_extent();

            // Partition primitives into two sets and build children.
            let mid;
            if centroid_bounds.max[dim] == centroid_bounds.min[dim] {
                // Create leaf node.
                let first_prim_offset = ordered_prims.len();
                for i in start..end {
                    let prim_index = prim_info[i].prim_index;
                    ordered_prims.push(self.primitives[prim_index].clone());
                }
                node.init_leaf(first_prim_offset, num_prims, &bounds);
                return node;
            } else {
                // Partition primitives using approximate SAH.
                if num_prims <= 2 {
                    // Partition primitives into equally-sized subsets.
                    mid = (start + end) / 2;
                    prim_info.select_nth_unstable_by(mid, |a, b| {
                        a.centroid[dim].total_cmp(&b.centroid[dim])
                    });
                } else {
                    // Allocate bucket info for SAH partition buckets.
                    const NUM_BUCKETS: u32 = 12;
                    let mut buckets = [BucketInfo::default(); NUM_BUCKETS as usize];

                    // Initialize bucket info for SAH partition buckets.
                    for i in start..end {
                        let mut b = NUM_BUCKETS
                            * (centroid_bounds.offset(&prim_info[i].centroid)[dim] as u32);
                        if b == NUM_BUCKETS {
                            b = NUM_BUCKETS - 1;
                        }
                        buckets[b as usize].count += 1;
                        buckets[b as usize].bounds =
                            buckets[b as usize].bounds.union(&prim_info[i].bounds);
                    }

                    // Compute costs for splitting after each bucket
                    let mut cost = [0.0; (NUM_BUCKETS - 1) as usize];
                    for i in 0..(NUM_BUCKETS - 1) {
                        let mut b0 = Bounds3::default();
                        let mut b1 = Bounds3::default();
                        let mut count0 = 0;
                        let mut count1 = 0;

                        for j in 0..=i {
                            b0 = b0.union(&buckets[j as usize].bounds);
                            count0 += buckets[j as usize].count;
                        }
                        for j in (i + 1)..NUM_BUCKETS {
                            b1 = b1.union(&buckets[j as usize].bounds);
                            count1 += buckets[j as usize].count;
                        }

                        cost[i as usize] = 1.0
                            + (count0 as f32 * b0.surface_area()
                                + count1 as f32 * b1.surface_area())
                                / bounds.surface_area();
                    }

                    // Find bucket to split at that minimizes SAH metric.
                    let mut min_cost = cost[0];
                    let mut min_cost_split_bucket = 0;
                    for i in 1..(NUM_BUCKETS - 1) {
                        let bucket_cost = cost[i as usize];
                        if bucket_cost < min_cost {
                            min_cost = bucket_cost;
                            min_cost_split_bucket = i;
                        }
                    }

                    // Either create leaf or split primitives at selected SAH bucket.
                    let leaf_cost = num_prims as f32;
                    if num_prims as u32 > self.max_primitives_in_node || min_cost < leaf_cost {
                        let pmid = partition(prim_info.iter_mut(), |&pi| {
                            let mut b =
                                NUM_BUCKETS * (centroid_bounds.offset(&pi.centroid)[dim] as u32);
                            if b == NUM_BUCKETS {
                                b = NUM_BUCKETS - 1;
                            }
                            b <= min_cost_split_bucket
                        });
                        mid = pmid - prim_info[0].prim_index;
                    } else {
                        // Create leaf node.
                        let first_prim_offset = ordered_prims.len();
                        for i in start..end {
                            let prim_index = prim_info[i].prim_index;
                            ordered_prims.push(self.primitives[prim_index].clone());
                        }
                        node.init_leaf(first_prim_offset, num_prims, &bounds);
                        return node;
                    }
                }
                node.init_interior(
                    dim as usize,
                    self.recursive_build(prim_info, start, mid, total_nodes, ordered_prims),
                    self.recursive_build(prim_info, mid, end, total_nodes, ordered_prims),
                );
            }
        }

        node
    }

    fn flatten(&mut self, node: &BVHBuildNode, offset: &mut usize) -> usize {
        self.nodes[*offset].bounds = node.bounds;

        let current_offset = *offset;
        *offset += 1;

        if node.num_prims > 0 {
            debug_assert!(node.children.len() == 0);
            self.nodes[current_offset].prims_offset = node.prims_offset;
            self.nodes[current_offset].num_prims = node.num_prims;
        } else {
            // Create interior flattened BVH node
            self.nodes[current_offset].axis = node.split_axis;
            self.nodes[current_offset].num_prims = 0;
            self.flatten(&node.children[0], offset);
            self.nodes[current_offset].second_child_offset =
                self.flatten(&node.children[1], offset);
        }

        current_offset
    }
}

impl Primitive for BVH {
    fn world_bound(&self) -> Bounds3 {
        if !self.nodes.is_empty() {
            self.nodes[0].bounds
        } else {
            Bounds3::default()
        }
    }

    fn intersect(&self, ray: &mut Ray, interaction: &mut SurfaceInteraction) -> bool {
        if self.nodes.is_empty() {
            return false;
        }

        let mut hit = false;
        let inverse_dir = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        let is_neg_dir = [
            (inverse_dir.x < 0.0) as usize,
            (inverse_dir.y < 0.0) as usize,
            (inverse_dir.z < 0.0) as usize,
        ];

        // Follow ray through BVH nodes to find primitive intersections.
        let mut to_visited_offset = 0;
        let mut current_node_index = 0;
        let mut nodes_to_visit = [0; 64];
        loop {
            let node = self.nodes[current_node_index];
            if node
                .bounds
                .intersect_range_precomputed(ray, &inverse_dir, is_neg_dir)
            {
                if node.num_prims > 0 {
                    // Intersect ray with primitives in leaf BVH node.
                    for i in 0..node.num_prims {
                        if self.primitives[node.prims_offset + i].intersect(ray, interaction) {
                            hit = true;
                        }
                    }
                    if to_visited_offset == 0 {
                        break;
                    }
                    to_visited_offset -= 1;
                    current_node_index = nodes_to_visit[to_visited_offset];
                } else {
                    // Put far BVH node on stack and advance to near node.
                    if is_neg_dir[node.axis] != 0 {
                        nodes_to_visit[to_visited_offset] = current_node_index + 1;
                        current_node_index = node.second_child_offset;
                    } else {
                        nodes_to_visit[to_visited_offset] = node.second_child_offset;
                        current_node_index += 1;
                    }
                    to_visited_offset += 1;
                }
            } else {
                if to_visited_offset == 0 {
                    break;
                }
                to_visited_offset -= 1;
                current_node_index = nodes_to_visit[to_visited_offset];
            }
        }

        hit
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        if self.nodes.is_empty() {
            return false;
        }

        let inverse_dir = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        let is_neg_dir = [
            (inverse_dir.x < 0.0) as usize,
            (inverse_dir.y < 0.0) as usize,
            (inverse_dir.z < 0.0) as usize,
        ];

        // Follow ray through BVH nodes to find primitive intersections.
        let mut to_visited_offset = 0;
        let mut current_node_index = 0;
        let mut nodes_to_visit = [0; 64];
        loop {
            let node = self.nodes[current_node_index];
            if node
                .bounds
                .intersect_range_precomputed(ray, &inverse_dir, is_neg_dir)
            {
                if node.num_prims > 0 {
                    // Intersect ray with primitives in leaf BVH node.
                    for i in 0..node.num_prims {
                        if self.primitives[node.prims_offset + i].intersect_test(ray) {
                            return true;
                        }
                    }
                    if to_visited_offset == 0 {
                        break;
                    }
                    to_visited_offset -= 1;
                    current_node_index = nodes_to_visit[to_visited_offset];
                } else {
                    // Put far BVH node on stack and advance to near node.
                    if is_neg_dir[node.axis] != 0 {
                        nodes_to_visit[to_visited_offset] = current_node_index + 1;
                        current_node_index = node.second_child_offset;
                    } else {
                        nodes_to_visit[to_visited_offset] = node.second_child_offset;
                        current_node_index += 1;
                    }
                    to_visited_offset += 1;
                }
            } else {
                if to_visited_offset == 0 {
                    break;
                }
                to_visited_offset -= 1;
                current_node_index = nodes_to_visit[to_visited_offset];
            }
        }

        false
    }

    fn compute_scattering_functions(
        &self,
        interaction: &mut SurfaceInteraction,
        transport_mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        panic!("Aggregate::compute_scattering_function should not be called")
    }

    fn material(&self) -> Option<&dyn Material> {
        None
    }
}

impl BVHPrimitiveInfo {
    pub fn new(prim_index: usize, bounds: Bounds3) -> Self {
        Self {
            prim_index,
            bounds,
            centroid: 0.5 * bounds.min + 0.5 * bounds.max,
        }
    }
}

impl BVHBuildNode {
    pub fn init_leaf(&mut self, offset: usize, num_prims: usize, bounds: &Bounds3) {
        self.prims_offset = offset;
        self.num_prims = num_prims;
        self.bounds = bounds.clone();
    }

    pub fn init_interior(&mut self, split_axis: usize, c0: Self, c1: Self) {
        self.split_axis = split_axis;
        self.bounds = c0.bounds.union(&c1.bounds);
        self.children = vec![c0, c1];
    }
}

impl Default for LinearBVHNode {
    fn default() -> Self {
        Self {
            bounds: Bounds3::default(),
            prims_offset: 0,
            second_child_offset: 0,
            num_prims: 0,
            axis: 0,
        }
    }
}

impl Default for BVHBuildNode {
    fn default() -> Self {
        Self {
            bounds: Bounds3::default(),
            children: vec![],
            split_axis: 0,
            prims_offset: 0,
            num_prims: 0,
        }
    }
}

impl Default for BucketInfo {
    fn default() -> Self {
        Self {
            count: 0,
            bounds: Bounds3::default(),
        }
    }
}
