use crate::{
    base::constants::Float,
    geometries::{point2::Point2F, vec2::Vec2F},
};

pub trait Filter: Send + Sync {
    fn evaluate(&self, point: &Point2F) -> Float;

    fn radius(&self) -> Vec2F;
    fn inverse_radius(&self) -> Vec2F;
}
