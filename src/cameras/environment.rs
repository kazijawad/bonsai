use crate::{
    base::{
        camera::{Camera, CameraSample, CameraSystem},
        film::Film,
        transform::AnimatedTransform,
    },
    geometries::{point3::Point3, ray::Ray, vec3::Vec3},
    medium::Medium,
    utils::math::{lerp, Float, PI},
};

pub struct EnvironmentCamera<'a> {
    camera: Camera<'a>,
}

impl<'a> EnvironmentCamera<'a> {
    pub fn new(
        camera_to_world: &AnimatedTransform,
        shutter_open: Float,
        shutter_close: Float,
        film: &'a Film,
        medium: &'a Medium,
    ) -> Self {
        Self {
            camera: Camera::new(camera_to_world, shutter_open, shutter_close, film, medium),
        }
    }
}

impl<'a> CameraSystem for EnvironmentCamera<'a> {
    fn generate_ray(&self, sample: &CameraSample, ray: &mut Ray) -> Float {
        // Compute ray direction.
        let theta = PI * sample.film_point.y / self.camera.film.full_resolution.y;
        let phi = 2.0 * PI * sample.film_point.x / self.camera.film.full_resolution.x;
        *ray = self.camera.camera_to_world.transform_ray(&Ray::new(
            &Point3::default(),
            &Vec3::new(
                theta.sin() * phi.cos(),
                theta.cos(),
                theta.sin() * phi.sin(),
            ),
            Float::INFINITY,
            lerp(
                sample.time,
                self.camera.shutter_open,
                self.camera.shutter_close,
            ),
            Some(self.camera.medium.to_owned()),
        ));

        1.0
    }
}
