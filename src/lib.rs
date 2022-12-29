// Temporarily disable while in development.
#![allow(unused_variables)]
#![allow(dead_code)]

mod accelerators;
mod base;
mod bssrdf;
mod camera;
mod film;
mod geometries;
mod interactions;
mod light;
mod medium;
mod primitives;
mod reflection;
mod renderer;
mod shapes;
mod spectra;
mod texture;
mod transform;
mod utils;

pub use accelerators::bvh::BVH;
pub use base::{aggregate::Aggregate, material::TestMaterial};
pub use camera::Camera;
pub use film::Film;
pub use geometries::{
    bounds2::Bounds2, bounds3::Bounds3, interval::Interval, mat4::Mat4, normal::Normal,
    point2::Point2, point3::Point3, quaternion::Quaternion, ray::Ray, vec2::Vec2, vec3::Vec3,
};
pub use medium::*;
pub use primitives::{geometric::GeometricPrimitive, transformed::TransformedPrimitive};
pub use renderer::Renderer;
pub use shapes::{
    cone::Cone, curve::Curve, cylinder::Cylinder, disk::Disk, hyperboloid::Hyperboloid,
    paraboloid::Paraboloid, sphere::Sphere, triangle::Triangle,
};
pub use transform::Transform;
pub use utils::{api::*, math::*, parser};
