use crate::{
    base::{
        bxdf::{
            abs_cos_theta, cos_theta, reflect, refract, same_hemisphere, BxDF, BxDFSample,
            BxDFType, BSDF_GLOSSY, BSDF_REFLECTION, BSDF_TRANSMISSION,
        },
        constants::Float,
        fresnel::{Fresnel, FresnelDielectric},
        material::TransportMode,
        microfacet::MicrofacetDistribution,
    },
    geometries::{normal::Normal, point2::Point2F, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
};

pub struct MicrofacetReflection {
    bxdf_type: BxDFType,
    r: RGBSpectrum,
    distribution: Box<dyn MicrofacetDistribution>,
    fresnel: Box<dyn Fresnel>,
}

pub struct MicrofacetTransmission {
    bxdf_type: BxDFType,
    t: RGBSpectrum,
    distribution: Box<dyn MicrofacetDistribution>,
    eta_a: Float,
    eta_b: Float,
    fresnel: FresnelDielectric,
    mode: TransportMode,
}

impl MicrofacetReflection {
    pub fn new(
        r: RGBSpectrum,
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

impl MicrofacetTransmission {
    pub fn new(
        t: RGBSpectrum,
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
    fn f(&self, wo: &Vec3, wi: &Vec3) -> RGBSpectrum {
        let cos_theta_o = abs_cos_theta(wo);
        let cos_theta_i = abs_cos_theta(wi);
        let mut wh = wi + wo;

        // Handle degenerate cases for microfacet reflection.
        if cos_theta_i == 0.0 || cos_theta_o == 0.0 {
            return RGBSpectrum::default();
        }
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return RGBSpectrum::default();
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

    fn sample(&self, wo: &Vec3, u: &Point2F) -> BxDFSample {
        // Sample microfacet orientation wh and reflected direction wi.
        if wo.z == 0.0 {
            return BxDFSample {
                wi: Vec3::default(),
                f: RGBSpectrum::default(),
                pdf: 0.0,
                sampled_type: None,
            };
        }

        let wh = self.distribution.sample(wo, u);
        if wo.dot(&wh) < 0.0 {
            return BxDFSample {
                wi: Vec3::default(),
                f: RGBSpectrum::default(),
                pdf: 0.0,
                sampled_type: None,
            };
        }

        let wi = reflect(wo, &wh);
        if !same_hemisphere(wo, &wi) {
            return BxDFSample {
                wi: Vec3::default(),
                f: RGBSpectrum::default(),
                pdf: 0.0,
                sampled_type: None,
            };
        }

        let f = self.f(wo, &wi);

        // Compute PDF of wi for microfacet reflection.
        let pdf = self.distribution.pdf(wo, &wh) / (4.0 * wo.dot(&wh));

        BxDFSample {
            wi,
            f,
            pdf,
            sampled_type: None,
        }
    }

    fn pdf(&self, wo: &Vec3, wi: &Vec3) -> Float {
        if !same_hemisphere(wo, wi) {
            return 0.0;
        }

        let wh = (wo + wi).normalize();

        self.distribution.pdf(wo, &wh) / (4.0 * wo.dot(&wh))
    }

    fn bxdf_type(&self) -> BxDFType {
        self.bxdf_type
    }
}

impl BxDF for MicrofacetTransmission {
    fn f(&self, wo: &Vec3, wi: &Vec3) -> RGBSpectrum {
        if same_hemisphere(wo, wi) {
            return RGBSpectrum::default();
        }

        let cos_theta_o = cos_theta(wo);
        let cos_theta_i = cos_theta(wi);
        if cos_theta_i == 0.0 || cos_theta_o == 0.0 {
            return RGBSpectrum::default();
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
            return RGBSpectrum::default();
        }

        let f = self.fresnel.evaluate(wo.dot(&wh));

        let sqrt_denom = wo.dot(&wh) + eta * wi.dot(&wh);
        let factor = if let TransportMode::Radiance = self.mode {
            1.0 / eta
        } else {
            1.0
        };

        (RGBSpectrum::new(1.0) - f)
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

    fn sample(&self, wo: &Vec3, u: &Point2F) -> BxDFSample {
        if wo.z == 0.0 {
            return BxDFSample {
                wi: Vec3::default(),
                f: RGBSpectrum::default(),
                pdf: 0.0,
                sampled_type: None,
            };
        }

        let wh = self.distribution.sample(wo, u);
        if wo.dot(&wh) < 0.0 {
            return BxDFSample {
                wi: Vec3::default(),
                f: RGBSpectrum::default(),
                pdf: 0.0,
                sampled_type: None,
            };
        }

        let eta = if cos_theta(wo) > 0.0 {
            self.eta_a / self.eta_b
        } else {
            self.eta_b / self.eta_a
        };

        if let Some(wi) = refract(wo, &Normal::from(wh), eta) {
            let f = self.f(wo, &wi);
            let pdf = self.pdf(wo, &wi);
            BxDFSample {
                wi,
                f,
                pdf,
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

    fn bxdf_type(&self) -> BxDFType {
        self.bxdf_type
    }
}
