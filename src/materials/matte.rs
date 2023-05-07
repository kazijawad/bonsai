use crate::{
    base::{
        bsdf::BSDF,
        constants::Float,
        material::{Material, TransportMode},
        spectrum::Spectrum,
        texture::Texture,
    },
    bxdfs::{lambertian::LambertianReflection, oren_nayer::OrenNayer},
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
};

pub struct MatteMaterial<'a> {
    pub kd: &'a (dyn Texture<RGBSpectrum> + 'a),
    pub sigma: &'a (dyn Texture<Float> + 'a),
}

impl<'a> Material for MatteMaterial<'a> {
    fn compute_scattering_functions(
        &self,
        si: &mut SurfaceInteraction,
        _mode: TransportMode,
        _allow_multiple_lobes: bool,
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
