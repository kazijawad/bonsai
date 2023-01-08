use crate::{
    geometries::{
        point2::Point2,
        ray::{Ray, RayDifferential},
    },
    utils::math::Float,
};

#[derive(Debug, Clone, Copy)]
pub struct CameraSample {
    film_point: Point2,
    lens_point: Point2,
    time: Float,
}

pub trait Camera: Send + Sync {
    fn generate_ray(&self, sample: &CameraSample, ray: &mut Ray) -> Float;

    fn generate_ray_differential(&self, sample: &CameraSample, r: &mut RayDifferential) -> Float {
        let weight = self.generate_ray(sample, &mut r.ray);

        // Find camera ray after shifting a fraction of a pixel in the x direction.
        let mut weight_x = 0.0;
        for epsilon in [0.05, -0.5] {
            let mut shift = sample.clone();
            shift.film_point.x += epsilon;

            let mut ray_x = Ray::default();
            weight_x = self.generate_ray(sample, &mut ray_x);

            r.rx_origin = r.ray.origin + (ray_x.origin - r.ray.origin) / epsilon;
            r.rx_direction = r.ray.direction + (ray_x.direction - r.ray.direction) / epsilon;

            if weight_x != 0.0 {
                break;
            }
        }
        if weight_x == 0.0 {
            return 0.0;
        }

        // Find camera ray after shifting a fraction of a pixel in the y direction.
        let mut weight_y = 0.0;
        for epsilon in [0.05, -0.5] {
            let mut shift = sample.clone();
            shift.film_point.y += epsilon;

            let mut ray_y = Ray::default();
            weight_y = self.generate_ray(sample, &mut ray_y);

            r.ry_origin = r.ray.origin + (ray_y.origin - r.ray.origin) / epsilon;
            r.ry_direction = r.ray.direction + (ray_y.direction - r.ray.direction) / epsilon;

            if weight_y != 0.0 {
                break;
            }
        }
        if weight_y == 0.0 {
            return 0.0;
        }

        r.has_differentials = true;
        weight
    }
}
