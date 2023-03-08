use std::sync::Arc;

use crate::{
    base::{
        bsdf::BSDF,
        material::{Material, TransportMode},
        spectrum::Spectrum,
        texture::Texture,
    },
    bxdfs::{lambertian::LambertianReflection, oren_nayer::OrenNayer},
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
    utils::math::Float,
};

pub struct MatteMaterial {
    kd: Arc<dyn Texture<RGBSpectrum>>,
    sigma: Arc<dyn Texture<Float>>,
}

impl MatteMaterial {
    pub fn new(kd: Arc<dyn Texture<RGBSpectrum>>, sigma: Arc<dyn Texture<Float>>) -> Self {
        Self { kd, sigma }
    }
}

impl Material for MatteMaterial {
    fn compute_scattering_functions(
        &self,
        si: &mut SurfaceInteraction,
        mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        let mut bsdf = BSDF::new(&si, 1.0);
        let r = self.kd.evaluate(si).clamp(0.0, Float::INFINITY);
        let sigma = self.sigma.evaluate(si).clamp(0.0, 90.0);
        if !r.is_black() {
            if sigma == 0.0 {
                bsdf.add(Box::new(LambertianReflection::new(r)));
            } else {
                bsdf.add(Box::new(OrenNayer::new(r, sigma)));
            }
        }

        si.bsdf = Some(bsdf);
    }
}
