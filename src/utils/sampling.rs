use rand::prelude::*;

use crate::{
    base::constants::{Float, PI},
    geometries::{point2::Point2, vec2::Vec2, vec3::Vec3},
};

pub fn shuffle<T>(sample: &mut [T], count: usize, num_dims: usize, rng: &mut StdRng) {
    for i in 0..count {
        let other = i + rng.gen_range(0..(count - i));
        for j in 0..num_dims {
            sample.swap(num_dims * i + j, num_dims * other + j);
        }
    }
}

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

pub fn cosine_hemisphere_pdf(cos_theta: Float) -> Float {
    cos_theta * (1.0 / PI)
}

pub fn concentric_sample_disk(u: &Point2) -> Point2 {
    // Map uniform values to [-1, 1].
    let u_offset = 2.0 * u - Vec2::new(1.0, 1.0);

    // Handle degeneracy at the origin.
    if u_offset.x == 0.0 && u_offset.y == 0.0 {
        return Point2::default();
    }

    // Apply concentric mapping to point.
    let (theta, radius) = if u_offset.x.abs() > u_offset.y.abs() {
        ((PI / 4.0) * (u_offset.y / u_offset.x), u_offset.x)
    } else {
        (
            (PI / 2.0) - (PI / 4.0) * (u_offset.x / u_offset.y),
            u_offset.y,
        )
    };

    radius * Point2::new(theta.cos(), theta.sin())
}

pub fn uniform_cone_pdf(cos_theta_max: Float) -> Float {
    1.0 / (2.0 * PI * (1.0 - cos_theta_max))
}

pub fn uniform_sample_cone(u: &Point2, cos_theta_max: Float) -> Vec3 {
    let cos_theta = (1.0 - u.x) + u.x * cos_theta_max;
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    let phi = u.y * 2.0 * PI;
    Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
}

pub fn uniform_hemisphere_pdf() -> Float {
    (1.0 / PI) / 2.0
}

pub fn uniform_sample_sphere(u: &Point2) -> Vec3 {
    let z = 1.0 - 2.0 * u.x;
    let r = Float::max(0.0, 1.0 - z * z).sqrt();
    let phi = 2.0 * PI * u.y;
    Vec3::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn uniform_sphere_pdf() -> Float {
    1.0 / (4.0 * PI)
}
