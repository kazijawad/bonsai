use crate::{
    base::filter::Filter,
    geometries::{point2::Point2, vec2::Vec2},
    utils::math::Float,
};

pub struct BoxFilter {
    radius: Vec2,
    inverse_radius: Vec2,
}

pub struct BoxFilterDescriptior {
    pub x_width: Option<Float>,
    pub y_width: Option<Float>,
}

impl BoxFilter {
    pub fn create(options: BoxFilterDescriptior) -> Self {
        let x_width = options.x_width.unwrap_or(0.5);
        let y_width = options.y_width.unwrap_or(0.5);
        Self::new(Vec2::new(x_width, y_width))
    }

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
