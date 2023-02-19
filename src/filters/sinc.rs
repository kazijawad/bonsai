use crate::{
    base::filter::Filter,
    geometries::{point2::Point2, vec2::Vec2},
    utils::math::{Float, PI},
};

pub struct LanczosSincFilter {
    radius: Vec2,
    inverse_radius: Vec2,
    tau: Float,
}

pub struct LanczosSincFilterDescriptior {
    pub x_width: Option<Float>,
    pub y_width: Option<Float>,
    pub tau: Option<Float>,
}

impl LanczosSincFilter {
    pub fn create(options: LanczosSincFilterDescriptior) -> Self {
        let x_width = options.x_width.unwrap_or(4.0);
        let y_width = options.y_width.unwrap_or(4.0);
        let tau = options.tau.unwrap_or(3.0);
        Self::new(Vec2::new(x_width, y_width), tau)
    }

    pub fn new(radius: Vec2, tau: Float) -> Self {
        Self {
            radius,
            inverse_radius: Vec2::new(1.0 / radius.x, 1.0 / radius.y),
            tau,
        }
    }

    fn sinc(&self, x: Float) -> Float {
        let x = x.abs();
        if x < 1e-5 {
            1.0
        } else {
            (PI * x).sin() / (PI * x)
        }
    }

    fn windowed_sinc(&self, x: Float, radius: Float) -> Float {
        let x = x.abs();
        if x > radius {
            0.0
        } else {
            let lanczos = self.sinc(x / self.tau);
            self.sinc(x) * lanczos
        }
    }
}

impl Filter for LanczosSincFilter {
    fn evaluate(&self, point: &Point2) -> Float {
        self.windowed_sinc(point.x, self.radius.x) * self.windowed_sinc(point.y, self.radius.y)
    }

    fn radius(&self) -> &Vec2 {
        &self.radius
    }

    fn inverse_radius(&self) -> &Vec2 {
        &self.inverse_radius
    }
}
