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

    fn sample_li(
        &self,
        it: &dyn Interaction,
        u: &Point2,
        wi: &mut Vec3,
        pdf: &mut Float,
    ) -> (RGBSpectrum, VisibilityTester);

    fn pdf_li(&self, it: &dyn Interaction, wi: &Vec3) -> Float;

    fn le(&self, _ray: &Ray) -> RGBSpectrum {
        RGBSpectrum::default()
    }

    fn sample_le(
        &self,
        u1: &Point2,
        u2: &Point2,
        time: Float,
        ray: &mut Ray,
        light_norm: &mut Normal,
        pdf_pos: &mut Float,
        pdf_dir: &mut Float,
    ) -> RGBSpectrum;

    fn pdf_le(&self, ray: &Ray, light_norm: Normal, pdf_pos: &mut Float, pdf_dir: &mut Float);

    fn is_infinite(&self) -> bool {
        false
    }
}

dyn_clone::clone_trait_object!(Light);

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
