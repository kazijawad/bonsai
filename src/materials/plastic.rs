use std::sync::Arc;

use crate::{
    base::{
        bsdf::BSDF,
        constants::Float,
        fresnel::FresnelDielectric,
        material::{Material, TransportMode},
        microfacet::TrowbridgeReitzDistribution,
        spectrum::Spectrum,
        texture::Texture,
    },
    bxdfs::{lambertian::LambertianReflection, microfacet::MicrofacetReflection},
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
};

pub struct PlasticMaterial {
    pub kd: Box<dyn Texture<RGBSpectrum>>,
    pub ks: Box<dyn Texture<RGBSpectrum>>,
    pub roughness: Box<dyn Texture<Float>>,
    pub remap_roughness: bool,
}

impl Material for PlasticMaterial {
    fn compute_scattering_functions(
        &self,
        si: &mut SurfaceInteraction,
        _mode: TransportMode,
        _allow_multiple_lobes: bool,
    ) {
        let mut bsdf = BSDF::new(&si, 1.0);

        let kd = self.kd.evaluate(si).clamp(0.0, Float::INFINITY);
        if !kd.is_black() {
            bsdf.add(Box::new(LambertianReflection::new(kd)))
        }

        let ks = self.ks.evaluate(si).clamp(0.0, Float::INFINITY);
        if !ks.is_black() {
            let fresnel = Box::new(FresnelDielectric::new(1.5, 1.0));

            let mut roughness = self.roughness.evaluate(si);
            if self.remap_roughness {
                roughness = TrowbridgeReitzDistribution::roughness_to_alpha(roughness);
            }

            let distribution = Box::new(TrowbridgeReitzDistribution::new(roughness, roughness));
            let specular = Box::new(MicrofacetReflection::new(ks, distribution, fresnel));
            bsdf.add(specular);
        }

        si.bsdf = Some(Arc::new(bsdf));
    }
}
