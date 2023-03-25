use crate::{
    base::{sampler::Sampler, scene::Scene},
    geometries::ray::Ray,
    spectra::rgb::RGBSpectrum,
};

pub trait Integrator: Send + Sync {
    fn preprocess(&self, scene: &Scene);

    fn radiance(
        &self,
        sampler: &mut Box<dyn Sampler>,
        ray: &mut Ray,
        scene: &Scene,
        depth: u32,
    ) -> RGBSpectrum;

    fn render(&mut self, scene: &Scene);
}
