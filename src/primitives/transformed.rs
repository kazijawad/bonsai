use crate::{
    base::{
        material::TransportMode,
        primitive::Primitive,
        transform::{AnimatedTransform, Transform},
    },
    geometries::{bounds3::Bounds3, ray::Ray},
    interactions::surface::SurfaceInteraction,
};

pub struct TransformedPrimitive<'a> {
    primitive: &'a dyn Primitive,
    primitive_to_world: &'a AnimatedTransform,
}

impl<'a> TransformedPrimitive<'a> {
    pub fn new(primitive: &'a dyn Primitive, primitive_to_world: &'a AnimatedTransform) -> Self {
        Self {
            primitive,
            primitive_to_world,
        }
    }
}

impl<'a> Primitive for TransformedPrimitive<'a> {
    fn world_bound(&self) -> Bounds3 {
        self.primitive_to_world
            .motion_bounds(&self.primitive.world_bound())
    }

    fn intersect(&self, r: &mut Ray, si: &mut SurfaceInteraction) -> bool {
        let mut interpolated_primitive_to_world = Transform::default();

        // Compute ray after transformation applied by primitive_to_world.
        self.primitive_to_world
            .interpolate(r.time, &mut interpolated_primitive_to_world);
        let mut ray = interpolated_primitive_to_world.inverse().transform_ray(r);
        if !self.primitive.intersect(&mut ray, si) {
            return false;
        }
        r.t_max = ray.t_max;

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
            .intersect_test(&interpolated_world_to_primitive.transform_ray(r))
    }

    fn compute_scattering_functions(
        &self,
        interaction: &mut SurfaceInteraction,
        transport_mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        panic!("TransformedPrimitive::compute_scattering_function should not be called")
    }
}
