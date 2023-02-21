use crate::{
    geometries::{point2::Point2, vec2::Vec2, vec3::Vec3},
    utils::math::{Float, PI},
};

pub fn uniform_sample_hemisphere(u: &Point2) -> Vec3 {
    let z = u[0];
    let r = Float::max(0.0, 1.0 - z * z).sqrt();
    let phi = 2.0 * PI * u[1];
    Vec3::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn cosine_sample_hemisphere(u: &Point2) -> Vec3 {
    let d = concentric_sample_disk(u);
    let z = Float::max(0.0, 1.0 - d.x * d.x - d.y * d.y).sqrt();
    Vec3::new(d.x, d.y, z)
}

pub fn concentric_sample_disk(u: &Point2) -> Point2 {
    // Map uniform values to [-1, 1].
    let u_offset = 2.0 * u - Vec2::new(1.0, 1.0);

    // Handle degeneracy at the origin.
    if u_offset.x == 0.0 && u_offset.y == 0.0 {
        return Point2::default();
    }

    // Apply concentric mapping to point.
    let (theta, r) = if u_offset.x.abs() > u_offset.y.abs() {
        ((PI / 4.0) * (u_offset.y / u_offset.x), u_offset.x)
    } else {
        (
            (PI / 2.0) - (PI / 4.0) * (u_offset.x / u_offset.y),
            u_offset.y,
        )
    };

    r * Point2::new(theta.cos(), theta.sin())
}

pub fn uniform_hemisphere_pdf() -> Float {
    (1.0 / PI) / 2.0
}
