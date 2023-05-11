use crate::{
    base::{
        bxdf::{
            abs_cos_theta, cos_theta, refract, BxDF, BxDFSample, BxDFType, BSDF_REFLECTION,
            BSDF_SPECULAR, BSDF_TRANSMISSION,
        },
        constants::Float,
        fresnel::{Fresnel, FresnelDielectric},
        material::TransportMode,
    },
    geometries::{normal::Normal, point2::Point2F, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
};

pub struct SpecularReflection {
    bxdf_type: BxDFType,
    r: RGBSpectrum,
    fresnel: Box<dyn Fresnel>,
}

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

    fn sample(&self, wo: &Vec3, _u: &Point2F) -> BxDFSample {
        let wi = Vec3::new(-wo.x, -wo.y, wo.z);
        BxDFSample {
            wi,
            f: self.fresnel.evaluate(cos_theta(&wi)) * self.r / abs_cos_theta(&wi),
            pdf: 1.0,
            sampled_type: None,
        }
    }

    fn pdf(&self, _wo: &Vec3, _wi: &Vec3) -> Float {
        0.0
    }

    fn bxdf_type(&self) -> BxDFType {
        self.bxdf_type
    }
}

impl BxDF for SpecularTransmission {
    fn f(&self, _wo: &Vec3, _wi: &Vec3) -> RGBSpectrum {
        RGBSpectrum::default()
    }

    fn sample(&self, wo: &Vec3, _u: &Point2F) -> BxDFSample {
        // Determine which eta is incident or transmitted.
        let entering = cos_theta(wo) > 0.0;
        let eta_i = if entering { self.eta_a } else { self.eta_b };
        let eta_t = if entering { self.eta_b } else { self.eta_a };

        // Compute ray direction for specular transmission.
        if let Some(wi) = refract(
            wo,
            &Normal::new(0.0, 0.0, 1.0).face_forward(&Normal::from(*wo)),
            eta_i / eta_t,
        ) {
            let mut f = self.t * (RGBSpectrum::new(1.0) - self.fresnel.evaluate(cos_theta(&wi)));

            // Account for non-symmetry with transmission to different medium.
            if let TransportMode::Radiance = self.mode {
                f *= (eta_i * eta_i) / (eta_t * eta_t);
            }
            f /= abs_cos_theta(&wi);

            BxDFSample {
                wi,
                f,
                pdf: 1.0,
                sampled_type: None,
            }
        } else {
            BxDFSample {
                wi: Vec3::default(),
                f: RGBSpectrum::default(),
                pdf: 0.0,
                sampled_type: None,
            }
        }
    }

    fn pdf(&self, _wo: &Vec3, _wi: &Vec3) -> Float {
        0.0
    }

    fn bxdf_type(&self) -> BxDFType {
        self.bxdf_type
    }
}
