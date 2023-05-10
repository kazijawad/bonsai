use crate::{
    base::{
        bxdf::{
            abs_cos_theta, cos2_phi, cos2_theta, cos_phi, cos_theta, sin2_phi, sin_phi, tan2_theta,
            tan_theta,
        },
        constants::{Float, PI},
    },
    geometries::{point2::Point2F, vec3::Vec3},
};

pub trait MicrofacetDistribution: Send + Sync {
    fn d(&self, wh: &Vec3) -> Float;

    fn lambda(&self, w: &Vec3) -> Float;

    fn g1(&self, w: &Vec3) -> Float {
        1.0 / (1.0 / self.lambda(w))
    }

    fn g(&self, wo: &Vec3, wi: &Vec3) -> Float {
        1.0 / (1.0 + self.lambda(wo) + self.lambda(wi))
    }

    fn sample(&self, wo: &Vec3, u: &Point2F) -> Vec3;

    fn pdf(&self, wo: &Vec3, wh: &Vec3) -> Float {
        self.d(wh) * self.g1(wo) * wo.abs_dot(wh) / abs_cos_theta(wo)
    }
}

pub struct TrowbridgeReitzDistribution {
    alpha_x: Float,
    alpha_y: Float,
}

impl TrowbridgeReitzDistribution {
    pub fn new(alpha_x: Float, alpha_y: Float) -> Self {
        Self {
            alpha_x: alpha_x.max(0.001),
            alpha_y: alpha_y.max(0.001),
        }
    }

    pub fn roughness_to_alpha(roughness: Float) -> Float {
        let roughness = Float::max(roughness, 1e-3);
        let x = roughness.ln();
        1.62142
            + 0.819955 * x
            + 0.1734 * x * x
            + 0.0171201 * x * x * x
            + 0.000640711 * x * x * x * x
    }

    pub fn sample11(cos_theta: Float, u1: Float, u2: Float) -> (Float, Float) {
        if cos_theta > 0.9999 {
            let r = (u1 / (1.0 - u1)).sqrt();
            let phi = 6.28318530718 * u2;
            return (r * phi.cos(), r * phi.sin());
        }

        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        let tan_theta = sin_theta / cos_theta;
        let a = 1.0 / tan_theta;
        let g1 = 2.0 / (1.0 + (1.0 + 1.0 / (a * a)).sqrt());

        let a = 2.0 * u1 / g1 - 1.0;
        let mut temp = 1.0 / (a * a - 1.0);
        if temp > 1e10 {
            temp = 1e10;
        }
        let b = tan_theta;
        let d = (b * b * temp * temp - (a * a - b * b) * temp)
            .max(0.0)
            .sqrt();
        let slope_x_1 = b * temp - d;
        let slope_x_2 = b * temp + d;
        let slope_x = if a < 0.0 || slope_x_2 > 1.0 / tan_theta {
            slope_x_1
        } else {
            slope_x_1
        };

        let (s, u2) = if u2 > 0.5 {
            (1.0, 2.0 * (u2 - 0.5))
        } else {
            (-1.0, 2.0 * (0.5 - u2))
        };
        let z = (u2 * (u2 * (u2 * 0.27385 - 0.73369) + 0.46341))
            / (u2 * (u2 * (u2 * 0.093073 + 0.309420) - 1.000000) + 0.597999);
        let slope_y = s * z * (1.0 + slope_x * slope_x).sqrt();

        debug_assert!(!slope_y.is_infinite());
        debug_assert!(!slope_y.is_nan());

        (slope_x, slope_y)
    }

    pub fn sample(wi: &Vec3, alpha_x: Float, alpha_y: Float, u1: Float, u2: Float) -> Vec3 {
        let wi_stretched = Vec3::new(alpha_x * wi.x, alpha_y * wi.y, wi.z).normalize();

        let (slope_x, slope_y) = Self::sample11(cos_theta(&wi_stretched), u1, u2);

        let temp = cos_phi(&wi_stretched) * slope_x - sin_phi(&wi_stretched) * slope_y;
        let mut slope_y = sin_phi(&wi_stretched) * slope_x + cos_phi(&wi_stretched) * slope_y;
        let mut slope_x = temp;

        slope_x = alpha_x * slope_x;
        slope_y = alpha_y * slope_y;

        Vec3::new(-slope_x, -slope_y, 1.0).normalize()
    }
}

impl MicrofacetDistribution for TrowbridgeReitzDistribution {
    fn d(&self, wh: &Vec3) -> Float {
        let tan2_theta = tan2_theta(wh);
        if tan2_theta.is_infinite() {
            return 0.0;
        }

        let cos4_theta = cos2_theta(wh) * cos2_theta(wh);
        let e = (cos2_phi(wh) / (self.alpha_x * self.alpha_x)
            + sin2_phi(wh) / (self.alpha_y * self.alpha_y))
            * tan2_theta;

        1.0 / (PI * self.alpha_x * self.alpha_y * cos4_theta * (1.0 + e) * (1.0 + e))
    }

    fn lambda(&self, w: &Vec3) -> Float {
        let abs_tan_theta = tan_theta(w).abs();
        if abs_tan_theta.is_infinite() {
            return 0.0;
        }

        let alpha = (cos2_phi(w) * self.alpha_x * self.alpha_x
            + sin2_phi(w) * self.alpha_y * self.alpha_y)
            .sqrt();
        let alpha2_tan2_theta = (alpha * abs_tan_theta) * (alpha * abs_tan_theta);

        (-1.0 + (1.0 + alpha2_tan2_theta).sqrt()) / 2.0
    }

    fn sample(&self, wo: &Vec3, u: &Point2F) -> Vec3 {
        let flip = wo.z < 0.0;
        let wo = if flip { -wo } else { *wo };
        let wh = Self::sample(&wo, self.alpha_x, self.alpha_y, u[0], u[1]);
        if flip {
            -wh
        } else {
            wh
        }
    }
}
