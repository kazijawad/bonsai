use crate::{
    base::{
        material::{Material, TransportMode},
        primitive::Primitive,
        shape::Shape,
    },
    geometries::{bounds3::Bounds3, ray::Ray},
    interactions::surface::SurfaceInteraction,
    utils::math::Float,
};

pub struct GeometricPrimitive<'a> {
    pub shape: &'a dyn Shape,
    pub material: &'a dyn Material,
}

impl<'a> GeometricPrimitive<'a> {
    pub fn new(shape: &'a dyn Shape, material: &'a dyn Material) -> Self {
        Self { shape, material }
    }
}

impl<'a> Primitive for GeometricPrimitive<'a> {
    fn world_bound(&self) -> Bounds3 {
        self.shape.world_bound()
    }

    fn intersect(&self, ray: &mut Ray, interaction: &mut SurfaceInteraction) -> bool {
        let mut t_hit: Float = 0.0;
        if !self.shape.intersect(ray, &mut t_hit, interaction, true) {
            return false;
        }
        ray.t_max = t_hit;
        true
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        self.shape.intersect_test(ray, true)
    }

    fn compute_scattering_functions(
        &self,
        interaction: &mut SurfaceInteraction,
        transport_mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        todo!()
    }
}
