use crate::{
    base::{
        camera::Camera,
        film::Film,
        transform::{AnimatedTransform, Transform},
    },
    geometries::{bounds2::Bounds2, vec3::Vec3},
    medium::Medium,
    utils::math::Float,
};

pub struct ProjectiveCamera<'a> {
    pub camera: Camera<'a>,
    pub camera_to_screen: Transform,
    pub raster_to_camera: Transform,
    pub screen_to_raster: Transform,
    pub raster_to_screen: Transform,
    pub lens_radius: Float,
    pub focal_distance: Float,
}

impl<'a> ProjectiveCamera<'a> {
    pub fn new(
        camera_to_world: &AnimatedTransform,
        camera_to_screen: &Transform,
        screen_window: &Bounds2,
        shutter_open: Float,
        shutter_close: Float,
        lens_radius: Float,
        focal_distance: Float,
        film: &'a Film,
        medium: &'a Medium,
    ) -> Self {
        // Compute projective camera screen transformations.
        let screen_to_raster =
            Transform::scale(film.full_resolution.x, film.full_resolution.y, 1.0)
                * Transform::scale(
                    1.0 / (screen_window.max.x - screen_window.min.x),
                    1.0 / (screen_window.min.y - screen_window.max.y),
                    1.0,
                )
                * Transform::translate(&Vec3::new(-screen_window.min.x, -screen_window.max.y, 0.0));
        let raster_to_screen = screen_to_raster.inverse();
        let raster_to_camera = &camera_to_screen.inverse() * &raster_to_screen;

        Self {
            camera: Camera::new(camera_to_world, shutter_open, shutter_close, film, medium),
            camera_to_screen: camera_to_screen.clone(),
            raster_to_camera,
            screen_to_raster,
            raster_to_screen,
            lens_radius,
            focal_distance,
        }
    }
}
