use crate::{
    base::{
        camera::{Camera, CameraSample},
        film::Film,
        transform::{AnimatedTransform, Transform},
    },
    geometries::{bounds2::Bounds2, point3::Point3, ray::Ray, vec3::Vec3},
    utils::math::{lerp, Float},
};

pub struct OrthographicCamera {
    camera_to_world: AnimatedTransform,
    camera_to_screen: Transform,
    screen_to_raster: Transform,
    raster_to_screen: Transform,
    raster_to_camera: Transform,
    shutter_open: Float,
    shutter_close: Float,
    lens_radius: Float,
    focal_distance: Float,
    dx_camera: Vec3,
    dy_camera: Vec3,
    film: Film,
}

impl OrthographicCamera {
    pub fn new(
        camera_to_world: AnimatedTransform,
        screen_window: &Bounds2,
        shutter_open: Float,
        shutter_close: Float,
        lens_radius: Float,
        focal_distance: Float,
        film: Film,
    ) -> Self {
        let camera_to_screen = Transform::orthographic(0.0, 1.0);

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

        // Compute differential changes in origin for orthographic camera rays.
        let dx_camera = raster_to_camera.transform_vec(&Vec3::new(1.0, 0.0, 0.0));
        let dy_camera = raster_to_camera.transform_vec(&Vec3::new(0.0, 1.0, 0.0));

        Self {
            camera_to_world,
            camera_to_screen,
            screen_to_raster,
            raster_to_screen,
            raster_to_camera,
            shutter_open,
            shutter_close,
            lens_radius,
            focal_distance,
            dx_camera,
            dy_camera,
            film,
        }
    }
}

impl Camera for OrthographicCamera {
    fn generate_ray(&self, sample: &CameraSample, ray: &mut Ray) -> Float {
        // Compute raster and camera sample positions.
        let film_point = Point3::new(sample.film_point.x, sample.film_point.y, 0.0);
        let camera_point = film_point.transform(&self.raster_to_camera);
        *ray = Ray::new(
            &camera_point,
            &Vec3::new(0.0, 0.0, 1.0),
            Float::INFINITY,
            0.0,
        );

        // Modify ray for depth of field.
        if self.lens_radius > 0.0 {
            // Sample point on lens.
            let lens_point = self.lens_radius * sample.lens_point.concentric_disk_sample();

            // Compute point on plane of focus.
            let focus_t = self.focal_distance / ray.direction.z;
            let focus_point = ray.at(focus_t);

            // Update ray for effect of lens.
            ray.origin = Point3::new(lens_point.x, lens_point.y, 0.0);
            ray.direction = (focus_point - ray.origin).normalize();
        }

        ray.time = lerp(sample.time, self.shutter_open, self.shutter_close);
        *ray = ray.animated_transform(&self.camera_to_world);

        1.0
    }

    fn generate_ray_differential(&self, sample: &CameraSample, ray: &mut Ray) -> Float {
        // Compute raster and camera sample positions.
        let film_point = Point3::new(sample.film_point.x, sample.film_point.y, 0.0);
        let camera_point = film_point.transform(&self.raster_to_camera);
        *ray = Ray::new(
            &camera_point,
            &Vec3::new(0.0, 0.0, 1.0),
            Float::INFINITY,
            0.0,
        );

        // Modify ray for depth of field.
        if self.lens_radius > 0.0 {
            // Sample point on lens.
            let lens_point = self.lens_radius * sample.lens_point.concentric_disk_sample();

            // Compute point on plane of focus.
            let focus_t = self.focal_distance / ray.direction.z;
            let focus_point = ray.at(focus_t);

            // Update ray for effect of lens.
            ray.origin = Point3::new(lens_point.x, lens_point.y, 0.0);
            ray.direction = (focus_point - ray.origin).normalize();
        }

        // Compute ray differentials.
        if self.lens_radius > 0.0 {
            // Sample point on lens.
            let lens_point = self.lens_radius * sample.lens_point.concentric_disk_sample();
            let focus_t = self.focal_distance / ray.direction.z;

            let focus_point = camera_point + self.dx_camera + (focus_t * Vec3::new(0.0, 0.0, 1.0));
            ray.rx_origin = Point3::new(lens_point.x, lens_point.y, 0.0);
            ray.rx_direction = (focus_point - ray.rx_origin).normalize();

            let focus_point = camera_point + self.dy_camera + (focus_t * Vec3::new(0.0, 0.0, 1.0));
            ray.ry_origin = Point3::new(lens_point.x, lens_point.y, 0.0);
            ray.ry_direction = (focus_point - ray.ry_origin).normalize();
        } else {
            ray.rx_origin = ray.origin + self.dx_camera;
            ray.ry_origin = ray.origin + self.dy_camera;
            ray.rx_direction = ray.direction;
            ray.ry_direction = ray.direction;
        }

        ray.time = lerp(sample.time, self.shutter_open, self.shutter_close);
        *ray = ray.animated_transform(&self.camera_to_world);
        ray.has_differentials = true;

        1.0
    }

    fn film(&self) -> &Film {
        &self.film
    }
}
