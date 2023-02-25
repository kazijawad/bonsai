use crate::{
    base::{
        bxdf::{BxDF, BxDFType},
        spectrum::Spectrum,
    },
    geometries::{point2::Point2, vec3::Vec3},
    utils::math::Float,
};

pub struct ScaledBxDF {
    bxdf_type: BxDFType,
    bxdf: Box<dyn BxDF>,
    scale: Spectrum,
}

impl ScaledBxDF {
    pub fn new(bxdf: Box<dyn BxDF>, scale: Spectrum) -> Self {
        Self {
            bxdf_type: bxdf.get_type(),
            bxdf,
            scale,
        }
    }
}

impl BxDF for ScaledBxDF {
    fn f(&self, wo: &Vec3, wi: &Vec3) -> Spectrum {
        self.scale * self.bxdf.f(wo, wi)
    }

    fn sample_f(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        sample: &Point2,
        pdf: &mut Float,
        sampled_type: &mut Option<BxDFType>,
    ) -> Spectrum {
        self.scale * self.bxdf.sample_f(wo, wi, sample, pdf, sampled_type)
    }

    fn rho_hd(&self, wo: &Vec3, num_samples: usize, samples: &[Point2]) -> Spectrum {
        self.scale * self.bxdf.rho_hd(wo, num_samples, samples)
    }

    fn rho_hh(&self, num_samples: usize, u1: &[Point2], u2: &[Point2]) -> Spectrum {
        self.scale * self.bxdf.rho_hh(num_samples, u1, u2)
    }

    fn pdf(&self, wo: &Vec3, wi: &Vec3) -> Float {
        self.bxdf.pdf(wo, wi)
    }

    fn get_type(&self) -> BxDFType {
        self.bxdf_type
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }
}
