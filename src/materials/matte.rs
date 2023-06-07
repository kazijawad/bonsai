use crate::{
    base::{
        bsdf::BSDF,
        constants::Float,
        interaction::Interaction,
        material::{Material, TransportMode},
        spectrum::Spectrum,
        texture::Texture,
    },
    bxdfs::{lambertian::LambertianReflection, oren_nayer::OrenNayer},
    spectra::rgb::RGBSpectrum,
};

pub struct MatteMaterial {
    pub kd: Box<dyn Texture<RGBSpectrum>>,
    pub sigma: Box<dyn Texture<Float>>,
}

impl Material for MatteMaterial {
    fn compute_scattering_functions(
        &self,
        it: &mut Interaction,
        _mode: TransportMode,
        _allow_multiple_lobes: bool,
    ) {
        let mut bsdf = BSDF::new(&it, 1.0);

        let r = self.kd.evaluate(it).clamp(0.0, Float::INFINITY);
        let sigma = self.sigma.evaluate(it).clamp(0.0, 90.0);

        if !r.is_black() {
            if sigma == 0.0 {
                bsdf.add(Box::new(LambertianReflection::new(r)));
            } else {
                bsdf.add(Box::new(OrenNayer::new(r, sigma)));
            }
        }

        let si = it.surface.as_mut().unwrap();
        si.bsdf = Some(bsdf);
    }
}
