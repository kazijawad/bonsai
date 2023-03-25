use crate::{
    base::constants::Float,
    geometries::{normal::Normal, point3::Point3, ray::Ray, vec3::Vec3},
};

pub trait Interaction: Send + Sync {
    fn position(&self) -> Point3;
    fn position_error(&self) -> Vec3;
    fn normal(&self) -> Normal;
    fn time(&self) -> Float;

    fn spawn_ray(&self, direction: &Vec3) -> Ray;
    fn spawn_ray_to_point(&self, point: Point3) -> Ray;
    fn spawn_ray_to_it(&self, interaction: &dyn Interaction) -> Ray;
}
