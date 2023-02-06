use crate::{
    geometries::{
        point2::Point2,
        ray::{Ray, RayDifferential},
    },
    utils::math::Float,
};

#[derive(Debug, Clone, Copy)]
pub struct CameraSample {
    pub film_point: Point2,
    pub lens_point: Point2,
    pub time: Float,
}

pub trait Camera: Send + Sync {
    fn generate_ray(&self, sample: &CameraSample, ray: &mut Ray) -> Float;

    fn generate_ray_differential(
        &self,
        sample: &CameraSample,
        diff: &mut RayDifferential,
    ) -> Float {
        let weight = self.generate_ray(sample, &mut diff.ray);

        // Find camera ray after shifting a fraction of a pixel in the x direction.
        let mut weight_x = 0.0;
        for epsilon in [0.05, -0.5] {
            let mut shift = sample.clone();
            shift.film_point.x += epsilon;

            let mut ray_x = Ray::default();
            weight_x = self.generate_ray(sample, &mut ray_x);

            diff.rx_origin = diff.ray.origin + (ray_x.origin - diff.ray.origin) / epsilon;
            diff.rx_direction =
                diff.ray.direction + (ray_x.direction - diff.ray.direction) / epsilon;

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

            diff.ry_origin = diff.ray.origin + (ray_y.origin - diff.ray.origin) / epsilon;
            diff.ry_direction =
                diff.ray.direction + (ray_y.direction - diff.ray.direction) / epsilon;

            if weight_y != 0.0 {
                break;
            }
        }
        if weight_y == 0.0 {
            return 0.0;
        }

        diff.has_differentials = true;
        weight
    }
}
