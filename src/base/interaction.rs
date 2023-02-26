use crate::geometries::{point3::Point3, ray::Ray, vec3::Vec3};

pub trait Interaction: Send + Sync {
    fn is_surface_interaction(&self) -> bool;
    fn is_medium_interaction(&self) -> bool;

    fn spawn_ray(&self, d: &Vec3) -> Ray;
    fn spawn_ray_to_point(&self, p: Point3) -> Ray;
    fn spawn_ray_to_it(&self, it: &dyn Interaction) -> Ray;
}
