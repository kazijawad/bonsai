use crate::{
    geometries::{point2::Point2, vec3::Vec3},
    utils::math::{Float, PI},
};

pub fn uniform_sample_hemisphere(u: &Point2) -> Vec3 {
    let z = u[0];
    let r = Float::max(0.0, 1.0 - z * z).sqrt();
    let phi = 2.0 * PI * u[1];
    Vec3::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn uniform_hemisphere_pdf() -> Float {
    (1.0 / PI) / 2.0
}
