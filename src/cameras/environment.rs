use crate::{
    base::{
        camera::{Camera, CameraSample},
        film::Film,
        transform::AnimatedTransform,
    },
    geometries::{point3::Point3, ray::Ray, vec3::Vec3},
    utils::math::{lerp, Float, PI},
};

pub struct EnvironmentCamera<'a> {
    camera_to_world: &'a AnimatedTransform<'a>,
    shutter_open: Float,
    shutter_close: Float,
    film: &'a Film,
}

impl<'a> EnvironmentCamera<'a> {
    pub fn new(
        camera_to_world: &'a AnimatedTransform,
        shutter_open: Float,
        shutter_close: Float,
        film: &'a Film,
    ) -> Self {
        Self {
            camera_to_world,
            shutter_open,
            shutter_close,
            film,
        }
    }
}

impl<'a> Camera for EnvironmentCamera<'a> {
    fn generate_ray(&self, sample: &CameraSample, ray: &mut Ray) -> Float {
        // Compute ray direction.
        let theta = PI * sample.film_point.y / self.film.full_resolution.y;
        let phi = 2.0 * PI * sample.film_point.x / self.film.full_resolution.x;
        *ray = self.camera_to_world.transform_ray(&Ray::new(
            &Point3::default(),
            &Vec3::new(
                theta.sin() * phi.cos(),
                theta.cos(),
                theta.sin() * phi.sin(),
            ),
            Float::INFINITY,
            lerp(sample.time, self.shutter_open, self.shutter_close),
        ));

        1.0
    }

    fn get_film(&self) -> &Film {
        self.film
    }
}
