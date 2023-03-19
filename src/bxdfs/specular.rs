use crate::{
    base::{
        bxdf::{BxDF, BxDFType, BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION},
        constants::Float,
        fresnel::{Fresnel, FresnelDielectric},
        material::TransportMode,
    },
    geometries::{normal::Normal, point2::Point2, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
    utils::bxdf::{abs_cos_theta, cos_theta, refract},
};

#[derive(Clone)]
pub struct SpecularReflection {
    bxdf_type: BxDFType,
    r: RGBSpectrum,
    fresnel: Box<dyn Fresnel>,
}

#[derive(Clone)]
pub struct SpecularTransmission {
    bxdf_type: BxDFType,
    t: RGBSpectrum,
    fresnel: FresnelDielectric,
    eta_a: Float,
    eta_b: Float,
    mode: TransportMode,
}

impl SpecularReflection {
    pub fn new(r: RGBSpectrum, fresnel: Box<dyn Fresnel>) -> Self {
        Self {
            bxdf_type: BSDF_REFLECTION | BSDF_SPECULAR,
            r,
            fresnel,
        }
    }
}

impl SpecularTransmission {
    pub fn new(t: RGBSpectrum, eta_a: Float, eta_b: Float, mode: TransportMode) -> Self {
        Self {
            bxdf_type: BSDF_TRANSMISSION | BSDF_SPECULAR,
            t,
            fresnel: FresnelDielectric::new(eta_a, eta_b),
            eta_a,
            eta_b,
            mode,
        }
    }
}

impl BxDF for SpecularReflection {
    fn f(&self, _wo: &Vec3, _wi: &Vec3) -> RGBSpectrum {
        RGBSpectrum::default()
    }

    fn sample_f(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        _sample: &Point2,
        pdf: &mut Float,
        _sampled_type: &mut Option<BxDFType>,
    ) -> RGBSpectrum {
        *wi = Vec3::new(-wo.x, -wo.y, wo.z);
        *pdf = 1.0;
        self.fresnel.evaluate(cos_theta(wi)) * self.r / abs_cos_theta(wi)
    }

    fn pdf(&self, _wo: &Vec3, _wi: &Vec3) -> Float {
        0.0
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }

    fn bxdf_type(&self) -> BxDFType {
        self.bxdf_type
    }
}

impl BxDF for SpecularTransmission {
    fn f(&self, _wo: &Vec3, _wi: &Vec3) -> RGBSpectrum {
        RGBSpectrum::default()
    }

    fn sample_f(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        _sample: &Point2,
        pdf: &mut Float,
        _sampled_type: &mut Option<BxDFType>,
    ) -> RGBSpectrum {
        // Determine which eta is incident or transmitted.
        let entering = cos_theta(wo) > 0.0;
        let eta_i = if entering { self.eta_a } else { self.eta_b };
        let eta_t = if entering { self.eta_b } else { self.eta_a };

        // Compute ray direction for specular transmission.
        if !refract(
            wo,
            &Normal::new(0.0, 0.0, 1.0).face_forward(&Normal::from(*wo)),
            eta_i / eta_t,
            wi,
        ) {
            return RGBSpectrum::default();
        }

        *pdf = 1.0;
        let mut factor = self.t * (RGBSpectrum::new(1.0) - self.fresnel.evaluate(cos_theta(&wi)));

        // Account for non-symmetry with transmission to different medium.
        if let TransportMode::Radiance = self.mode {
            factor *= (eta_i * eta_i) / (eta_t * eta_t);
        }

        factor / abs_cos_theta(&wi)
    }

    fn pdf(&self, _wo: &Vec3, _wi: &Vec3) -> Float {
        0.0
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }

    fn bxdf_type(&self) -> BxDFType {
        self.bxdf_type
    }
}
