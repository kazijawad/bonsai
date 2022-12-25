use std::sync::Arc;

use crate::{math::Float, transform::Transform};

pub enum CurveType {
    Flat,
    Cylinder,
    Ribbon,
}

pub struct CurveCommon {
    curve_type: CurveType,
}

pub struct Curve<'a> {
    object_to_world: &'a Transform,
    world_to_object: &'a Transform,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
    common: Arc<CurveCommon>,
    u_min: Float,
    u_max: Float,
}

impl<'a> Curve<'a> {
    pub fn new(
        object_to_world: &'a Transform,
        world_to_object: &'a Transform,
        reverse_orientation: bool,
        common: Arc<CurveCommon>,
        u_min: Float,
        u_max: Float,
    ) -> Self {
        let transform_swaps_handedness = object_to_world.swaps_handedness();

        Self {
            object_to_world,
            world_to_object,
            reverse_orientation,
            transform_swaps_handedness,
            common,
            u_min,
            u_max,
        }
    }
}
