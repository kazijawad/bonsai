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
    film::{Film, FilmOptions},
    filter::Filter,
    integrator::Integrator,
    light::{AreaLight, Light},
    material::Material,
    primitive::Primitive,
    sampler::Sampler,
    scene::Scene,
    texture::{
        CylindricalMapping2D, IdentityMapping3D, PlanarMapping2D, SphericalMapping2D, Texture,
        TextureMapping2D, TextureMapping3D, UVMapping2D,
    },
    transform::{AnimatedTransform, Transform},
};
pub use cameras::perspective::{PerspectiveCamera, PerspectiveCameraOptions};
pub use filters::{
    gaussian::GaussianFilter, mitchell::MitchellFilter, r#box::BoxFilter, sinc::LanczosSincFilter,
    triangle::TriangleFilter,
};
pub use geometries::{
    bounds2::Bounds2, bounds3::Bounds3, interval::Interval, mat4::Mat4, normal::Normal,
    point2::Point2, point3::Point3, quaternion::Quaternion, ray::Ray, vec2::Vec2, vec3::Vec3,
};
pub use integrators::whitted::WhittedIntegrator;
pub use lights::{
    diffuse::{DiffuseAreaLight, DiffuseAreaLightOptions},
    directional::{DirectionalLight, DirectionalLightOptions},
    point::{PointLight, PointLightOptions},
    spot::{SpotLight, SpotLightOptions},
};
pub use materials::{matte::MatteMaterial, plastic::PlasticMaterial};
pub use primitives::{geometric::GeometricPrimitive, transformed::TransformedPrimitive};
pub use samplers::stratified::{StratifiedSampler, StratifiedSamplerOptions};
pub use shapes::{
    cone::{Cone, ConeOptions},
    cylinder::{Cylinder, CylinderOptions},
    disk::{Disk, DiskOptions},
    sphere::{Sphere, SphereOptions},
    triangle::{Triangle, TriangleMesh, TriangleMeshOptions, TriangleOptions},
};
pub use spectra::rgb::RGBSpectrum;
pub use textures::{constant::ConstantTexture, uv::UVTexture};
