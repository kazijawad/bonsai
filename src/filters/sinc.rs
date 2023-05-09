use crate::{
    base::{
        constants::{Float, PI},
        filter::Filter,
    },
    geometries::{point2::Point2F, vec2::Vec2F},
};

pub struct LanczosSincFilter {
    radius: Vec2F,
    inverse_radius: Vec2F,
    tau: Float,
}

impl LanczosSincFilter {
    pub fn new(radius: Vec2F, tau: Float) -> Self {
        Self {
            radius,
            inverse_radius: Vec2F::new(1.0 / radius.x, 1.0 / radius.y),
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
    fn evaluate(&self, point: &Point2F) -> Float {
        self.windowed_sinc(point.x, self.radius.x) * self.windowed_sinc(point.y, self.radius.y)
    }

    fn radius(&self) -> Vec2F {
        self.radius
    }

    fn inverse_radius(&self) -> Vec2F {
        self.inverse_radius
    }
}
