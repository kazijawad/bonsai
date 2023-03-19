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
    constants::Float,
    film::Film,
    filter::Filter,
    integrator::Integrator,
    light::Light,
    material::Material,
    sampler::Sampler,
    scene::Scene,
    texture::{
        CylindricalMapping2D, IdentityMapping3D, PlanarMapping2D, SphericalMapping2D, Texture,
        TextureMapping2D, TextureMapping3D, UVMapping2D,
    },
    transform::{AnimatedTransform, Transform},
};
pub use cameras::perspective::PerspectiveCamera;
pub use filters::{
    gaussian::GaussianFilter, mitchell::MitchellFilter, r#box::BoxFilter, sinc::LanczosSincFilter,
    triangle::TriangleFilter,
};
pub use geometries::{
    bounds2::Bounds2, bounds3::Bounds3, interval::Interval, mat4::Mat4, normal::Normal,
    point2::Point2, point3::Point3, quaternion::Quaternion, ray::Ray, vec2::Vec2, vec3::Vec3,
};
pub use integrators::whitted::WhittedIntegrator;
pub use lights::{point::PointLight, spot::SpotLight};
pub use materials::{matte::MatteMaterial, plastic::PlasticMaterial};
pub use primitives::{geometric::GeometricPrimitive, transformed::TransformedPrimitive};
pub use samplers::stratified::StratifiedSampler;
pub use shapes::{cone::Cone, cylinder::Cylinder, disk::Disk, sphere::Sphere};
pub use spectra::rgb::RGBSpectrum;
pub use textures::{constant::ConstantTexture, image::ImageTexture, uv::UVTexture};
