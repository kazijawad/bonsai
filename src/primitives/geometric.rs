use std::sync::Arc;

use crate::{
    base::{
        constants::Float,
        material::{Material, TransportMode},
        primitive::Primitive,
        shape::Shape,
    },
    geometries::{bounds3::Bounds3, ray::Ray},
    interactions::surface::SurfaceInteraction,
};

#[derive(Clone)]
pub struct GeometricPrimitive {
    pub shape: Arc<dyn Shape>,
    pub material: Arc<dyn Material>,
}

impl GeometricPrimitive {
    pub fn new(shape: Arc<dyn Shape>, material: Arc<dyn Material>) -> Self {
        Self { shape, material }
    }
}

impl Primitive for GeometricPrimitive {
    fn world_bound(&self) -> Bounds3 {
        self.shape.world_bound()
    }

    fn intersect(&self, ray: &mut Ray, si: &mut SurfaceInteraction) -> bool {
        let mut t_hit: Float = 0.0;
        if !self.shape.intersect(ray, &mut t_hit, si, true) {
            return false;
        }
        ray.t_max = t_hit;
        si.primitive = Some(Arc::new(self.clone()));
        true
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        self.shape.intersect_test(ray, true)
    }

    fn compute_scattering_functions(
        &self,
        si: &mut SurfaceInteraction,
        mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        self.material
            .compute_scattering_functions(si, mode, allow_multiple_lobes);
    }

    fn material(&self) -> Option<&dyn Material> {
        Some(self.material.as_ref())
    }
}
