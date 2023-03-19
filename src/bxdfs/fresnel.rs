use crate::{
    base::{
        bxdf::{BxDF, BxDFType, BSDF_GLOSSY, BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION},
        constants::{Float, PI},
        material::TransportMode,
        microfacet::MicrofacetDistribution,
    },
    geometries::{normal::Normal, point2::Point2, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
    utils::{
        bxdf::{abs_cos_theta, cos_theta, fresnel_dielectric, reflect, refract, same_hemisphere},
        sampling::cosine_sample_hemisphere,
    },
};

#[derive(Clone)]
pub struct FresnelSpecular {
    bxdf_type: BxDFType,
    r: RGBSpectrum,
    t: RGBSpectrum,
    eta_a: Float,
    eta_b: Float,
    mode: TransportMode,
}

#[derive(Clone)]
pub struct FresnelBlend {
    bxdf_type: BxDFType,
    rd: RGBSpectrum,
    rs: RGBSpectrum,
    distribution: Box<dyn MicrofacetDistribution>,
}

impl FresnelSpecular {
    pub fn new(
        r: RGBSpectrum,
        t: RGBSpectrum,
        eta_a: Float,
        eta_b: Float,
        mode: TransportMode,
    ) -> Self {
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
    pub fn new(
        rd: RGBSpectrum,
        rs: RGBSpectrum,
        distribution: Box<dyn MicrofacetDistribution>,
    ) -> Self {
        Self {
            bxdf_type: BSDF_REFLECTION | BSDF_GLOSSY,
            rd,
            rs,
            distribution,
        }
    }

    fn schlick_fresnel(&self, cos_theta: Float) -> RGBSpectrum {
        let pow5 = |v: Float| -> Float { (v * v) * (v * v) * v };
        self.rs + RGBSpectrum::new(pow5(1.0 - cos_theta)) * (RGBSpectrum::new(1.0) - self.rs)
    }
}

impl BxDF for FresnelSpecular {
    fn f(&self, _wo: &Vec3, _wi: &Vec3) -> RGBSpectrum {
        RGBSpectrum::default()
    }

    fn sample_f(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        sample: &Point2,
        pdf: &mut Float,
        sampled_type: &mut Option<BxDFType>,
    ) -> RGBSpectrum {
        let f = fresnel_dielectric(cos_theta(wo), self.eta_a, self.eta_b);
        if sample[0] < f {
            // Compute specular reflection.
            // Compute perfect specular reflection direction.
            *wi = Vec3::new(-wo.x, -wo.y, wo.z);

            if sampled_type.is_some() {
                *sampled_type = Some(BSDF_SPECULAR | BSDF_REFLECTION);
            }

            *pdf = f;

            RGBSpectrum::new(f) * self.r / abs_cos_theta(&wi)
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
                return RGBSpectrum::default();
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

impl BxDF for FresnelBlend {
    fn f(&self, wo: &Vec3, wi: &Vec3) -> RGBSpectrum {
        let pow5 = |v: Float| -> Float { (v * v) * (v * v) * v };

        let diffuse = RGBSpectrum::new(28.0 / (23.0 * PI))
            * self.rd
            * (RGBSpectrum::new(1.0) - self.rs)
            * (1.0 - pow5(1.0 - 0.5 * abs_cos_theta(wi)))
            * (1.0 - pow5(1.0 - 0.5 * abs_cos_theta(wo)));

        let mut wh = wi + wo;
        if wh.x == 0.0 && wh.y == 0.0 && wh.z == 0.0 {
            return RGBSpectrum::default();
        }
        wh = wh.normalize();

        let specular = RGBSpectrum::new(
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
        _sampled_type: &mut Option<BxDFType>,
    ) -> RGBSpectrum {
        let mut sample = sample.clone();
        if sample.x < 0.5 {
            sample.x = (2.0 * sample[0]).min(1.0 - Float::EPSILON);

            // Cosine-sample the hemisphere, flipping the direction if necessary.
            *wi = cosine_sample_hemisphere(&sample);
            if wo.z < 0.0 {
                wi.z *= -1.0;
            }
        } else {
            sample.x = (2.0 * (sample.x - 0.5)).min(1.0 - Float::EPSILON);

            // Sample microfacet orientation wh and reflected direction wi.
            let wh = self.distribution.sample_wh(wo, &sample);
            *wi = reflect(wo, &wh);
            if !same_hemisphere(wo, &wi) {
                return RGBSpectrum::default();
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

    fn bxdf_type(&self) -> BxDFType {
        self.bxdf_type
    }
}
