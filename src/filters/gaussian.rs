use crate::{
    base::{constants::Float, filter::Filter},
    geometries::{point2::Point2F, vec2::Vec2F},
};

pub struct GaussianFilter {
    radius: Vec2F,
    inverse_radius: Vec2F,
    alpha: Float,
    exp_x: Float,
    exp_y: Float,
}

impl GaussianFilter {
    pub fn new(radius: Vec2F, alpha: Float) -> Self {
        Self {
            radius,
            inverse_radius: Vec2F::new(1.0 / radius.x, 1.0 / radius.y),
            alpha,
            exp_x: (-alpha * radius.x * radius.x).exp(),
            exp_y: (-alpha * radius.y * radius.y).exp(),
        }
    }

    fn gaussian(&self, d: Float, exp_v: Float) -> Float {
        ((-self.alpha * d * d).exp() - exp_v).max(0.0)
    }
}

impl Filter for GaussianFilter {
    fn evaluate(&self, point: &Point2F) -> Float {
        self.gaussian(point.x, self.exp_x) * self.gaussian(point.y, self.exp_y)
    }

    fn radius(&self) -> Vec2F {
        self.radius
    }

    fn inverse_radius(&self) -> Vec2F {
        self.inverse_radius
    }
}
