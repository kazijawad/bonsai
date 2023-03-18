use crate::{
    base::{constants::Float, filter::Filter},
    geometries::{point2::Point2, vec2::Vec2},
};

pub struct BoxFilter {
    radius: Vec2,
    inverse_radius: Vec2,
}

impl BoxFilter {
    pub fn new(radius: Vec2) -> Self {
        Self {
            radius,
            inverse_radius: Vec2::new(1.0 / radius.x, 1.0 / radius.y),
        }
    }
}

impl Filter for BoxFilter {
    fn evaluate(&self, point: &Point2) -> Float {
        1.0
    }

    fn radius(&self) -> Vec2 {
        self.radius
    }

    fn inverse_radius(&self) -> Vec2 {
        self.inverse_radius
    }
}
