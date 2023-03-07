use crate::{
    base::{
        bxdf::{BxDF, BxDFType, BSDF_DIFFUSE, BSDF_REFLECTION},
        spectrum::Spectrum,
    },
    geometries::vec3::Vec3,
    utils::{
        bxdf::{abs_cos_theta, cos_phi, sin_phi, sin_theta},
        math::{Float, PI},
    },
};

#[derive(Debug, Clone)]
pub struct OrenNayer {
    bxdf_type: BxDFType,
    r: Spectrum,
    a: Float,
    b: Float,
}

impl OrenNayer {
    pub fn new(r: Spectrum, sigma: Float) -> Self {
        let sigma = sigma.to_radians();
        let sigma_2 = sigma * sigma;
        Self {
            bxdf_type: BSDF_REFLECTION | BSDF_DIFFUSE,
            r,
            a: 1.0 - (sigma_2 / (2.0 * (sigma_2 + 0.33))),
            b: 0.45 * sigma_2 / (sigma_2 + 0.09),
        }
    }
}

impl BxDF for OrenNayer {
    fn f(&self, wo: &Vec3, wi: &Vec3) -> Spectrum {
        let sin_theta_i = sin_theta(wi);
        let sin_theta_o = sin_theta(wo);

        // Compute cosine term of Oren-Nayar model.
        let max_cos = if sin_theta_i > 1e-4 && sin_theta_o > 1e-4 {
            let sin_phi_i = sin_phi(wi);
            let cos_phi_i = cos_phi(wi);

            let sin_phi_o = sin_phi(wo);
            let cos_phi_o = cos_phi(wo);

            let cos_diff = cos_phi_i * cos_phi_o + sin_phi_i * sin_phi_o;

            Float::max(0.0, cos_diff)
        } else {
            0.0
        };

        // Compute sine and tangent terms of Oren-Nayar model.
        let (sin_alpha, tan_beta) = if abs_cos_theta(wi) > abs_cos_theta(wo) {
            (sin_theta_o, sin_theta_i / abs_cos_theta(wi))
        } else {
            (sin_theta_i, sin_theta_o / abs_cos_theta(wo))
        };

        self.r * (1.0 / PI) * (self.a + self.b * max_cos * sin_alpha * tan_beta)
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }
}
