use crate::{
    base::constants::Float,
    geometries::{point2::Point2, vec2::Vec2},
};

pub trait Filter: Send + Sync {
    fn evaluate(&self, point: &Point2) -> Float;

    fn radius(&self) -> Vec2;
    fn inverse_radius(&self) -> Vec2;
}
