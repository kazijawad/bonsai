use crate::{
    base::{
        constants::{Float, PI},
        sampling::{cosine_sample_hemisphere, uniform_hemisphere_pdf, uniform_sample_hemisphere},
    },
    geometries::{point2::Point2F, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
    utils::bxdf::{abs_cos_theta, same_hemisphere},
};

pub type BxDFType = u8;

pub const BSDF_REFLECTION: BxDFType = 1 << 0;
pub const BSDF_TRANSMISSION: BxDFType = 1 << 1;
pub const BSDF_DIFFUSE: BxDFType = 1 << 2;
pub const BSDF_GLOSSY: BxDFType = 1 << 3;
pub const BSDF_SPECULAR: BxDFType = 1 << 4;
pub const BSDF_ALL: BxDFType =
    BSDF_DIFFUSE | BSDF_GLOSSY | BSDF_SPECULAR | BSDF_REFLECTION | BSDF_TRANSMISSION;

pub trait BxDF: Send + Sync {
    fn f(&self, wo: &Vec3, wi: &Vec3) -> RGBSpectrum;

    fn sample(&self, wo: &Vec3, u: &Point2F) -> (Vec3, RGBSpectrum, Float, Option<BxDFType>) {
        // Cosine-sample the hemisphere, flipping the direction if necessary.
        let mut wi = cosine_sample_hemisphere(u);
        if wo.z < 0.0 {
            wi.z *= -1.0;
        }
        let radiance = self.f(wo, &wi);
        let pdf = self.pdf(wo, &wi);
        (wi, radiance, pdf, None)
    }

    fn rho_hd(&self, wo: &Vec3, num_samples: usize, u: &[Point2F]) -> RGBSpectrum {
        let mut reflectance = RGBSpectrum::default();
        for i in 0..num_samples {
            let (wi, factor, pdf, _) = self.sample(wo, &u[i]);
            if pdf > 0.0 {
                reflectance += factor * abs_cos_theta(&wi) / pdf;
            }
        }
        reflectance / (num_samples as Float)
    }

    fn rho_hh(&self, num_samples: usize, u1: &[Point2F], u2: &[Point2F]) -> RGBSpectrum {
        let mut reflectance = RGBSpectrum::default();
        for i in 0..num_samples {
            let wo = uniform_sample_hemisphere(&u1[i]);
            let pdf_o = uniform_hemisphere_pdf();
            let (wi, factor, pdf_i, _) = self.sample(&wo, &u2[i]);
            if pdf_i > 0.0 {
                reflectance += factor * abs_cos_theta(&wi) * abs_cos_theta(&wo) / (pdf_o * pdf_i);
            }
        }
        reflectance / (PI * num_samples as Float)
    }

    fn pdf(&self, wo: &Vec3, wi: &Vec3) -> Float {
        if same_hemisphere(wo, wi) {
            abs_cos_theta(wi) * (1.0 / PI)
        } else {
            0.0
        }
    }

    fn bxdf_type(&self) -> BxDFType;

    fn matches_flags(&self, flags: BxDFType) -> bool {
        let bxdf_type = self.bxdf_type();
        (bxdf_type & flags) == bxdf_type
    }
}
