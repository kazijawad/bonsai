use crate::{
    base::{constants::Float, filter::Filter},
    geometries::{point2::Point2F, vec2::Vec2F},
};

pub struct TriangleFilter {
    radius: Vec2F,
    inverse_radius: Vec2F,
}

impl TriangleFilter {
    pub fn new(radius: Vec2F) -> Self {
        Self {
            radius,
            inverse_radius: Vec2F::new(1.0 / radius.x, 1.0 / radius.y),
        }
    }
}

impl Filter for TriangleFilter {
    fn evaluate(&self, point: &Point2F) -> Float {
        (self.radius.x - point.x.abs()).max(0.0) * (self.radius.y - point.y.abs()).max(0.0)
    }

    fn radius(&self) -> Vec2F {
        self.radius
    }

    fn inverse_radius(&self) -> Vec2F {
        self.inverse_radius
    }
}
