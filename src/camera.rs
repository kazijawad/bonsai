use crate::{
    geometries::{point3::Point3, ray::Ray, vec3::Vec3},
    math::Float,
};

pub struct Camera {
    pub position: Vec3,
    pub look_at: Vec3,
    pub fov: Float,
    pub aspect_ratio: Float,
    pub aperature: Float,
    pub focus_distance: Float,
    pub start_time: Float,
    pub end_time: Float,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    lens_radius: Float,
}

impl Camera {
    pub fn new(
        position: Vec3,
        look_at: Vec3,
        fov: Float,
        aspect_ratio: Float,
        aperature: Float,
        focus_distance: Float,
        start_time: Float,
        end_time: Float,
    ) -> Self {
        let up = Vec3::new(0.0, 1.0, 0.0);

        let theta = fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_width = 2.0 * h;
        let viewport_height = aspect_ratio * viewport_width;

        let w = (position - look_at).normalize();
        let u = up.cross(&w).normalize();
        let v = w.cross(&u);

        let horizontal = focus_distance * viewport_width * u;
        let vertical = focus_distance * viewport_height * v;
        let lower_left_corner = position - horizontal / 2.0 - vertical / 2.0 - focus_distance * w;

        let lens_radius = aperature / 2.0;

        Self {
            position,
            look_at,
            fov,
            aspect_ratio,
            aperature,
            focus_distance,
            start_time,
            end_time,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            lens_radius,
        }
    }

    pub fn at(&self, s: Float, t: Float) -> Ray {
        let direction = self.lens_radius * Vec3::default();
        let offset = self.u * direction.x + self.v * direction.y;

        Ray::new(
            &Point3::from(self.position + offset),
            &(self.lower_left_corner + s * self.horizontal + t * self.vertical
                - self.position
                - offset),
            Float::INFINITY,
            0.0,
            None,
        )
    }
}