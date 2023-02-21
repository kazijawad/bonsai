use std::mem;

use crate::{
    base::spectrum::{CoefficientSpectrum, Spectrum},
    geometries::{normal::Normal, vec3::Vec3},
    utils::math::Float,
};

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

pub fn cosd_phi(wa: &Vec3, wb: &Vec3) -> Float {
    let wa_xy = wa.x * wa.x + wa.y * wa.y;
    let wb_xy = wb.x * wb.x + wb.y * wb.y;
    if wa_xy == 0.0 || wb_xy == 0.0 {
        1.0
    } else {
        ((wa.x * wb.x + wa.y * wb.y) / (wa_xy * wb_xy).sqrt()).clamp(-1.0, 1.0)
    }
}

pub fn reflect(wo: &Vec3, n: &Vec3) -> Vec3 {
    -wo + 2.0 * wo.dot(n) * n
}

pub fn refract(wi: &Vec3, n: &Normal, eta: Float, wt: &mut Vec3) -> bool {
    // Compute cos(theta) using Snell's law.
    let cos_theta_i = n.dot(&Normal::from(*wi));
    let sin2_theta_i = Float::max(0.0, 1.0 - cos_theta_i * cos_theta_i);
    let sin2_theta_t = eta * eta * sin2_theta_i;

    // Handle total internal reflection for transmission.
    if sin2_theta_t >= 1.0 {
        return false;
    }

    let cos_theta_t = (1.0 - sin2_theta_t).sqrt();
    *wt = eta * -wi + (eta * cos_theta_i - cos_theta_t) * Vec3::from(*n);

    true
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
    eta_i: &Spectrum,
    eta_t: &Spectrum,
    k: &Spectrum,
) -> Spectrum {
    let cos_theta_i = Spectrum::new(cos_theta_i.clamp(-1.0, 1.0));
    let eta = eta_t / eta_i;
    let eta_k = k / eta_i;

    let cos_theta_i2 = cos_theta_i * cos_theta_i;
    let sin_theta_i2 = Spectrum::new(1.0) - cos_theta_i2;
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
