use crate::{
    base::{
        bxdf::{BxDF, BxDFType, BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION},
        material::TransportMode,
        spectrum::{CoefficientSpectrum, Spectrum},
    },
    geometries::{normal::Normal, point2::Point2, vec3::Vec3},
    utils::{
        bxdf::{abs_cos_theta, cos_theta, fresnel_dielectric, refract},
        math::Float,
    },
};

pub struct FresnelSpecular {
    bxdf_type: BxDFType,
    r: Spectrum,
    t: Spectrum,
    eta_a: Float,
    eta_b: Float,
    mode: TransportMode,
}

impl FresnelSpecular {
    pub fn new(
        r: &Spectrum,
        t: &Spectrum,
        eta_a: Float,
        eta_b: Float,
        mode: TransportMode,
    ) -> Self {
        Self {
            bxdf_type: BSDF_REFLECTION | BSDF_TRANSMISSION | BSDF_SPECULAR,
            r: r.clone(),
            t: t.clone(),
            eta_a,
            eta_b,
            mode,
        }
    }
}

impl BxDF for FresnelSpecular {
    fn distribution(&self, wo: &Vec3, wi: &Vec3) -> Spectrum {
        Spectrum::default()
    }

    fn sample_distribution(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        sample: &Point2,
        pdf: &mut Float,
        sampled_type: &mut Option<BxDFType>,
    ) -> Spectrum {
        let f = fresnel_dielectric(cos_theta(wo), self.eta_a, self.eta_b);
        if sample[0] < f {
            // Compute specular reflection.
            // Compute perfect specular reflection direction.
            *wi = Vec3::new(-wo.x, -wo.y, wo.z);

            if sampled_type.is_some() {
                *sampled_type = Some(BSDF_SPECULAR | BSDF_REFLECTION);
            }

            *pdf = f;

            Spectrum::new(f) * self.r / abs_cos_theta(&wi)
        } else {
            // Compute specular transmission.
            // Figure out which eta is incident and which is transmitted.
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
                return Spectrum::default();
            }

            let mut ft = self.t * (1.0 - f);
            // Account for non-symmetry with transmission to different medium.
            if let TransportMode::Radiance = self.mode {
                ft *= (eta_i * eta_i) / (eta_t * eta_t);
            }

            if sampled_type.is_some() {
                *sampled_type = Some(BSDF_SPECULAR | BSDF_TRANSMISSION);
            }

            *pdf = 1.0 - f;

            ft / abs_cos_theta(&wi)
        }
    }

    fn pdf(&self, wo: &Vec3, wi: &Vec3) -> Float {
        0.0
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }
}
