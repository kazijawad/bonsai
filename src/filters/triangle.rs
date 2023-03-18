use crate::{
    base::{constants::Float, filter::Filter},
    geometries::{point2::Point2, vec2::Vec2},
};

pub struct TriangleFilter {
    radius: Vec2,
    inverse_radius: Vec2,
}

impl TriangleFilter {
    pub fn new(radius: Vec2) -> Self {
        Self {
            radius,
            inverse_radius: Vec2::new(1.0 / radius.x, 1.0 / radius.y),
        }
    }
}

impl Filter for TriangleFilter {
    fn evaluate(&self, point: &Point2) -> Float {
        (self.radius.x - point.x.abs()).max(0.0) * (self.radius.y - point.y.abs()).max(0.0)
    }

    fn radius(&self) -> Vec2 {
        self.radius
    }

    fn inverse_radius(&self) -> Vec2 {
        self.inverse_radius
    }
}
