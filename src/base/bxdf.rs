use std::mem;

use crate::{
    base::{
        constants::{Float, PI},
        sampling::{cosine_sample_hemisphere, uniform_hemisphere_pdf, uniform_sample_hemisphere},
        spectrum::Spectrum,
    },
    geometries::{normal::Normal, point2::Point2F, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
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

pub fn cos_theta(w: &Vec3) -> Float {
    w.z
}

pub fn cos2_theta(w: &Vec3) -> Float {
    w.z * w.z
}

pub fn abs_cos_theta(w: &Vec3) -> Float {
    w.z.abs()
}

pub fn sin2_theta(w: &Vec3) -> Float {
    Float::max(0.0, 1.0 - cos2_theta(w))
}

pub fn sin_theta(w: &Vec3) -> Float {
    sin2_theta(w).sqrt()
}

pub fn tan_theta(w: &Vec3) -> Float {
    sin_theta(w) / cos_theta(w)
}

pub fn tan2_theta(w: &Vec3) -> Float {
    sin2_theta(w) / cos2_theta(w)
}

pub fn cos_phi(w: &Vec3) -> Float {
    let v = sin_theta(w);
    if v == 0.0 {
        1.0
    } else {
        (w.x / v).clamp(-1.0, 1.0)
    }
}

pub fn sin_phi(w: &Vec3) -> Float {
    let v = sin_theta(w);
    if v == 0.0 {
        0.0
    } else {
        (w.y / v).clamp(-1.0, 1.0)
    }
}

pub fn cos2_phi(w: &Vec3) -> Float {
    cos_phi(w) * cos_phi(w)
}

pub fn sin2_phi(w: &Vec3) -> Float {
    sin_phi(w) * sin_phi(w)
}

pub fn reflect(wo: &Vec3, n: &Vec3) -> Vec3 {
    -wo + 2.0 * wo.dot(n) * n
}

pub fn refract(wi: &Vec3, n: &Normal, eta: Float) -> Option<Vec3> {
    // Compute cos(theta) using Snell's law.
    let cos_theta_i = n.dot(&Normal::from(*wi));
    let sin2_theta_i = Float::max(0.0, 1.0 - cos_theta_i * cos_theta_i);
    let sin2_theta_t = eta * eta * sin2_theta_i;

    // Handle total internal reflection for transmission.
    if sin2_theta_t >= 1.0 {
        return None;
    }

    let cos_theta_t = (1.0 - sin2_theta_t).sqrt();
    Some(eta * -wi + (eta * cos_theta_i - cos_theta_t) * Vec3::from(*n))
}

pub fn same_hemisphere(w: &Vec3, wp: &Vec3) -> bool {
    w.z * wp.z > 0.0
}

pub fn fresnel_dielectric(cos_theta_i: Float, mut eta_i: Float, mut eta_t: Float) -> Float {
    let cos_theta_i = cos_theta_i.clamp(-1.0, 1.0);

    // Potentially swap indices of refraction.
    if !(cos_theta_i > 0.0) {
        mem::swap(&mut eta_i, &mut eta_t);
    }

    // Compute cos(theta) using Snell's law.
    let sin_theta_i = Float::max(0.0, 1.0 - cos_theta_i * cos_theta_i).sqrt();
    let sin_theta_t = eta_i / eta_t * sin_theta_i;

    // Handle total internal reflection.
    if sin_theta_t >= 1.0 {
        return 1.0;
    }

    let cos_theta_t = Float::max(0.0, 1.0 - sin_theta_t * sin_theta_t).sqrt();
    let par_reflect = ((eta_t * cos_theta_i) - (eta_i * cos_theta_t))
        / ((eta_t * cos_theta_i) + (eta_i * cos_theta_t));
    let perp_reflect = ((eta_i * cos_theta_i) - (eta_t * cos_theta_t))
        / ((eta_i * cos_theta_i) + (eta_t * cos_theta_t));
    (par_reflect * par_reflect + perp_reflect * perp_reflect) / 2.0
}

pub fn fresnel_conductor(
    cos_theta_i: Float,
    eta_i: &RGBSpectrum,
    eta_t: &RGBSpectrum,
    k: &RGBSpectrum,
) -> RGBSpectrum {
    let cos_theta_i = RGBSpectrum::new(cos_theta_i.clamp(-1.0, 1.0));
    let eta = eta_t / eta_i;
    let eta_k = k / eta_i;

    let cos_theta_i2 = cos_theta_i * cos_theta_i;
    let sin_theta_i2 = RGBSpectrum::new(1.0) - cos_theta_i2;
    let eta_2 = eta * eta;
    let eta_k2 = eta_k * eta_k;

    let t0 = eta_2 - eta_k2 - sin_theta_i2;
    let a2_b2 = (t0 * t0 + eta_2 * eta_k2 * 4.0).sqrt();
    let t1 = a2_b2 + cos_theta_i2;
    let a = ((a2_b2 + t0) * 0.5).sqrt();
    let t2 = cos_theta_i * a * 2.0;
    let rs = (t1 - t2) / (t1 + t2);

    let t3 = cos_theta_i2 * a2_b2 + sin_theta_i2 * sin_theta_i2;
    let t4 = t2 * sin_theta_i2;
    let rp = rs * (t3 - t4) / (t3 + t4);

    (rp + rs) * 0.5
}
