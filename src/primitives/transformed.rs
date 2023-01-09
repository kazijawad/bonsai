use std::sync::Arc;

use crate::{
    base::{
        material::{Material, TransportMode},
        primitive::Primitive,
        transform::{AnimatedTransform, Transform},
    },
    geometries::{bounds3::Bounds3, ray::Ray},
    interactions::surface::SurfaceInteraction,
    light::AreaLight,
};

pub struct TransformedPrimitive<'a> {
    primitive: Arc<dyn Primitive<'a> + 'a>,
    primitive_to_world: &'a AnimatedTransform,
}

impl<'a> TransformedPrimitive<'a> {
    pub fn new(
        primitive: Arc<dyn Primitive<'a> + 'a>,
        primitive_to_world: &'a AnimatedTransform,
    ) -> Arc<Self> {
        Arc::new(Self {
            primitive: primitive.clone(),
            primitive_to_world,
        })
    }
}

impl<'a> Primitive<'a> for TransformedPrimitive<'a> {
    fn world_bound(&self) -> Bounds3 {
        self.primitive_to_world
            .motion_bounds(&self.primitive.world_bound())
    }

    fn intersect(&self, r: &mut Ray, interaction: &mut SurfaceInteraction) -> bool {
        let mut interpolated_primitive_to_world = Transform::default();

        // Compute ray after transformation applied by primitive_to_world.
        self.primitive_to_world
            .interpolate(r.time, &mut interpolated_primitive_to_world);
        let mut ray = interpolated_primitive_to_world.inverse().transform_ray(r);
        if !self.primitive.intersect(&mut ray, interaction) {
            return false;
        }
        r.t_max = ray.t_max;

        // Transform instance's intersection data to world space.
        if !interpolated_primitive_to_world.is_identity() {
            *interaction =
                interpolated_primitive_to_world.transform_surface_interaction(&interaction.clone());
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

    fn get_area_light(&self) -> Option<Arc<AreaLight>> {
        None
    }

    fn get_material(&self) -> Option<Arc<dyn Material + 'a>> {
        None
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
