use crate::{
    base::{
        bxdf::{
            abs_cos_theta, cos_theta, fresnel_dielectric, reflect, refract, same_hemisphere, BxDF,
            BxDFSample, BxDFType, BSDF_GLOSSY, BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION,
        },
        constants::{Float, PI},
        material::TransportMode,
        microfacet::MicrofacetDistribution,
        sampling::cosine_sample_hemisphere,
    },
    geometries::{normal::Normal, point2::Point2F, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
};

pub struct FresnelSpecular {
    bxdf_type: BxDFType,
    r: RGBSpectrum,
    t: RGBSpectrum,
    eta_a: Float,
    eta_b: Float,
    mode: TransportMode,
}

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

    fn sample(&self, wo: &Vec3, u: &Point2F) -> BxDFSample {
        let f = fresnel_dielectric(cos_theta(wo), self.eta_a, self.eta_b);
        if u[0] < f {
            // Compute specular reflection.
            let wi = Vec3::new(-wo.x, -wo.y, wo.z);
            let pdf = f;
            let f = RGBSpectrum::new(f) * self.r / abs_cos_theta(&wi);
            let sampled_type = Some(BSDF_SPECULAR | BSDF_REFLECTION);
            BxDFSample {
                wi,
                f,
                pdf,
                sampled_type: Some(BSDF_SPECULAR | BSDF_REFLECTION),
            }
        } else {
            // Compute specular transmission.
            // Figure out which eta is incident and which is transmitted.
            let entering = cos_theta(wo) > 0.0;
            let eta_i = if entering { self.eta_a } else { self.eta_b };
            let eta_t = if entering { self.eta_b } else { self.eta_a };

            // Compute ray direction for specular transmission.
            if let Some(wi) = refract(
                wo,
                &Normal::new(0.0, 0.0, 1.0).face_forward(&Normal::from(*wo)),
                eta_i / eta_t,
            ) {
                let mut ft = self.t * (1.0 - f);
                // Account for non-symmetry with transmission to different medium.
                if let TransportMode::Radiance = self.mode {
                    ft *= (eta_i * eta_i) / (eta_t * eta_t);
                }
                BxDFSample {
                    wi,
                    f: ft / abs_cos_theta(&wi),
                    pdf: 1.0 - f,
                    sampled_type: Some(BSDF_SPECULAR | BSDF_TRANSMISSION),
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
    }

    fn pdf(&self, _wo: &Vec3, _wi: &Vec3) -> Float {
        0.0
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

    fn sample(&self, wo: &Vec3, u: &Point2F) -> BxDFSample {
        let mut u = u.clone();

        let mut wi;
        if u[0] < 0.5 {
            u[0] = (2.0 * u[0]).min(1.0 - Float::EPSILON);

            // Cosine-sample the hemisphere, flipping the direction if necessary.
            wi = cosine_sample_hemisphere(&u);
            if wo.z < 0.0 {
                wi.z *= -1.0;
            }
        } else {
            u[0] = (2.0 * (u[0] - 0.5)).min(1.0 - Float::EPSILON);

            // Sample microfacet orientation wh and reflected direction wi.
            let wh = self.distribution.sample(wo, &u);
            wi = reflect(wo, &wh);
            if !same_hemisphere(wo, &wi) {
                return BxDFSample {
                    wi,
                    f: RGBSpectrum::default(),
                    pdf: 0.0,
                    sampled_type: None,
                };
            }
        }

        let f = self.f(wo, &wi);
        let pdf = self.pdf(wo, &wi);

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
        let wh_pdf = self.distribution.pdf(wo, &wh);

        0.5 * (abs_cos_theta(wi) * (1.0 / PI) + wh_pdf / (4.0 * wo.dot(&wh)))
    }

    fn bxdf_type(&self) -> BxDFType {
        self.bxdf_type
    }
}
