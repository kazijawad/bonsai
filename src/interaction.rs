use crate::{
    geometries::{normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    math::Float,
    medium::{Medium, MediumInterface},
    shape::Shape,
};

pub trait Interaction: Send + Sync {
    fn is_surface_interaction(&self) -> bool {
        false
    }

    fn is_medium_interaction(&self) -> bool {
        false
    }

    fn spawn_ray(&self, d: &Vec3) -> Ray;
    fn spawn_ray_to_point(&self, p: Point3) -> Ray;
    fn spawn_ray_to_interaction(&self, it: Box<dyn Interaction>) -> Ray;

    fn get_medium(&self) -> Medium;
    fn get_medium_with_vec(&self, w: &Vec3) -> Medium;
}

struct Shading {
    normal: Normal,
    dpdu: Vec3,
    dpdv: Vec3,
    dndu: Normal,
    dndv: Normal,
}

pub struct SurfaceInteraction {
    point: Point3,
    point_error: Point3,
    normal: Normal,
    negative_direction: Vec3,
    time: Float,
    medium: MediumInterface,
    uv: Point2,
    dpdu: Vec3,
    dpdv: Vec3,
    dndu: Normal,
    dndv: Normal,
    shape: Box<dyn Shape>,
    shading: Shading,
    // primitive: Box<dyn Primitive>,
    // bsdf: BSDF,
    // bssrdf: BSSRDF,
    dpdx: Vec3,
    dpdy: Vec3,
    dudx: Float,
    dvdx: Float,
    dudy: Float,
    dvdy: Float,
    face_index: i32,
}
