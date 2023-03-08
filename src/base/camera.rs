use crate::{
    base::film::Film,
    geometries::{point2::Point2, ray::Ray},
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

    fn generate_ray_differential(&self, sample: &CameraSample, ray: &mut Ray) -> Float {
        let weight = self.generate_ray(sample, ray);
        if weight == 0.0 {
            return 0.0;
        }

        // Find camera ray after shifting a fraction of a pixel in the x direction.
        let mut weight_x = 0.0;
        for epsilon in [0.05, -0.5] {
            let mut shift = sample.clone();
            shift.film_point.x += epsilon;

            let mut ray_x = Ray::default();
            weight_x = self.generate_ray(sample, &mut ray_x);

            ray.rx_origin = ray.origin + (ray_x.origin - ray.origin) / epsilon;
            ray.rx_direction = ray.direction + (ray_x.direction - ray.direction) / epsilon;
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

            ray.ry_origin = ray.origin + (ray_y.origin - ray.origin) / epsilon;
            ray.ry_direction = ray.direction + (ray_y.direction - ray.direction) / epsilon;

            if weight_y != 0.0 {
                break;
            }
        }
        if weight_y == 0.0 {
            return 0.0;
        }

        ray.has_differentials = true;
        weight
    }

    fn film(&self) -> &Film;
}
