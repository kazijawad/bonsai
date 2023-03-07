use crate::{
    base::filter::Filter,
    geometries::{point2::Point2, vec2::Vec2},
    utils::math::Float,
};

pub struct TriangleFilter {
    radius: Vec2,
    inverse_radius: Vec2,
}

pub struct TriangleFilterDescriptior {
    pub x_width: Option<Float>,
    pub y_width: Option<Float>,
}

impl TriangleFilter {
    pub fn create(desc: TriangleFilterDescriptior) -> Self {
        let x_width = desc.x_width.unwrap_or(2.0);
        let y_width = desc.y_width.unwrap_or(2.0);
        Self::new(Vec2::new(x_width, y_width))
    }

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
