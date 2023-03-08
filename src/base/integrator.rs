use crate::{base::scene::Scene, geometries::ray::RayDifferential, spectra::rgb::RGBSpectrum};

pub trait Integrator: Send + Sync {
    fn preprocess(&self, scene: &Scene);

    fn li(&self, ray: &RayDifferential, scene: &Scene, depth: u32) -> RGBSpectrum;

    fn render(&mut self, scene: &Scene);
}
