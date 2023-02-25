use crate::{
    geometries::{point2::Point2, vec3::Vec3},
    utils::{
        bxdf::{abs_cos_theta, cos2_phi, cos2_theta, sin2_phi, tan2_theta, tan_theta},
        math::{Float, PI},
    },
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

    fn sample_wh(&self, wo: &Vec3, u: &Point2) -> Vec3;

    fn pdf(&self, wo: &Vec3, wh: &Vec3) -> Float;

    fn roughness_to_alpha(&self, roughness: Float) -> Float;
}

#[derive(Debug)]
pub struct BeckmannDistribution {
    sample_visible_area: bool,
    alpha_x: Float,
    alpha_y: Float,
}

#[derive(Debug)]
pub struct TrowbridgeReitzDistribution {
    sample_visible_area: bool,
    alpha_x: Float,
    alpha_y: Float,
}

impl BeckmannDistribution {
    pub fn new(alpha_x: Float, alpha_y: Float, sample_visible_area: bool) -> Self {
        Self {
            sample_visible_area,
            alpha_x: alpha_x.max(0.001),
            alpha_y: alpha_y.max(0.001),
        }
    }
}

impl TrowbridgeReitzDistribution {
    pub fn new(alpha_x: Float, alpha_y: Float, sample_visible_area: bool) -> Self {
        Self {
            sample_visible_area,
            alpha_x: alpha_x.max(0.001),
            alpha_y: alpha_y.max(0.001),
        }
    }
}

impl MicrofacetDistribution for BeckmannDistribution {
    fn d(&self, wh: &Vec3) -> Float {
        let tan2_theta = tan2_theta(wh);
        if tan2_theta.is_infinite() {
            return 0.0;
        }

        let cos4_theta = cos2_theta(wh) * cos2_theta(wh);

        (-tan2_theta
            * (cos2_phi(wh) / (self.alpha_x * self.alpha_x)
                + sin2_phi(wh) / (self.alpha_y * self.alpha_y)))
            .exp()
            / (PI * self.alpha_x * self.alpha_y * cos4_theta)
    }

    fn lambda(&self, w: &Vec3) -> Float {
        let abs_tan_theta = tan_theta(w).abs();
        if abs_tan_theta.is_infinite() {
            return 0.0;
        }

        let alpha = (cos2_phi(w) * self.alpha_x * self.alpha_x
            + sin2_phi(w) * self.alpha_y * self.alpha_y)
            .sqrt();
        let a = 1.0 / (alpha * abs_tan_theta);
        if a >= 1.6 {
            return 0.0;
        }

        (1.0 - 1.259 * a + 0.396 * a * a) / (3.535 * a + 2.181 * a * a)
    }

    fn sample_wh(&self, wo: &Vec3, u: &Point2) -> Vec3 {
        todo!()
    }

    fn pdf(&self, wo: &Vec3, wh: &Vec3) -> Float {
        if self.sample_visible_area {
            self.d(wh) * self.g1(wo) * wo.abs_dot(wh) / abs_cos_theta(wo)
        } else {
            self.d(wh) * abs_cos_theta(wh)
        }
    }

    fn roughness_to_alpha(&self, roughness: Float) -> Float {
        let roughness = Float::max(roughness, 1e-3);
        let x = roughness.ln();
        1.62142
            + 0.819955 * x
            + 0.1734 * x * x
            + 0.0171201 * x * x * x
            + 0.000640711 * x * x * x * x
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

    fn sample_wh(&self, wo: &Vec3, u: &Point2) -> Vec3 {
        todo!()
    }

    fn pdf(&self, wo: &Vec3, wh: &Vec3) -> Float {
        if self.sample_visible_area {
            self.d(wh) * self.g1(wo) * wo.abs_dot(wh) / abs_cos_theta(wo)
        } else {
            self.d(wh) * abs_cos_theta(wh)
        }
    }

    fn roughness_to_alpha(&self, roughness: Float) -> Float {
        let roughness = Float::max(roughness, 1e-3);
        let x = roughness.ln();
        1.62142
            + 0.819955 * x
            + 0.1734 * x * x
            + 0.0171201 * x * x * x
            + 0.000640711 * x * x * x * x
    }
}
