use crate::{
    base::spectrum::{CoefficientSpectrum, Spectrum},
    geometries::{point2::Point2, vec3::Vec3},
    utils::{
        bxdf::abs_cos_theta,
        math::{Float, PI},
        sampling::{uniform_hemisphere_pdf, uniform_sample_hemisphere},
    },
};

pub type BxDFType = i32;

pub const BSDF_REFLECTION: i32 = 1 << 0;
pub const BSDF_TRANSMISSION: i32 = 1 << 1;
pub const BSDF_DIFFUSE: i32 = 1 << 2;
pub const BSDF_GLOSSY: i32 = 1 << 3;
pub const BSDF_SPECULAR: i32 = 1 << 4;
pub const BSDF_ALL: i32 =
    BSDF_DIFFUSE | BSDF_GLOSSY | BSDF_SPECULAR | BSDF_REFLECTION | BSDF_TRANSMISSION;

pub trait BxDF: Send + Sync {
    fn distribution(&self, wo: &Vec3, wi: &Vec3) -> Spectrum;

    fn sample_distribution(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        sample: &Point2,
        pdf: &mut Float,
        sampled_type: Option<BxDFType>,
    ) -> Spectrum;

    fn hemispherical_directional_reflectance(
        &self,
        wo: &Vec3,
        num_samples: usize,
        samples: &[Point2],
    ) -> Spectrum {
        let mut reflection_factor = Spectrum::new(0.0);

        for i in 0..num_samples {
            let mut wi = Vec3::default();
            let mut pdf = 0.0;
            let factor = self.sample_distribution(wo, &mut wi, &samples[i], &mut pdf, None);
            if pdf > 0.0 {
                reflection_factor += factor * abs_cos_theta(&wi) / pdf;
            }
        }

        reflection_factor / Spectrum::new(num_samples as Float)
    }

    fn hemispherical_hemispherical_reflectance(
        &self,
        num_samples: usize,
        u1: &[Point2],
        u2: &[Point2],
    ) -> Spectrum {
        let mut reflection_factor = Spectrum::new(0.0);

        for i in 0..num_samples {
            let wo = uniform_sample_hemisphere(&u1[i]);
            let mut wi = Vec3::default();

            let pdf_o = uniform_hemisphere_pdf();
            let mut pdf_i = 0.0;

            let factor = self.sample_distribution(&wo, &mut wi, &u2[i], &mut pdf_i, None);
            if pdf_i > 0.0 {
                reflection_factor +=
                    factor * abs_cos_theta(&wi) * abs_cos_theta(&wo) / (pdf_o * pdf_i);
            }
        }

        reflection_factor / (PI * num_samples as Float)
    }

    fn pdf(&self, wo: &Vec3, wi: &Vec3) -> Float;

    fn matches_flags(&self, t: BxDFType) -> bool;
}
