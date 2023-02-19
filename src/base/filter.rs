use crate::{
    geometries::{point2::Point2, vec2::Vec2},
    utils::math::Float,
};

pub trait Filter: Send + Sync {
    fn evaluate(&self, point: &Point2) -> Float;

    fn radius(&self) -> &Vec2;
    fn inverse_radius(&self) -> &Vec2;
}
