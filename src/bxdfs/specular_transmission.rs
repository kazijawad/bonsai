use crate::{
    base::{
        bxdf::{BxDF, BxDFType, BSDF_SPECULAR, BSDF_TRANSMISSION},
        fresnel::{Fresnel, FresnelDielectric},
        material::TransportMode,
        spectrum::{CoefficientSpectrum, Spectrum},
    },
    geometries::{normal::Normal, point2::Point2, vec3::Vec3},
    utils::{
        bxdf::{abs_cos_theta, cos_theta, refract},
        math::Float,
    },
};

pub struct SpecularTransmission {
    bxdf_type: BxDFType,
    transmission_factor: Spectrum,
    fresnel: FresnelDielectric,
    eta_a: Float,
    eta_b: Float,
    mode: TransportMode,
}

impl SpecularTransmission {
    pub fn new(
        transmission_factor: &Spectrum,
        eta_a: Float,
        eta_b: Float,
        mode: TransportMode,
    ) -> Self {
        Self {
            bxdf_type: BSDF_TRANSMISSION | BSDF_SPECULAR,
            transmission_factor: transmission_factor.clone(),
            fresnel: FresnelDielectric::new(eta_a, eta_b),
            eta_a,
            eta_b,
            mode,
        }
    }
}

impl BxDF for SpecularTransmission {
    fn distribution(&self, wo: &Vec3, wi: &Vec3) -> Spectrum {
        Spectrum::new(0.0)
    }

    fn sample_distribution(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        sample: &Point2,
        pdf: &mut Float,
        sampled_type: Option<BxDFType>,
    ) -> Spectrum {
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
            return Spectrum::new(0.0);
        }

        *pdf = 1.0;
        let mut factor =
            self.transmission_factor * (Spectrum::new(1.0) - self.fresnel.evaluate(cos_theta(&wi)));

        // Account for non-symmetry with transmission to different medium.
        if let TransportMode::Radiance = self.mode {
            factor *= (eta_i * eta_i) / (eta_t * eta_t);
        }

        factor / abs_cos_theta(&wi)
    }

    fn pdf(&self, wo: &Vec3, wi: &Vec3) -> Float {
        0.0
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }
}
