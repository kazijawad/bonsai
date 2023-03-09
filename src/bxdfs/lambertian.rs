use crate::{
    base::bxdf::{BxDF, BxDFType, BSDF_DIFFUSE, BSDF_REFLECTION, BSDF_TRANSMISSION},
    geometries::{point2::Point2, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
    utils::{
        bxdf::{abs_cos_theta, same_hemisphere},
        math::{Float, PI},
        sampling::cosine_sample_hemisphere,
    },
};

#[derive(Debug, Clone)]
pub struct LambertianReflection {
    bxdf_type: BxDFType,
    r: RGBSpectrum,
}

#[derive(Debug, Clone)]
pub struct LambertianTransmission {
    bxdf_type: BxDFType,
    t: RGBSpectrum,
}

impl LambertianReflection {
    pub fn new(r: RGBSpectrum) -> Self {
        Self {
            bxdf_type: BSDF_REFLECTION | BSDF_DIFFUSE,
            r,
        }
    }
}

impl LambertianTransmission {
    pub fn new(t: RGBSpectrum) -> Self {
        Self {
            bxdf_type: BSDF_TRANSMISSION | BSDF_DIFFUSE,
            t,
        }
    }
}

impl BxDF for LambertianReflection {
    fn f(&self, wo: &Vec3, wi: &Vec3) -> RGBSpectrum {
        self.r * (1.0 / PI)
    }

    fn rho_hd(&self, wo: &Vec3, num_samples: usize, samples: &[Point2]) -> RGBSpectrum {
        self.r
    }

    fn rho_hh(&self, num_samples: usize, u1: &[Point2], u2: &[Point2]) -> RGBSpectrum {
        self.r
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }
}

impl BxDF for LambertianTransmission {
    fn f(&self, wo: &Vec3, wi: &Vec3) -> RGBSpectrum {
        self.t * (1.0 / PI)
    }

    fn sample_f(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        sample: &Point2,
        pdf: &mut Float,
        sampled_type: &mut Option<BxDFType>,
    ) -> RGBSpectrum {
        *wi = cosine_sample_hemisphere(sample);
        if wo.z > 0.0 {
            wi.z *= -1.0;
        }
        *pdf = self.pdf(wo, wi);
        self.f(wo, wi)
    }

    fn rho_hd(&self, wo: &Vec3, num_samples: usize, samples: &[Point2]) -> RGBSpectrum {
        self.t
    }

    fn rho_hh(&self, num_samples: usize, u1: &[Point2], u2: &[Point2]) -> RGBSpectrum {
        self.t
    }

    fn pdf(&self, wo: &Vec3, wi: &Vec3) -> Float {
        if !same_hemisphere(wo, wi) {
            abs_cos_theta(wi) * (1.0 / PI)
        } else {
            0.0
        }
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }
}