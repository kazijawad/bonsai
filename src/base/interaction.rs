use std::sync::Arc;

use crate::{
    geometries::{point3::Point3, ray::Ray, vec3::Vec3},
    medium::Medium,
};

pub trait Interaction: Send + Sync {
    fn is_surface_interaction(&self) -> bool;
    fn is_medium_interaction(&self) -> bool;

    fn spawn_ray(&self, d: &Vec3) -> Ray;
    fn spawn_ray_to_point(&self, p: Point3) -> Ray;
    fn spawn_ray_to_interaction(&self, it: Arc<dyn Interaction>) -> Ray;

    fn get_medium(&self) -> Medium;
    fn get_medium_with_vec(&self, w: &Vec3) -> Medium;
}
