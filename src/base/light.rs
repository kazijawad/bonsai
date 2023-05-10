use crate::{
    base::{constants::Float, interaction::Interaction, scene::Scene},
    geometries::{normal::Normal, point2::Point2F, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
};

pub trait Light: Send + Sync {
    fn power(&self) -> RGBSpectrum;

    fn radiance(&self, _: &Ray) -> RGBSpectrum {
        RGBSpectrum::default()
    }

    fn sample_point(&self, it: &dyn Interaction, u: &Point2F) -> LightPointSample;

    fn point_pdf(&self, _: &dyn Interaction, _dir: &Vec3) -> Float {
        0.0
    }

    fn sample_ray(
        &self,
        u1: &Point2F,
        u2: &Point2F,
        time: Float,
    ) -> (RGBSpectrum, Ray, Normal, Float, Float);

    fn ray_pdf(&self, ray: &Ray, n: &Normal) -> (Float, Float);

    fn is_infinite(&self) -> bool {
        false
    }
}

pub trait AreaLight: Light {
    fn emission(&self, it: &dyn Interaction, dir: &Vec3) -> RGBSpectrum;
}

pub struct LightPointSample {
    pub radiance: RGBSpectrum,
    pub wi: Vec3,
    pub pdf: Float,
    pub visibility: Option<VisibilityTester>,
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
