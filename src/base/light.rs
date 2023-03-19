use dyn_clone::DynClone;

use crate::{
    base::{constants::Float, interaction::Interaction, primitive::Primitive, scene::Scene},
    geometries::{normal::Normal, point2::Point2, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
};

pub trait Light: Send + Sync + DynClone {
    fn preprocess(&mut self, _scene: &Scene) {}

    fn power(&self) -> RGBSpectrum;

    fn sample_point(
        &self,
        interaction: &dyn Interaction,
        sample: &Point2,
    ) -> (RGBSpectrum, Vec3, Float, VisibilityTester);

    fn pdf(&self, _interaction: &dyn Interaction, _incident_direction: &Vec3) -> Float {
        0.0
    }

    fn radiance(&self, _ray: &Ray) -> RGBSpectrum {
        RGBSpectrum::default()
    }

    fn sample_ray(
        &self,
        origin_sample: &Point2,
        direction_sample: &Point2,
        time: Float,
    ) -> (RGBSpectrum, Ray, Normal, Float, Float);

    fn pdf_ray(&self, ray: &Ray, surface_normal: &Normal) -> (Float, Float);

    fn is_infinite(&self) -> bool {
        false
    }
}

dyn_clone::clone_trait_object!(Light);

pub trait AreaLight: Light {
    fn emission(&self, interaction: &dyn Interaction, direction: &Vec3) -> RGBSpectrum;
}

pub struct VisibilityTester {
    pub p0: BaseInteraction,
    pub p1: BaseInteraction,
}

impl VisibilityTester {
    pub fn new(p0: BaseInteraction, p1: BaseInteraction) -> Self {
        Self { p0, p1 }
    }

    pub fn is_unoccluded(&self, scene: &Scene) -> bool {
        !scene.intersect_test(&self.p0.spawn_ray_to_it(&self.p1))
    }
}
