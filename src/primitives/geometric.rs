use std::sync::Arc;

use crate::{
    base::{
        constants::Float,
        interaction::Interaction,
        light::AreaLight,
        material::{Material, TransportMode},
        primitive::Primitive,
        shape::Shape,
    },
    geometries::{bounds3::Bounds3, ray::Ray},
};

#[derive(Clone)]
pub struct GeometricPrimitive {
    pub shape: Arc<dyn Shape>,
    pub material: Arc<dyn Material>,
    pub area_light: Option<Arc<dyn AreaLight>>,
}

impl Primitive for GeometricPrimitive {
    fn bounds(&self) -> Bounds3 {
        self.shape.world_bounds()
    }

    fn intersect(&self, ray: &mut Ray, si: &mut Interaction) -> bool {
        let mut t_hit: Float = 0.0;
        if !self.shape.intersect(ray, &mut t_hit, si) {
            return false;
        }
        ray.t_max = t_hit;

        let so = si.surface.as_mut().unwrap();
        so.primitive = Some(Arc::new(self.clone()));

        true
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        self.shape.intersect_test(ray)
    }

    fn compute_scattering_functions(
        &self,
        si: &mut Interaction,
        mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        self.material
            .compute_scattering_functions(si, mode, allow_multiple_lobes);
    }

    fn material(&self) -> Option<&dyn Material> {
        Some(self.material.as_ref())
    }

    fn area_light(&self) -> Option<&dyn AreaLight> {
        self.area_light.as_deref()
    }
}
