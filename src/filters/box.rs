use crate::{
    base::{constants::Float, filter::Filter},
    geometries::{point2::Point2F, vec2::Vec2F},
};

pub struct BoxFilter {
    radius: Vec2F,
    inverse_radius: Vec2F,
}

impl BoxFilter {
    pub fn new(radius: Vec2F) -> Self {
        Self {
            radius,
            inverse_radius: Vec2F::new(1.0 / radius.x, 1.0 / radius.y),
        }
    }
}

impl Filter for BoxFilter {
    fn evaluate(&self, _: &Point2F) -> Float {
        1.0
    }

    fn radius(&self) -> Vec2F {
        self.radius
    }

    fn inverse_radius(&self) -> Vec2F {
        self.inverse_radius
    }
}
