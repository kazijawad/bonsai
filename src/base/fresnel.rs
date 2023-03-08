use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::{
    spectra::rgb::RGBSpectrum,
    utils::{
        bxdf::{fresnel_conductor, fresnel_dielectric},
        math::Float,
    },
};

pub trait Fresnel: Debug + Send + Sync + DynClone {
    fn evaluate(&self, cos_theta_i: Float) -> RGBSpectrum;
}

dyn_clone::clone_trait_object!(Fresnel);

#[derive(Debug, Clone)]
pub struct FresnelConductor {
    eta_i: RGBSpectrum,
    eta_t: RGBSpectrum,
    k: RGBSpectrum,
}

#[derive(Debug, Clone)]
pub struct FresnelDielectric {
    eta_i: Float,
    eta_t: Float,
}

#[derive(Debug, Clone)]
pub struct FresnelNoOp;

impl FresnelConductor {
    pub fn new(eta_i: &RGBSpectrum, eta_t: &RGBSpectrum, k: &RGBSpectrum) -> Self {
        Self {
            eta_i: eta_i.clone(),
            eta_t: eta_t.clone(),
            k: k.clone(),
        }
    }
}

impl FresnelDielectric {
    pub fn new(eta_i: Float, eta_t: Float) -> Self {
        Self { eta_i, eta_t }
    }
}

impl Fresnel for FresnelConductor {
    fn evaluate(&self, cos_theta_i: Float) -> RGBSpectrum {
        fresnel_conductor(cos_theta_i.abs(), &self.eta_i, &self.eta_t, &self.k)
    }
}

impl Fresnel for FresnelDielectric {
    fn evaluate(&self, cos_theta_i: Float) -> RGBSpectrum {
        RGBSpectrum::new(fresnel_dielectric(cos_theta_i, self.eta_i, self.eta_t))
    }
}

impl Fresnel for FresnelNoOp {
    fn evaluate(&self, cos_theta_i: Float) -> RGBSpectrum {
        RGBSpectrum::new(1.0)
    }
}
