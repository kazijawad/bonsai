use std::fmt::Debug;

use crate::{
    base::{interaction::Interaction, primitive::Primitive, scene::Scene},
    geometries::{normal::Normal, point2::Point2, ray::Ray, ray::RayDifferential, vec3::Vec3},
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
    utils::math::Float,
};

#[derive(Debug, Clone, Copy)]
pub enum LightFlag {
    DeltaPosition,
    DeltaDirection,
    Area,
    Infinite,
}

pub trait Light: Debug + Send + Sync {
    fn preprocess(&self, scene: &Scene) {}

    fn power(&self) -> RGBSpectrum;

    fn sample_li(
        &self,
        it: &dyn Interaction,
        u: &Point2,
        wi: &mut Vec3,
        pdf: &mut Float,
    ) -> (RGBSpectrum, VisibilityTester);

    fn pdf_li(&self, it: &dyn Interaction, wi: &Vec3) -> Float;

    fn le(&self, ray: &RayDifferential) -> RGBSpectrum {
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

    fn flag(&self) -> LightFlag;

    fn is_delta_flag(&self) -> bool {
        let flag = self.flag();
        if let LightFlag::DeltaPosition = flag {
            true
        } else if let LightFlag::DeltaDirection = flag {
            true
        } else {
            false
        }
    }
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
