use std::sync::Arc;

use crate::{
    base::{
        light::AreaLight,
        material::{Material, TransportMode},
        primitive::Primitive,
        transform::{AnimatedTransform, Transform},
    },
    geometries::{bounds3::Bounds3, ray::Ray},
    interactions::surface::SurfaceInteraction,
};

pub struct TransformedPrimitive {
    primitive: Arc<dyn Primitive>,
    primitive_to_world: Arc<AnimatedTransform>,
}

impl TransformedPrimitive {
    pub fn new(primitive: Arc<dyn Primitive>, primitive_to_world: Arc<AnimatedTransform>) -> Self {
        Self {
            primitive,
            primitive_to_world,
        }
    }
}

impl Primitive for TransformedPrimitive {
    fn world_bound(&self) -> Bounds3 {
        self.primitive_to_world
            .motion_bounds(&self.primitive.world_bound())
    }

    fn intersect(&self, ray: &mut Ray, si: &mut SurfaceInteraction) -> bool {
        let mut interpolated_primitive_to_world = Transform::default();

        // Compute ray after transformation applied by primitive_to_world.
        self.primitive_to_world
            .interpolate(ray.time, &mut interpolated_primitive_to_world);
        let mut ray = ray.transform(&interpolated_primitive_to_world.inverse());
        if !self.primitive.intersect(&mut ray, si) {
            return false;
        }
        ray.t_max = ray.t_max;

        // Transform instance's intersection data to world space.
        if !interpolated_primitive_to_world.is_identity() {
            *si = si.transform(&interpolated_primitive_to_world);
        }

        true
    }

    fn intersect_test(&self, r: &Ray) -> bool {
        let mut interpolated_primitive_to_world = Transform::default();
        self.primitive_to_world
            .interpolate(r.time, &mut interpolated_primitive_to_world);

        let interpolated_world_to_primitive = interpolated_primitive_to_world.inverse();
        self.primitive
            .intersect_test(&r.transform(&interpolated_world_to_primitive))
    }

    fn compute_scattering_functions(
        &self,
        _interaction: &mut SurfaceInteraction,
        _transport_mode: TransportMode,
        _allow_multiple_lobes: bool,
    ) {
        panic!("TransformedPrimitive::compute_scattering_function should not be called")
    }

    fn material(&self) -> Option<&dyn Material> {
        None
    }

    fn area_light(&self) -> Option<&dyn AreaLight> {
        None
    }
}
