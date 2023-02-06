// Temporarily disable while in development.
#![allow(unused_variables)]
#![allow(dead_code)]

mod accelerators;
mod base;
mod cameras;
mod geometries;
mod interactions;
mod primitives;
mod renderer;
mod shapes;
mod spectra;
mod texture;
mod utils;

pub use accelerators::bvh::BVH;
pub use base::{
    aggregate::Aggregate,
    camera::Camera,
    film::Film,
    material::TestMaterial,
    spectrum::{CoefficientSpectrum, Spectrum},
    transform::{AnimatedTransform, Transform},
};
pub use cameras::{
    environment::EnvironmentCamera, orthographic::OrthographicCamera,
    perspective::PerspectiveCamera,
};
pub use geometries::{
    bounds2::Bounds2, bounds3::Bounds3, interval::Interval, mat4::Mat4, normal::Normal,
    point2::Point2, point3::Point3, quaternion::Quaternion, ray::Ray, vec2::Vec2, vec3::Vec3,
};
pub use primitives::{geometric::GeometricPrimitive, transformed::TransformedPrimitive};
pub use renderer::Renderer;
pub use shapes::{
    cone::Cone, curve::Curve, cylinder::Cylinder, disk::Disk, hyperboloid::Hyperboloid,
    paraboloid::Paraboloid, sphere::Sphere, triangle::Triangle,
};
pub use utils::math::*;
