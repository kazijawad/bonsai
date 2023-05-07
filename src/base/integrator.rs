use crate::{
    base::{sampler::Sampler, scene::Scene},
    geometries::ray::Ray,
    spectra::rgb::RGBSpectrum,
};

pub trait Integrator<'a>: Send + Sync {
    fn radiance(
        &self,
        sampler: &mut Box<dyn Sampler>,
        ray: &mut Ray,
        scene: &Scene<'a>,
        depth: u32,
    ) -> RGBSpectrum;

    fn render(&mut self, scene: &Scene<'a>);
}
