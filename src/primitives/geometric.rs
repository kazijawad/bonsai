use std::sync::Arc;

use crate::{
    base::{primitive::Primitive, shape::Shape},
    geometries::{bounds3::Bounds3, ray::Ray},
    interaction::SurfaceInteraction,
    light::AreaLight,
    material::Material,
    medium::MediumInterface,
    utils::math::Float,
};

pub struct GeometricPrimitive<'a> {
    pub shape: Arc<dyn Shape<'a> + 'a>,
    pub material: Option<Arc<dyn Material + 'a>>,
    pub area_light: Option<Arc<AreaLight>>,
    pub medium_interface: &'a MediumInterface,
}

impl<'a> GeometricPrimitive<'a> {
    pub fn new(
        shape: Arc<dyn Shape<'a> + 'a>,
        material: Option<Arc<dyn Material + 'a>>,
        area_light: Option<Arc<AreaLight>>,
        medium_interface: &'a MediumInterface,
    ) -> Arc<Self> {
        Arc::new(Self {
            shape: shape.clone(),
            material: material.clone(),
            area_light: area_light.clone(),
            medium_interface,
        })
    }
}

impl<'a> Primitive<'a> for Arc<GeometricPrimitive<'a>> {
    fn world_bound(&self) -> Bounds3 {
        self.shape.world_bound()
    }

    fn intersect(&self, ray: &mut Ray, interaction: &mut SurfaceInteraction<'a>) -> bool {
        let mut t_hit: Float = 0.0;
        if !self.shape.intersect(ray, &mut t_hit, interaction, true) {
            return false;
        }
        ray.t_max = t_hit;
        interaction.primitive = Some(self.clone());
        // TODO: Initialize medium interface after shape intersection.
        true
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        self.shape.intersect_test(ray, true)
    }

    fn area_light(&self) -> Option<Arc<AreaLight>> {
        self.area_light.clone()
    }

    fn get_material(&self) -> Option<Arc<dyn Material + 'a>> {
        self.material.clone()
    }

    fn compute_scattering_function(&self, interaction: &mut SurfaceInteraction) {
        todo!()
    }
}
