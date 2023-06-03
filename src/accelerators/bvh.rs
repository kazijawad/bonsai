use std::sync::Arc;

use itertools::partition;
use rayon::prelude::*;

use crate::{
    base::{
        constants::Float,
        light::AreaLight,
        material::{Material, TransportMode},
        primitive::Primitive,
    },
    geometries::{bounds3::Bounds3, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::surface::SurfaceInteraction,
};

const MAX_PRIMITIVES_IN_NODE: usize = 256;
const PARTITION_BUCKET_SIZE: usize = 16;

pub struct BVH {
    primitives: Vec<Arc<dyn Primitive>>,
    nodes: Vec<BVHNode>,
}

struct BVHNode {
    bounds: Bounds3,
    primitive_offset: usize,
    second_child_offset: usize,
    count: usize,
    axis: usize,
}

struct BVHPrimitiveInfo {
    index: usize,
    bounds: Bounds3,
    centroid: Point3,
}

struct BVHBuildNode {
    bounds: Bounds3,
    children: Box<[BVHBuildNode]>,
    split_axis: usize,
    offset: usize,
    count: usize,
}

struct BucketInfo {
    count: Float,
    bounds: Bounds3,
}

impl BVH {
    pub fn new(primitives: Vec<Arc<dyn Primitive>>) -> Self {
        if primitives.is_empty() {
            return Self {
                primitives,
                nodes: vec![],
            };
        }

        // Store relevant primitive calculations.
        let mut primitive_info: Vec<BVHPrimitiveInfo> = primitives
            .par_iter()
            .enumerate()
            .map(|(i, p)| BVHPrimitiveInfo::new(i, p.bounds()))
            .collect();

        // Build BVH tree for primitives.
        let mut total_nodes = 0;
        let mut ordered_primitives: Vec<Arc<dyn Primitive>> = Vec::with_capacity(primitives.len());
        let root = Self::build(
            &primitives,
            &mut primitive_info,
            &mut total_nodes,
            &mut ordered_primitives,
        );

        let mut nodes: Vec<BVHNode> = Vec::with_capacity(total_nodes);
        unsafe { nodes.set_len(total_nodes) }

        let offset = &mut 0;
        Self::flatten(&mut nodes, &root, offset);
        debug_assert_eq!(total_nodes, *offset);

        Self {
            primitives: ordered_primitives,
            nodes,
        }
    }

    fn build(
        primitives: &[Arc<dyn Primitive>],
        primitive_info: &mut [BVHPrimitiveInfo],
        count: &mut usize,
        ordered_primitives: &mut Vec<Arc<dyn Primitive>>,
    ) -> BVHBuildNode {
        debug_assert_ne!(primitive_info.len(), 0);

        let mut node = BVHBuildNode::default();
        *count += 1;

        // Compute bounds of all primitives in BVH node.
        let mut bounds = Bounds3::default();
        for p in primitive_info.iter() {
            bounds.union_mut(&p.bounds);
        }

        let size = primitive_info.len();
        if size == 1 {
            // Create leaf node.
            let node_offset = ordered_primitives.len();
            let index = primitive_info[0].index;

            ordered_primitives.push(primitives[index].clone());
            node.init_leaf(node_offset, size, &bounds);

            return node;
        } else {
            // Compute bound of primitive centroids, choose split dimension.
            let mut centroid_bounds = Bounds3::default();
            for p in primitive_info.iter() {
                centroid_bounds.union_point_mut(&p.centroid);
            }
            let dim = centroid_bounds.maximum_extent();

            // Partition primitives into two sets and build children.
            if centroid_bounds.max[dim] == centroid_bounds.min[dim] {
                // Create leaf node.
                let node_offset = ordered_primitives.len();

                for p in primitive_info.iter() {
                    ordered_primitives.push(primitives[p.index].clone());
                }

                node.init_leaf(node_offset, size, &bounds);

                return node;
            } else {
                let mut mid = primitive_info.len() / 2;

                // Partition primitives using approximate SAH.
                if size <= 2 {
                    // Partition primitives into equally-sized subsets.
                    primitive_info.select_nth_unstable_by(mid, |a, b| {
                        a.centroid[dim].total_cmp(&b.centroid[dim])
                    });
                } else {
                    // Allocate bucket info for SAH partition buckets.
                    let mut buckets: Vec<BucketInfo> = Vec::with_capacity(PARTITION_BUCKET_SIZE);
                    unsafe { buckets.set_len(PARTITION_BUCKET_SIZE) }

                    // Initialize bucket info for SAH partition buckets.
                    for p in primitive_info.iter() {
                        let mut b = PARTITION_BUCKET_SIZE
                            * centroid_bounds.offset(&p.centroid)[dim] as usize;
                        if b == PARTITION_BUCKET_SIZE {
                            b = PARTITION_BUCKET_SIZE - 1;
                        }

                        debug_assert!(b < PARTITION_BUCKET_SIZE);

                        buckets[b].count += 1.0;
                        buckets[b].bounds.union_mut(&p.bounds);
                    }

                    // Compute costs for splitting after each bucket.
                    let mut cost = vec![0.0; PARTITION_BUCKET_SIZE - 1].into_boxed_slice();
                    for i in 0..(PARTITION_BUCKET_SIZE - 1) {
                        let mut b0 = Bounds3::default();
                        let mut b1 = Bounds3::default();
                        let mut count0 = 0.0;
                        let mut count1 = 0.0;

                        for j in 0..=i {
                            b0.union_mut(&buckets[j].bounds);
                            count0 += buckets[j].count;
                        }
                        for j in (i + 1)..PARTITION_BUCKET_SIZE {
                            b1.union_mut(&buckets[j].bounds);
                            count1 += buckets[j].count;
                        }

                        cost[i] = 1.0
                            + (count0 * b0.surface_area() + count1 * b1.surface_area())
                                / bounds.surface_area();
                    }

                    // Find bucket to split at that minimizes SAH metric.
                    let mut min_cost = cost[0];
                    let mut min_cost_split_bucket = 0;

                    for i in 1..(PARTITION_BUCKET_SIZE - 1) {
                        if cost[i] < min_cost {
                            min_cost = cost[i];
                            min_cost_split_bucket = i;
                        }
                    }

                    // Either create leaf or split primitives at selected SAH bucket.
                    let leaf_cost = size as Float;
                    if size > MAX_PRIMITIVES_IN_NODE || min_cost < leaf_cost {
                        mid = partition(primitive_info.iter_mut(), |pi| {
                            let mut b = PARTITION_BUCKET_SIZE
                                * centroid_bounds.offset(&pi.centroid)[dim] as usize;

                            if b == PARTITION_BUCKET_SIZE {
                                b = PARTITION_BUCKET_SIZE - 1;
                            }

                            debug_assert!(b < PARTITION_BUCKET_SIZE);

                            b <= min_cost_split_bucket
                        });
                    } else {
                        // Create leaf node.
                        let prim_offset = ordered_primitives.len();

                        for p in primitive_info.iter() {
                            ordered_primitives.push(primitives[p.index].clone());
                        }

                        node.init_leaf(prim_offset, size, &bounds);

                        return node;
                    }
                }

                node.init_interior(
                    dim,
                    Self::build(
                        primitives,
                        &mut primitive_info[..mid],
                        count,
                        ordered_primitives,
                    ),
                    Self::build(
                        primitives,
                        &mut primitive_info[mid..],
                        count,
                        ordered_primitives,
                    ),
                );
            }
        }

        node
    }

    fn flatten(nodes: &mut [BVHNode], node: &BVHBuildNode, offset: &mut usize) -> usize {
        nodes[*offset].bounds = node.bounds;

        let current_offset = *offset;
        *offset += 1;

        if node.count > 0 {
            debug_assert!(node.children.len() == 0);
            nodes[current_offset].primitive_offset = node.offset;
            nodes[current_offset].count = node.count;
        } else {
            // Create interior flattened BVH node
            nodes[current_offset].axis = node.split_axis;
            nodes[current_offset].count = 0;
            Self::flatten(nodes, &node.children[0], offset);
            nodes[current_offset].second_child_offset =
                Self::flatten(nodes, &node.children[1], offset);
        }

        current_offset
    }
}

impl Primitive for BVH {
    fn bounds(&self) -> Bounds3 {
        if self.nodes.is_empty() {
            Bounds3::default()
        } else {
            self.nodes[0].bounds
        }
    }

    fn intersect(&self, ray: &mut Ray, si: &mut SurfaceInteraction) -> bool {
        if self.nodes.is_empty() {
            return false;
        }

        let mut hit = false;
        let inv_dir = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        let is_neg_dir = [
            (inv_dir.x < 0.0) as usize,
            (inv_dir.y < 0.0) as usize,
            (inv_dir.z < 0.0) as usize,
        ];

        // Follow ray through BVH nodes to find primitive intersections.
        let mut to_visit_offset = 0;
        let mut current_node_index = 0;
        let mut nodes_to_visit = vec![0; 64];

        loop {
            if to_visit_offset >= nodes_to_visit.len() {
                nodes_to_visit.append(&mut vec![0; 64]);
            }

            let node = &self.nodes[current_node_index];

            if node
                .bounds
                .intersect_range_precomputed(ray, &inv_dir, is_neg_dir)
            {
                if node.count > 0 {
                    // Intersect ray with primitives in leaf BVH node.
                    for i in 0..node.count {
                        let primitive = &self.primitives[node.primitive_offset + i];
                        if primitive.intersect(ray, si) {
                            hit = true;
                        }
                    }

                    if to_visit_offset == 0 {
                        break;
                    }

                    to_visit_offset -= 1;
                    current_node_index = nodes_to_visit[to_visit_offset];
                } else {
                    // Put far BVH node on stack and advance to near node.
                    if is_neg_dir[node.axis] != 0 {
                        nodes_to_visit[to_visit_offset] = current_node_index + 1;
                        current_node_index = node.second_child_offset;
                    } else {
                        nodes_to_visit[to_visit_offset] = node.second_child_offset;
                        current_node_index += 1;
                    }

                    to_visit_offset += 1;
                }
            } else {
                if to_visit_offset == 0 {
                    break;
                }

                to_visit_offset -= 1;
                current_node_index = nodes_to_visit[to_visit_offset];
            }
        }

        hit
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        if self.nodes.is_empty() {
            return false;
        }

        let inv_dir = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        let is_neg_dir = [
            (inv_dir.x < 0.0) as usize,
            (inv_dir.y < 0.0) as usize,
            (inv_dir.z < 0.0) as usize,
        ];

        // Follow ray through BVH nodes to find primitive intersections.
        let mut to_visit_offset = 0;
        let mut current_node_index = 0;
        let mut nodes_to_visit = vec![0; 64];

        loop {
            if to_visit_offset >= nodes_to_visit.len() {
                nodes_to_visit.append(&mut vec![0; 64]);
            }

            let node = &self.nodes[current_node_index];

            if node
                .bounds
                .intersect_range_precomputed(ray, &inv_dir, is_neg_dir)
            {
                if node.count > 0 {
                    // Intersect ray with primitives in leaf BVH node.
                    for i in 0..node.count {
                        if self.primitives[node.primitive_offset + i].intersect_test(ray) {
                            return true;
                        }
                    }

                    if to_visit_offset == 0 {
                        break;
                    }

                    to_visit_offset -= 1;
                    current_node_index = nodes_to_visit[to_visit_offset];
                } else {
                    // Put far BVH node on stack and advance to near node.
                    if is_neg_dir[node.axis] != 0 {
                        nodes_to_visit[to_visit_offset] = current_node_index + 1;
                        current_node_index = node.second_child_offset;
                    } else {
                        nodes_to_visit[to_visit_offset] = node.second_child_offset;
                        current_node_index += 1;
                    }

                    to_visit_offset += 1;
                }
            } else {
                if to_visit_offset == 0 {
                    break;
                }

                to_visit_offset -= 1;
                current_node_index = nodes_to_visit[to_visit_offset];
            }
        }

        false
    }

    fn compute_scattering_functions(
        &self,
        _si: &mut SurfaceInteraction,
        _transport_mode: TransportMode,
        _allow_multiple_lobes: bool,
    ) {
        unimplemented!();
    }

    fn material(&self) -> Option<&dyn Material> {
        None
    }

    fn area_light(&self) -> Option<&dyn AreaLight> {
        None
    }
}

impl BVHPrimitiveInfo {
    pub fn new(index: usize, bounds: Bounds3) -> Self {
        Self {
            index,
            bounds,
            centroid: 0.5 * bounds.min + 0.5 * bounds.max,
        }
    }
}

impl BVHBuildNode {
    pub fn init_leaf(&mut self, offset: usize, num_prims: usize, bounds: &Bounds3) {
        self.offset = offset;
        self.count = num_prims;
        self.bounds = bounds.clone();
    }

    pub fn init_interior(&mut self, split_axis: usize, c0: Self, c1: Self) {
        self.split_axis = split_axis;
        self.bounds = c0.bounds.union(&c1.bounds);
        self.children = vec![c0, c1].into_boxed_slice();
    }
}

impl Default for BVHNode {
    fn default() -> Self {
        Self {
            bounds: Bounds3::default(),
            primitive_offset: 0,
            second_child_offset: 0,
            count: 0,
            axis: 0,
        }
    }
}

impl Default for BVHBuildNode {
    fn default() -> Self {
        Self {
            bounds: Bounds3::default(),
            children: Vec::new().into_boxed_slice(),
            split_axis: 0,
            offset: 0,
            count: 0,
        }
    }
}

impl Default for BucketInfo {
    fn default() -> Self {
        Self {
            count: 0.0,
            bounds: Bounds3::default(),
        }
    }
}
