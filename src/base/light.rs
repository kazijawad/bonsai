use std::fmt::Debug;

use crate::{
    base::{
        interaction::Interaction,
        primitive::Primitive,
        scene::Scene,
        spectrum::{CoefficientSpectrum, Spectrum},
    },
    geometries::{normal::Normal, point2::Point2, ray::Ray, ray::RayDifferential, vec3::Vec3},
    utils::math::Float,
};

pub type LightFlags = u8;

pub const DELTA_POSITION: LightFlags = 1;
pub const DELTA_DIRECTION: LightFlags = 2;
pub const AREA: LightFlags = 4;
pub const INFINITE: LightFlags = 8;

pub fn is_delta_light(flags: LightFlags) -> bool {
    flags & DELTA_POSITION != 0 || flags & DELTA_DIRECTION != 0
}

pub trait Light: Debug + Send + Sync {
    fn preprocess(&self, scene: &Scene) {}

    fn power(&self) -> Spectrum;

    fn sample_li(
        &self,
        it: &dyn Interaction,
        wi: &mut Vec3,
        pdf: &mut Float,
        vis: &mut VisibilityTester,
    ) -> Spectrum;

    fn pdf_li(&self, it: &dyn Interaction, wi: &Vec3) -> Float;

    fn le(&self, ray: &RayDifferential) -> Spectrum {
        Spectrum::new(0.0)
    }

    fn sample_le(
        &self,
        u1: &Point2,
        u2: &Point2,
        time: Float,
        ray: &mut Ray,
        light_norm: Normal,
        pdf_pos: &mut Float,
        pdf_dir: &mut Float,
    ) -> Spectrum;

    fn pdf_le(&self, ray: &Ray, light_norm: Normal, pdf_pos: &mut Float, pdf_dir: &mut Float);
}

pub struct VisibilityTester {
    pub p0: Box<dyn Interaction>,
    pub p1: Box<dyn Interaction>,
}

impl VisibilityTester {
    pub fn new(p0: Box<dyn Interaction>, p1: Box<dyn Interaction>) -> Self {
        Self { p0, p1 }
    }

    pub fn is_unoccluded(&self, scene: &Scene) -> bool {
        !scene.intersect_test(&self.p0.spawn_ray_to_it(self.p1.as_ref()))
    }
}
