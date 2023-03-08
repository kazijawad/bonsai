use crate::{
    base::{
        camera::{Camera, CameraSample},
        film::Film,
        transform::AnimatedTransform,
    },
    geometries::{point3::Point3, ray::Ray, vec3::Vec3},
    utils::math::{lerp, Float, PI},
};

pub struct EnvironmentCamera {
    camera_to_world: AnimatedTransform,
    shutter_open: Float,
    shutter_close: Float,
    film: Film,
}

impl<'a> EnvironmentCamera {
    pub fn new(
        camera_to_world: AnimatedTransform,
        shutter_open: Float,
        shutter_close: Float,
        film: Film,
    ) -> Self {
        Self {
            camera_to_world,
            shutter_open,
            shutter_close,
            film,
        }
    }
}

impl Camera for EnvironmentCamera {
    fn generate_ray(&self, sample: &CameraSample, ray: &mut Ray) -> Float {
        // Compute ray direction.
        let theta = PI * sample.film_point.y / self.film.full_resolution.y;
        let phi = 2.0 * PI * sample.film_point.x / self.film.full_resolution.x;

        *ray = Ray::new(
            &Point3::default(),
            &Vec3::new(
                theta.sin() * phi.cos(),
                theta.cos(),
                theta.sin() * phi.sin(),
            ),
            Float::INFINITY,
            lerp(sample.time, self.shutter_open, self.shutter_close),
        )
        .animated_transform(&self.camera_to_world);

        1.0
    }

    fn film(&self) -> &Film {
        &self.film
    }
}
