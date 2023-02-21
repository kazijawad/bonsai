use crate::{
    base::{
        bxdf::{BxDF, BxDFType, BSDF_DIFFUSE, BSDF_REFLECTION},
        spectrum::Spectrum,
    },
    geometries::{point2::Point2, vec3::Vec3},
    utils::math::PI,
};

pub struct LambertianReflection {
    bxdf_type: BxDFType,
    factor: Spectrum,
}

impl LambertianReflection {
    pub fn new(factor: &Spectrum) -> Self {
        Self {
            bxdf_type: BSDF_REFLECTION | BSDF_DIFFUSE,
            factor: factor.clone(),
        }
    }
}

impl BxDF for LambertianReflection {
    fn distribution(&self, wo: &Vec3, wi: &Vec3) -> Spectrum {
        self.factor * (1.0 / PI)
    }

    fn hemispherical_directional_reflectance(
        &self,
        wo: &Vec3,
        num_samples: usize,
        samples: &[Point2],
    ) -> Spectrum {
        self.factor
    }

    fn hemispherical_hemispherical_reflectance(
        &self,
        num_samples: usize,
        u1: &[Point2],
        u2: &[Point2],
    ) -> Spectrum {
        self.factor
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }
}
