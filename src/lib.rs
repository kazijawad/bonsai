// Temporarily disable while in development.
#![allow(unused_variables)]
#![allow(dead_code)]

mod accelerators;
mod base;
mod bxdfs;
mod cameras;
mod filters;
mod geometries;
mod integrators;
mod interactions;
mod lights;
mod materials;
mod primitives;
mod samplers;
mod shapes;
mod spectra;
mod textures;
mod utils;

pub use accelerators::bvh::BVH;
pub use base::{
    camera::Camera,
    film::{Film, FilmDescriptor},
    filter::Filter,
    integrator::Integrator,
    material::Material,
    sampler::Sampler,
    scene::Scene,
    spectrum::{CoefficientSpectrum, Spectrum},
    texture::{
        CylindricalMapping2D, IdentityMapping3D, PlanarMapping2D, SphericalMapping2D, Texture,
        TextureMapping2D, TextureMapping3D, UVMapping2D,
    },
    transform::{AnimatedTransform, Transform},
};
pub use cameras::{
    environment::EnvironmentCamera, orthographic::OrthographicCamera,
    perspective::PerspectiveCamera,
};
pub use filters::{
    gaussian::{GaussianFilter, GaussianFilterDescriptior},
    mitchell::{MitchellFilter, MitchellFilterDescriptior},
    r#box::{BoxFilter, BoxFilterDescriptior},
    sinc::{LanczosSincFilter, LanczosSincFilterDescriptior},
    triangle::{TriangleFilter, TriangleFilterDescriptior},
};
pub use geometries::{
    bounds2::Bounds2, bounds3::Bounds3, interval::Interval, mat4::Mat4, normal::Normal,
    point2::Point2, point3::Point3, quaternion::Quaternion, ray::Ray, vec2::Vec2, vec3::Vec3,
};
pub use integrators::sampler::SamplerIntegrator;
pub use materials::{matte::MatteMaterial, plastic::PlasticMaterial};
pub use primitives::{geometric::GeometricPrimitive, transformed::TransformedPrimitive};
pub use samplers::stratified::StratifiedSampler;
pub use shapes::{
    cone::Cone, cylinder::Cylinder, disk::Disk, hyperboloid::Hyperboloid, paraboloid::Paraboloid,
    sphere::Sphere, triangle::Triangle,
};
pub use textures::{
    checkerboard::Checkerboard2DTexture, constant::ConstantTexture, dots::DotsTexture,
    image::ImageTexture, uv::UVTexture,
};
pub use utils::math::*;
