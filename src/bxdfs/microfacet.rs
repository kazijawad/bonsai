use crate::{
    base::{
        bxdf::{BxDF, BxDFType, BSDF_GLOSSY, BSDF_REFLECTION, BSDF_TRANSMISSION},
        fresnel::{Fresnel, FresnelDielectric},
        material::TransportMode,
        microfacet::MicrofacetDistribution,
        spectrum::{CoefficientSpectrum, Spectrum},
    },
    geometries::{normal::Normal, point2::Point2, vec3::Vec3},
    utils::{
        bxdf::{abs_cos_theta, cos_theta, reflect, refract, same_hemisphere},
        math::Float,
    },
};

pub struct MicrofacetReflection {
    bxdf_type: BxDFType,
    r: Spectrum,
    distribution: Box<dyn MicrofacetDistribution>,
    fresnel: Box<dyn Fresnel>,
}

pub struct MicrofactTransmission {
    bxdf_type: BxDFType,
    t: Spectrum,
    distribution: Box<dyn MicrofacetDistribution>,
    eta_a: Float,
    eta_b: Float,
    fresnel: FresnelDielectric,
    mode: TransportMode,
}

impl MicrofacetReflection {
    pub fn new(
        r: Spectrum,
        distribution: Box<dyn MicrofacetDistribution>,
        fresnel: Box<dyn Fresnel>,
    ) -> Self {
        Self {
            bxdf_type: BSDF_REFLECTION | BSDF_GLOSSY,
            r,
            distribution,
            fresnel,
        }
    }
}

impl MicrofactTransmission {
    pub fn new(
        t: Spectrum,
        distribution: Box<dyn MicrofacetDistribution>,
        eta_a: Float,
        eta_b: Float,
        mode: TransportMode,
    ) -> Self {
        Self {
            bxdf_type: BSDF_TRANSMISSION | BSDF_GLOSSY,
            t,
            distribution,
            eta_a,
            eta_b,
            fresnel: FresnelDielectric::new(eta_a, eta_b),
            mode,
        }
    }
}

impl BxDF for MicrofacetReflection {
    fn f(&self, wo: &Vec3, wi: &Vec3) -> Spectrum {
        let cos_theta_o = abs_cos_theta(wo);
        let cos_theta_i = abs_cos_theta(wi);
        let mut wh = wi + wo;

        // Handle degenerate cases for microfacet reflection.
        if cos_theta_i == 0.0 || cos_theta_o == 0.0 {
            return Spectrum::default();
        }
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return Spectrum::default();
        }

        wh = wh.normalize();

        // For the fresnel evaluation, make sure wh is in the same hemisphere
        // as the surface normal, so that total internal reflection is handled
        // correctly.
        let f = self
            .fresnel
            .evaluate(wi.dot(&wh.face_forward(&Vec3::new(0.0, 0.0, 1.0))));

        self.r * self.distribution.d(&wh) * self.distribution.g(wo, wi) * f
            / (4.0 * cos_theta_i * cos_theta_o)
    }

    fn sample_f(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        sample: &Point2,
        pdf: &mut Float,
        sampled_type: &mut Option<BxDFType>,
    ) -> Spectrum {
        // Sample microfacet orientation wh and reflected direction wi.
        if wo.z == 0.0 {
            return Spectrum::default();
        }

        let wh = self.distribution.sample_wh(wo, sample);
        if wo.dot(&wh) < 0.0 {
            return Spectrum::default();
        }

        *wi = reflect(wo, &wh);
        if !same_hemisphere(wo, &wi) {
            return Spectrum::default();
        }

        // Compute PDF of wi for microfacet reflection.
        *pdf = self.distribution.pdf(wo, &wh) / (4.0 * wo.dot(&wh));

        self.f(wo, &wi)
    }

    fn pdf(&self, wo: &Vec3, wi: &Vec3) -> Float {
        if !same_hemisphere(wo, wi) {
            return 0.0;
        }

        let wh = (wo + wi).normalize();

        self.distribution.pdf(wo, &wh) / (4.0 * wo.dot(&wh))
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }
}

impl BxDF for MicrofactTransmission {
    fn f(&self, wo: &Vec3, wi: &Vec3) -> Spectrum {
        if same_hemisphere(wo, wi) {
            return Spectrum::default();
        }

        let cos_theta_o = cos_theta(wo);
        let cos_theta_i = cos_theta(wi);
        if cos_theta_i == 0.0 || cos_theta_o == 0.0 {
            return Spectrum::default();
        }

        // Compute wh from wo and wi for microfacet transmission.
        let eta = if cos_theta_o > 0.0 {
            self.eta_b / self.eta_a
        } else {
            self.eta_a / self.eta_b
        };

        let mut wh = (*wo + wi * eta).normalize();
        if wh.z < 0.0 {
            wh = -wh;
        }

        // Check for same side.
        if wo.dot(&wh) * wi.dot(&wh) > 0.0 {
            return Spectrum::default();
        }

        let f = self.fresnel.evaluate(wo.dot(&wh));

        let sqrt_denom = wo.dot(&wh) + eta * wi.dot(&wh);
        let factor = if let TransportMode::Radiance = self.mode {
            1.0 / eta
        } else {
            1.0
        };

        (Spectrum::new(1.0) - f)
            * self.t
            * (self.distribution.d(&wh)
                * self.distribution.g(wo, wi)
                * eta
                * eta
                * wi.abs_dot(&wh)
                * wo.abs_dot(&wh)
                * factor
                * factor
                / (cos_theta_i * cos_theta_o * sqrt_denom * sqrt_denom))
                .abs()
    }

    fn sample_f(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        sample: &Point2,
        pdf: &mut Float,
        sampled_type: &mut Option<BxDFType>,
    ) -> Spectrum {
        if wo.z == 0.0 {
            return Spectrum::default();
        }

        let wh = self.distribution.sample_wh(wo, sample);
        if wo.dot(&wh) < 0.0 {
            return Spectrum::default();
        }

        let eta = if cos_theta(wo) > 0.0 {
            self.eta_a / self.eta_b
        } else {
            self.eta_b / self.eta_a
        };

        if !refract(wo, &Normal::from(wh), eta, wi) {
            return Spectrum::default();
        }

        *pdf = self.pdf(wo, &wi);

        self.f(wo, &wi)
    }

    fn pdf(&self, wo: &Vec3, wi: &Vec3) -> Float {
        if same_hemisphere(wo, wi) {
            return 0.0;
        }

        // Compute wh from wo and wi for microfacet transmission.
        let eta = if cos_theta(wo) > 0.0 {
            self.eta_b / self.eta_a
        } else {
            self.eta_a / self.eta_b
        };

        let wh = (*wo + wi * eta).normalize();

        if wo.dot(&wh) * wi.dot(&wh) > 0.0 {
            return 0.0;
        }

        // Compute change of variables for microfacet transmission.
        let sqrt_denom = wo.dot(&wh) + eta * wi.dot(&wh);
        let dwh_dwi = ((eta * eta * wi.dot(&wh)) / (sqrt_denom * sqrt_denom)).abs();

        self.distribution.pdf(wo, &wh) * dwh_dwi
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }
}
