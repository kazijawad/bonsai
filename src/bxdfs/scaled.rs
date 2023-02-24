use crate::{
    base::{
        bxdf::{BxDF, BxDFType},
        spectrum::Spectrum,
    },
    geometries::{point2::Point2, vec3::Vec3},
    utils::math::Float,
};

pub struct ScaledBxDF {
    pub bxdf_type: BxDFType,
    bxdf: Box<dyn BxDF>,
    scale: Spectrum,
}

impl ScaledBxDF {
    pub fn new(bxdf_type: BxDFType, bxdf: Box<dyn BxDF>, scale: Spectrum) -> Self {
        Self {
            bxdf_type,
            bxdf,
            scale,
        }
    }
}

impl BxDF for ScaledBxDF {
    fn distribution(&self, wo: &Vec3, wi: &Vec3) -> Spectrum {
        self.scale * self.bxdf.distribution(wo, wi)
    }

    fn sample_distribution(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        sample: &Point2,
        pdf: &mut Float,
        sampled_type: &mut Option<BxDFType>,
    ) -> Spectrum {
        self.scale
            * self
                .bxdf
                .sample_distribution(wo, wi, sample, pdf, sampled_type)
    }

    fn hemispherical_directional_reflectance(
        &self,
        wo: &Vec3,
        num_samples: usize,
        samples: &[Point2],
    ) -> Spectrum {
        self.scale
            * self
                .bxdf
                .hemispherical_directional_reflectance(wo, num_samples, samples)
    }

    fn hemispherical_hemispherical_reflectance(
        &self,
        num_samples: usize,
        samples_1: &[Point2],
        samples_2: &[Point2],
    ) -> Spectrum {
        self.scale
            * self
                .bxdf
                .hemispherical_hemispherical_reflectance(num_samples, samples_1, samples_2)
    }

    fn pdf(&self, wo: &Vec3, wi: &Vec3) -> Float {
        self.bxdf.pdf(wo, wi)
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }
}
