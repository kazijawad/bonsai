use crate::{
    base::{
        bxdf::{BxDF, BxDFType, BSDF_GLOSSY, BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION},
        material::TransportMode,
        microfacet::MicrofacetDistribution,
        spectrum::{CoefficientSpectrum, Spectrum},
    },
    geometries::{normal::Normal, point2::Point2, vec3::Vec3},
    utils::{
        bxdf::{abs_cos_theta, cos_theta, fresnel_dielectric, reflect, refract, same_hemisphere},
        math::{Float, ONE_MINUS_EPSILON, PI},
        sampling::cosine_sample_hemisphere,
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

pub struct FresnelBlend {
    bxdf_type: BxDFType,
    rd: Spectrum,
    rs: Spectrum,
    distribution: Box<dyn MicrofacetDistribution>,
}

impl FresnelSpecular {
    pub fn new(r: Spectrum, t: Spectrum, eta_a: Float, eta_b: Float, mode: TransportMode) -> Self {
        Self {
            bxdf_type: BSDF_REFLECTION | BSDF_TRANSMISSION | BSDF_SPECULAR,
            r,
            t,
            eta_a,
            eta_b,
            mode,
        }
    }
}

impl FresnelBlend {
    pub fn new(rd: Spectrum, rs: Spectrum, distribution: Box<dyn MicrofacetDistribution>) -> Self {
        Self {
            bxdf_type: BSDF_REFLECTION | BSDF_GLOSSY,
            rd,
            rs,
            distribution,
        }
    }

    fn schlick_fresnel(&self, cos_theta: Float) -> Spectrum {
        let pow5 = |v: Float| -> Float { (v * v) * (v * v) * v };
        self.rs + Spectrum::new(pow5(1.0 - cos_theta)) * (Spectrum::new(1.0) - self.rs)
    }
}

impl BxDF for FresnelSpecular {
    fn f(&self, wo: &Vec3, wi: &Vec3) -> Spectrum {
        Spectrum::default()
    }

    fn sample_f(
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

impl BxDF for FresnelBlend {
    fn f(&self, wo: &Vec3, wi: &Vec3) -> Spectrum {
        let pow5 = |v: Float| -> Float { (v * v) * (v * v) * v };

        let diffuse = Spectrum::new(28.0 / (23.0 * PI))
            * self.rd
            * (Spectrum::new(1.0) - self.rs)
            * (1.0 - pow5(1.0 - 0.5 * abs_cos_theta(wi)))
            * (1.0 - pow5(1.0 - 0.5 * abs_cos_theta(wo)));

        let mut wh = wi + wo;
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return Spectrum::default();
        }
        wh = wh.normalize();

        let specular = Spectrum::new(
            self.distribution.d(&wh)
                / (4.0 * wi.abs_dot(&wh) * abs_cos_theta(wi).max(abs_cos_theta(wo))),
        ) * self.schlick_fresnel(wi.dot(&wh));

        diffuse + specular
    }

    fn sample_f(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        sample: &Point2,
        pdf: &mut Float,
        sampled_type: &mut Option<BxDFType>,
    ) -> Spectrum {
        let mut sample = sample.clone();
        if sample.x < 0.5 {
            sample.x = (2.0 * sample[0]).min(ONE_MINUS_EPSILON);

            // Cosine-sample the hemisphere, flipping the direction if necessary.
            *wi = cosine_sample_hemisphere(&sample);
            if wo.z < 0.0 {
                wi.z *= -1.0;
            }
        } else {
            sample.x = (2.0 * (sample.x - 0.5)).min(ONE_MINUS_EPSILON);

            // Sample microfacet orientation wh and reflected direction wi.
            let wh = self.distribution.sample_wh(wo, &sample);
            *wi = reflect(wo, &wh);
            if !same_hemisphere(wo, &wi) {
                return Spectrum::default();
            }
        }

        *pdf = self.pdf(wo, &wi);

        self.f(wo, &wi)
    }

    fn pdf(&self, wo: &Vec3, wi: &Vec3) -> Float {
        if !same_hemisphere(wo, wi) {
            return 0.0;
        }

        let wh = (wo + wi).normalize();
        let wh_pdf = self.distribution.pdf(wo, &wh);

        0.5 * (abs_cos_theta(wi) * (1.0 / PI) + wh_pdf / (4.0 * wo.dot(&wh)))
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }
}
