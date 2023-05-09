use rand::prelude::*;

use crate::{
    base::constants::{Float, PI},
    geometries::{point2::Point2F, vec2::Vec2, vec3::Vec3},
};

pub fn shuffle<T>(sample: &mut [T], count: usize, num_dims: usize, rng: &mut StdRng) {
    for i in 0..count {
        let other = i + rng.gen_range(0..(count - i));
        for j in 0..num_dims {
            sample.swap(num_dims * i + j, num_dims * other + j);
        }
    }
}

pub fn stratified_sample_1d(
    samples: &mut [Float],
    num_samples: usize,
    rng: &mut StdRng,
    jitter: bool,
) {
    let inverse_num_samples = 1.0 / num_samples as Float;
    for i in 0..num_samples {
        let delta = if jitter { rng.gen_range(0.0..1.0) } else { 0.5 };
        samples[i] = ((i as Float + delta) * inverse_num_samples).min(1.0 - Float::EPSILON);
    }
}

pub fn stratified_sample_2d(
    samples: &mut [Point2F],
    nx: usize,
    ny: usize,
    rng: &mut StdRng,
    jitter: bool,
) {
    let dx = 1.0 / nx as Float;
    let dy = 1.0 / ny as Float;
    let mut i = 0;
    for y in 0..ny {
        for x in 0..nx {
            let jx = if jitter { rng.gen_range(0.0..1.0) } else { 0.5 };
            let jy = if jitter { rng.gen_range(0.0..1.0) } else { 0.5 };
            samples[i].x = Float::min((x as Float + jx) * dx, 1.0 - Float::EPSILON);
            samples[i].y = Float::min((y as Float + jy) * dy, 1.0 - Float::EPSILON);
            i += 1;
        }
    }
}

pub fn latin_hypercube(
    samples: &mut [Point2F],
    num_samples: usize,
    num_dims: usize,
    rng: &mut StdRng,
) {
    // Generate LHS samples along diagonal.
    let inverse_num_samples = 1.0 / num_samples as Float;
    for i in 0..num_samples {
        for j in 0..num_dims {
            let sj = (i as Float + rng.gen_range(0.0..1.0)) * inverse_num_samples;
            samples[num_dims * i + j].x = sj.min(1.0 - Float::EPSILON);
        }
    }

    // Permute LHS samples in each dimension.
    for i in 0..num_dims {
        for j in 0..num_samples {
            let other = j + rng.gen_range(0..(num_samples - j));
            samples.swap(num_dims * j + i, num_dims * other + i);
        }
    }
}

pub fn concentric_sample_disk(u: &Point2F) -> Point2F {
    // Map uniform values to [-1, 1].
    let offset = 2.0 * u - Vec2::new(1.0, 1.0);

    // Handle degeneracy at the origin.
    if offset.x == 0.0 && offset.y == 0.0 {
        return Point2F::default();
    }

    // Apply concentric mapping to point.
    let (theta, radius) = if offset.x.abs() > offset.y.abs() {
        ((PI / 4.0) * (offset.y / offset.x), offset.x)
    } else {
        ((PI / 2.0) - (PI / 4.0) * (offset.x / offset.y), offset.y)
    };

    radius * Point2F::new(theta.cos(), theta.sin())
}

pub fn uniform_sample_hemisphere(u: &Point2F) -> Vec3 {
    let z = u[0];
    let r = (1.0 - z * z).max(0.0).sqrt();
    let phi = 2.0 * PI * u[1];
    Vec3::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn uniform_hemisphere_pdf() -> Float {
    1.0 / (2.0 * PI)
}

pub fn uniform_sample_sphere(u: &Point2F) -> Vec3 {
    let z = 1.0 - 2.0 * u[0];
    let r = (1.0 - z * z).max(0.0).sqrt();
    let phi = 2.0 * PI * u[1];
    Vec3::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn uniform_sphere_pdf() -> Float {
    1.0 / (4.0 * PI)
}

pub fn uniform_sample_cone(u: &Point2F, cos_theta_max: Float) -> Vec3 {
    let cos_theta = (1.0 - u[0]) + u[0] * cos_theta_max;
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    let phi = u[1] * 2.0 * PI;
    Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
}

pub fn uniform_cone_pdf(cos_theta_max: Float) -> Float {
    1.0 / (2.0 * PI * (1.0 - cos_theta_max))
}

pub fn uniform_sample_triangle(u: &Point2F) -> Point2F {
    let sqrt0 = u[0].sqrt();
    Point2F::new(1.0 - sqrt0, u[1] * sqrt0)
}

pub fn cosine_sample_hemisphere(u: &Point2F) -> Vec3 {
    let d = concentric_sample_disk(u);
    let z = (1.0 - d.x * d.x - d.y * d.y).max(0.0).sqrt();
    Vec3::new(d.x, d.y, z)
}

pub fn cosine_hemisphere_pdf(cos_theta: Float) -> Float {
    cos_theta * (1.0 / PI)
}
