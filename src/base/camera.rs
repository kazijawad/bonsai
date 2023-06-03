use crate::{
    base::{constants::Float, film::Film},
    geometries::{point2::Point2F, ray::Ray},
};

#[derive(Clone, Copy)]
pub struct CameraSample {
    pub film: Point2F,
    pub lens: Point2F,
    pub time: Float,
}

pub trait Camera: Send + Sync {
    fn generate_ray(&self, sample: &CameraSample, ray: &mut Ray) -> Float;

    fn film(&self) -> &Film;
}
