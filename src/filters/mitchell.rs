use crate::{
    base::{constants::Float, filter::Filter},
    geometries::{point2::Point2, vec2::Vec2},
};

pub struct MitchellFilter {
    radius: Vec2,
    inverse_radius: Vec2,
    b: Float,
    c: Float,
}

impl MitchellFilter {
    pub fn new(radius: Vec2, b: Float, c: Float) -> Self {
        Self {
            radius,
            inverse_radius: Vec2::new(1.0 / radius.x, 1.0 / radius.y),
            b,
            c,
        }
    }

    fn mitchell(&self, x: Float) -> Float {
        let x = (2.0 * x).abs();
        if x > 1.0 {
            ((-self.b - 6.0 * self.c) * x * x * x
                + (6.0 * self.b + 30.0 * self.c) * x * x
                + (-12.0 * self.b - 48.0 * self.c) * x
                + (8.0 * self.b + 24.0 * self.c))
                * (1.0 / 6.0)
        } else {
            ((12.0 - 9.0 * self.b - 6.0 * self.c) * x * x * x
                + (-18.0 + 12.0 * self.b + 6.0 * self.c) * x * x
                + (6.0 - 2.0 * self.b))
                * (1.0 / 6.0)
        }
    }
}

impl Filter for MitchellFilter {
    fn evaluate(&self, point: &Point2) -> Float {
        self.mitchell(point.x * self.inverse_radius.x)
            * self.mitchell(point.y * self.inverse_radius.y)
    }

    fn radius(&self) -> Vec2 {
        self.radius
    }

    fn inverse_radius(&self) -> Vec2 {
        self.inverse_radius
    }
}
