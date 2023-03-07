use crate::{
    base::filter::Filter,
    geometries::{point2::Point2, vec2::Vec2},
    utils::math::Float,
};

pub struct GaussianFilter {
    radius: Vec2,
    inverse_radius: Vec2,
    alpha: Float,
    exp_x: Float,
    exp_y: Float,
}

pub struct GaussianFilterDescriptior {
    pub x_width: Float,
    pub y_width: Float,
    pub alpha: Float,
}

impl GaussianFilter {
    pub fn create(desc: &GaussianFilterDescriptior) -> Self {
        Self::new(Vec2::new(desc.x_width, desc.y_width), desc.alpha)
    }

    pub fn new(radius: Vec2, alpha: Float) -> Self {
        Self {
            radius,
            inverse_radius: Vec2::new(1.0 / radius.x, 1.0 / radius.y),
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
    fn evaluate(&self, point: &Point2) -> Float {
        self.gaussian(point.x, self.exp_x) * self.gaussian(point.y, self.exp_y)
    }

    fn radius(&self) -> Vec2 {
        self.radius
    }

    fn inverse_radius(&self) -> Vec2 {
        self.inverse_radius
    }
}

impl Default for GaussianFilterDescriptior {
    fn default() -> Self {
        Self {
            x_width: 2.0,
            y_width: 2.0,
            alpha: 2.0,
        }
    }
}
