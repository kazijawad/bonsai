use std::{mem, sync::Arc};

use itertools::partition;

use crate::{
    base::{
        aggregate::Aggregate,
        material::{Material, TransportMode},
        primitive::Primitive,
    },
    geometries::{bounds3::Bounds3, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::surface::SurfaceInteraction,
    light::AreaLight,
};

pub struct BVH<'a> {
    primitives: Vec<Arc<dyn Primitive<'a>>>,
    max_primitives_in_node: u32,
    nodes: Vec<LinearBVHNode>,
}

#[derive(Debug, Clone, Copy)]
struct LinearBVHNode {
    bounds: Bounds3,
    // Leaf Node:
    primitives_offset: usize,
    // Interior Node:
    second_child_offset: usize,
    // Zero for interior nodes:
    num_primitives: usize,
    axis: usize,
}

#[derive(Debug, Clone, Copy)]
struct BVHPrimitiveInfo {
    primitive_index: usize,
    bounds: Bounds3,
    centroid: Point3,
}

#[derive(Debug)]
struct BVHBuildNode {
    bounds: Bounds3,
    children: Vec<BVHBuildNode>,
    split_axis: usize,
    primitives_offset: usize,
    num_primitives: usize,
}

#[derive(Debug, Clone, Copy)]
struct BucketInfo {
    count: u32,
    bounds: Bounds3,
}

impl<'a> BVH<'a> {
    pub fn new(primitives: Vec<Arc<dyn Primitive<'a>>>, max_primitives_in_node: u32) -> Box<Self> {
        let mut bvh = Self {
            primitives,
            max_primitives_in_node: max_primitives_in_node.min(255),
            nodes: vec![],
        };
        if bvh.primitives.is_empty() {
            return Box::new(bvh);
        }

        // Initialize primitive into array from primitives.
        let mut primitive_info: Vec<BVHPrimitiveInfo> = Vec::with_capacity(bvh.primitives.len());
        for (i, primitive) in bvh.primitives.iter().enumerate() {
            primitive_info.push(BVHPrimitiveInfo::new(i, primitive.world_bound()));
        }

        // Build BVH tree for primitives.
        let mut total_nodes = 0;
        let mut ordered_primitives: Vec<Arc<dyn Primitive<'a>>> =
            Vec::with_capacity(bvh.primitives.len());
        let root = bvh.recursive_build(
            &mut primitive_info,
            0,
            bvh.primitives.len(),
            &mut total_nodes,
            &mut ordered_primitives,
        );

        mem::swap(&mut bvh.primitives, &mut ordered_primitives);

        for i in 0..total_nodes {
            bvh.nodes.push(LinearBVHNode::default());
        }
        let offset = &mut 0;
        bvh.flatten(&root, offset);
        println!("{:?}", bvh.nodes);

        Box::new(bvh)
    }

    fn recursive_build(
        &self,
        primitive_info: &mut Vec<BVHPrimitiveInfo>,
        start: usize,
        end: usize,
        total_nodes: &mut u32,
        ordered_primitives: &mut Vec<Arc<dyn Primitive<'a>>>,
    ) -> BVHBuildNode {
        let mut node = BVHBuildNode::default();
        *total_nodes += 1;

        // Compute bounds of all primitives in BVH node.
        let mut bounds = Bounds3::default();
        for i in start..end {
            bounds = bounds.union(&primitive_info[i].bounds);
        }

        let num_primitives = end - start;
        if num_primitives == 1 {
            // Create leaf node.
            let first_primitive_offset = ordered_primitives.len();
            for i in start..end {
                let primitive_index = primitive_info[i].primitive_index;
                ordered_primitives.push(self.primitives[primitive_index].clone());
            }
            node.init_leaf(first_primitive_offset, num_primitives, &bounds);
            return node;
        } else {
            // Compute bound of primitive centroids, choose split dimension.
            let mut centroid_bounds = Bounds3::default();
            for i in start..end {
                centroid_bounds = centroid_bounds.union_point(&primitive_info[i].centroid);
            }
            let dim = centroid_bounds.maximum_extent();

            // Partition primitives into two sets and build children.
            let mid;
            if centroid_bounds.max[dim] == centroid_bounds.min[dim] {
                // Create leaf node.
                let first_primitive_offset = ordered_primitives.len();
                for i in start..end {
                    let primitive_index = primitive_info[i].primitive_index;
                    ordered_primitives.push(self.primitives[primitive_index].clone());
                }
                node.init_leaf(first_primitive_offset, num_primitives, &bounds);
                return node;
            } else {
                // Partition primitives using approximate SAH.
                if num_primitives <= 2 {
                    // Partition primitives into equally-sized subsets.
                    mid = (start + end) / 2;
                    primitive_info.select_nth_unstable_by(mid, |a, b| {
                        a.centroid[dim].total_cmp(&b.centroid[dim])
                    });
                } else {
                    // Allocate bucket info for SAH partition buckets.
                    const NUM_BUCKETS: u32 = 12;
                    let mut buckets = [BucketInfo::default(); NUM_BUCKETS as usize];

                    // Initialize bucket info for SAH partition buckets.
                    for i in start..end {
                        let mut b = NUM_BUCKETS
                            * (centroid_bounds.offset(&primitive_info[i].centroid)[dim] as u32);
                        if b == NUM_BUCKETS {
                            b = NUM_BUCKETS - 1;
                        }
                        buckets[b as usize].count += 1;
                        buckets[b as usize].bounds =
                            buckets[b as usize].bounds.union(&primitive_info[i].bounds);
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
                    let leaf_cost = num_primitives as f32;
                    if num_primitives as u32 > self.max_primitives_in_node || min_cost < leaf_cost {
                        let pmid = partition(primitive_info.iter_mut(), |&pi| {
                            let mut b =
                                NUM_BUCKETS * (centroid_bounds.offset(&pi.centroid)[dim] as u32);
                            if b == NUM_BUCKETS {
                                b = NUM_BUCKETS - 1;
                            }
                            b <= min_cost_split_bucket
                        });
                        mid = pmid - primitive_info[0].primitive_index;
                    } else {
                        // Create leaf node.
                        let first_primitive_offset = ordered_primitives.len();
                        for i in start..end {
                            let primitive_index = primitive_info[i].primitive_index;
                            ordered_primitives.push(self.primitives[primitive_index].clone());
                        }
                        node.init_leaf(first_primitive_offset, num_primitives, &bounds);
                        return node;
                    }
                }
                node.init_interior(
                    dim as usize,
                    self.recursive_build(
                        primitive_info,
                        start,
                        mid,
                        total_nodes,
                        ordered_primitives,
                    ),
                    self.recursive_build(primitive_info, mid, end, total_nodes, ordered_primitives),
                );
            }
        }

        node
    }

    fn flatten(&mut self, node: &BVHBuildNode, offset: &mut usize) -> usize {
        self.nodes[*offset].bounds = node.bounds;

        let current_offset = *offset;
        *offset += 1;

        if node.num_primitives > 0 {
            debug_assert!(node.children.len() == 0);
            self.nodes[current_offset].primitives_offset = node.primitives_offset;
            self.nodes[current_offset].num_primitives = node.num_primitives;
        } else {
            // Create interior flattened BVH node
            self.nodes[current_offset].axis = node.split_axis;
            self.nodes[current_offset].num_primitives = 0;
            self.flatten(&node.children[0], offset);
            self.nodes[current_offset].second_child_offset =
                self.flatten(&node.children[1], offset);
        }

        current_offset
    }
}

impl<'a> Primitive<'a> for BVH<'a> {
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
        let inverted_direction = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        let is_negative_direction = [
            (inverted_direction.x < 0.0) as u32,
            (inverted_direction.y < 0.0) as u32,
            (inverted_direction.z < 0.0) as u32,
        ];

        // Follow ray through BVH nodes to find primitive intersections.
        let mut to_visited_offset = 0;
        let mut current_node_index = 0;
        let mut nodes_to_visit = [0; 64];
        loop {
            let node = self.nodes[current_node_index];
            if node.bounds.intersect_range_precomputed(
                ray,
                &inverted_direction,
                is_negative_direction,
            ) {
                if node.num_primitives > 0 {
                    // Intersect ray with primitives in leaf BVH node.
                    for i in 0..node.num_primitives {
                        if self.primitives[node.primitives_offset + i].intersect(ray, interaction) {
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
                    if is_negative_direction[node.axis] != 0 {
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

        let mut hit = false;
        let inverted_direction = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        let is_negative_direction = [
            (inverted_direction.x < 0.0) as u32,
            (inverted_direction.y < 0.0) as u32,
            (inverted_direction.z < 0.0) as u32,
        ];

        // Follow ray through BVH nodes to find primitive intersections.
        let mut to_visited_offset = 0;
        let mut current_node_index = 0;
        let mut nodes_to_visit = [0; 64];
        loop {
            let node = self.nodes[current_node_index];
            if node.bounds.intersect_range_precomputed(
                ray,
                &inverted_direction,
                is_negative_direction,
            ) {
                if node.num_primitives > 0 {
                    // Intersect ray with primitives in leaf BVH node.
                    for i in 0..node.num_primitives {
                        if self.primitives[node.primitives_offset + i].intersect_test(ray) {
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
                    if is_negative_direction[node.axis] != 0 {
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

    fn get_area_light(&self) -> Option<Arc<AreaLight>> {
        panic!("Aggregate::get_area_light should not be called")
    }

    fn get_material(&self) -> Option<Arc<dyn Material>> {
        panic!("Aggregate::get_material should not be called")
    }

    fn compute_scattering_functions(
        &self,
        interaction: &mut SurfaceInteraction,
        transport_mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        panic!("Aggregate::compute_scattering_function should not be called")
    }
}

impl<'a> Aggregate<'a> for BVH<'a> {}

impl BVHPrimitiveInfo {
    pub fn new(primitive_index: usize, bounds: Bounds3) -> Self {
        Self {
            primitive_index,
            bounds,
            centroid: 0.5 * bounds.min + 0.5 * bounds.max,
        }
    }
}

impl BVHBuildNode {
    pub fn init_leaf(&mut self, offset: usize, num_primitives: usize, bounds: &Bounds3) {
        self.primitives_offset = offset;
        self.num_primitives = num_primitives;
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
            primitives_offset: 0,
            second_child_offset: 0,
            num_primitives: 0,
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
            primitives_offset: 0,
            num_primitives: 0,
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
