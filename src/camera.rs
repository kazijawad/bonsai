use rand::{rngs::StdRng, Rng};

use crate::{math::vec3::Vec3, ray::Ray};

#[derive(Debug, Clone, Copy)]
pub struct CameraSettings {
    pub position: Vec3,
    pub look_at: Vec3,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub aperature: f32,
    pub focus_distance: f32,
    pub start_time: f32,
    pub end_time: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub settings: CameraSettings,

    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    lens_radius: f32,
}

impl Camera {
    pub fn new(settings: &CameraSettings) -> Self {
        let up = Vec3::new(0.0, 1.0, 0.0);

        let theta = settings.fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_width = 2.0 * h;
        let viewport_height = settings.aspect_ratio * viewport_width;

        let w = Vec3::normalize(&(settings.position - settings.look_at));
        let u = Vec3::normalize(&Vec3::cross(&up, &w));
        let v = Vec3::cross(&w, &u);

        let horizontal = settings.focus_distance * viewport_width * u;
        let vertical = settings.focus_distance * viewport_height * v;
        let lower_left_corner =
            settings.position - horizontal / 2.0 - vertical / 2.0 - settings.focus_distance * w;

        let lens_radius = settings.aperature / 2.0;

        Self {
            settings: settings.clone(),

            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            lens_radius,
        }
    }

    pub fn get_ray(&self, s: f32, t: f32, rng: &mut StdRng) -> Ray {
        let direction = self.lens_radius * Vec3::random_in_unit_disk(rng);
        let offset = self.u * direction.x + self.v * direction.y;

        Ray::new(
            &(self.settings.position + offset),
            &(self.lower_left_corner + s * self.horizontal + t * self.vertical
                - self.settings.position
                - offset),
            rng.gen_range(self.settings.start_time..self.settings.end_time),
        )
    }
}
