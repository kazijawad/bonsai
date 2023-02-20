use crate::{
    base::{
        bxdf::{BxDF, BxDFType, BSDF_REFLECTION, BSDF_SPECULAR},
        fresnel::Fresnel,
        spectrum::{CoefficientSpectrum, Spectrum},
    },
    geometries::{point2::Point2, vec3::Vec3},
    utils::{
        bxdf::{abs_cos_theta, cos_theta},
        math::Float,
    },
};

pub struct SpecularReflection<'a> {
    bxdf_type: BxDFType,
    reflection_factor: Spectrum,
    fresnel: &'a dyn Fresnel,
}

impl<'a> SpecularReflection<'a> {
    pub fn new(reflection_factor: &Spectrum, fresnel: &'a dyn Fresnel) -> Self {
        Self {
            bxdf_type: BSDF_REFLECTION | BSDF_SPECULAR,
            reflection_factor: reflection_factor.clone(),
            fresnel,
        }
    }
}

impl<'a> BxDF for SpecularReflection<'a> {
    fn distribution(&self, wo: &Vec3, wi: &Vec3) -> Spectrum {
        Spectrum::new(0.0)
    }

    fn sample_distribution(
        &self,
        wo: &Vec3,
        wi: &mut Vec3,
        sample: &Point2,
        pdf: &mut Float,
        sampled_type: Option<BxDFType>,
    ) -> Spectrum {
        *wi = Vec3::new(-wo.x, -wo.y, wo.z);
        *pdf = 1.0;
        self.fresnel.evaluate(cos_theta(wi)) * self.reflection_factor / abs_cos_theta(wi)
    }

    fn pdf(&self, wo: &Vec3, wi: &Vec3) -> Float {
        0.0
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.bxdf_type & t) == self.bxdf_type
    }
}
