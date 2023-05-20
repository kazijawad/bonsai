mod accelerators;
mod base;
mod bxdfs;
mod cameras;
mod filters;
mod geometries;
mod integrators;
mod interactions;
mod io;
mod lights;
mod materials;
mod primitives;
mod samplers;
mod shapes;
mod spectra;
mod textures;

pub use accelerators::bvh::BVH;
pub use base::{
    camera::Camera,
    constants::Float,
    film::{Film, FilmOptions},
    filter::Filter,
    integrator::{Integrator, SamplerIntegrator},
    light::{AreaLight, Light},
    material::Material,
    mipmap::MIPMap,
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
    bounds2::{Bounds2, Bounds2F, Bounds2I},
    bounds3::Bounds3,
    interval::Interval,
    mat4::Mat4,
    normal::Normal,
    point2::{Point2, Point2F, Point2I},
    point3::Point3,
    quaternion::Quaternion,
    ray::Ray,
    vec2::{Vec2, Vec2F, Vec2I},
    vec3::Vec3,
};
pub use integrators::whitted::WhittedIntegrator;
pub use io::{
    image::{Image, ImageWrapMode},
    obj::OBJ,
};
pub use lights::{
    diffuse::{DiffuseAreaLight, DiffuseAreaLightOptions},
    directional::{DirectionalLight, DirectionalLightOptions},
    infinite::{InfiniteAreaLight, InfiniteAreaLightOptions},
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
pub use textures::{
    constant::ConstantTexture,
    image::{ImageTexture, ImageTextureOptions},
    uv::UVTexture,
};
