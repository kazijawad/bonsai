use crate::{
    base::{
        camera::{Camera, CameraSample},
        constants::Float,
        film::Film,
        math::lerp,
        sampling::concentric_sample_disk,
        transform::{AnimatedTransform, Transform},
    },
    geometries::{bounds2::Bounds2F, point3::Point3, ray::Ray, vec3::Vec3},
};

pub struct PerspectiveCamera {
    camera_to_world: AnimatedTransform,
    raster_to_camera: Transform,
    shutter_open: Float,
    shutter_close: Float,
    lens_radius: Float,
    focal_distance: Float,
    dx_camera: Vec3,
    dy_camera: Vec3,
    film: Film,
}

pub struct PerspectiveCameraOptions {
    pub animated_transform: AnimatedTransform,
    pub shutter_open: Float,
    pub shutter_close: Float,
    pub lens_radius: Float,
    pub focal_distance: Float,
    pub fov: Float,
    pub near: Float,
    pub far: Float,
    pub film: Film,
}

impl PerspectiveCamera {
    pub fn new(opts: PerspectiveCameraOptions) -> Self {
        let film = opts.film;

        let camera_to_world = opts.animated_transform;
        let camera_to_screen = Transform::perspective(opts.fov, opts.near, opts.far);

        let mut screen_window = Bounds2F::default();
        let frame = film.full_resolution.x / film.full_resolution.y;
        if frame > 1.0 {
            screen_window.min.x = -frame;
            screen_window.max.x = frame;
            screen_window.min.y = -1.0;
            screen_window.max.y = 1.0;
        } else {
            screen_window.min.x = -1.0;
            screen_window.max.x = 1.0;
            screen_window.min.y = -1.0 / frame;
            screen_window.max.y = 1.0 / frame;
        }

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

        // Compute differential changes in origin for perspective camera rays.
        let origin = Point3::default().transform(&raster_to_camera);
        let dx_camera = Point3::new(1.0, 0.0, 0.0).transform(&raster_to_camera) - origin;
        let dy_camera = Point3::new(0.0, 1.0, 0.0).transform(&raster_to_camera) - origin;

        Self {
            camera_to_world,
            raster_to_camera,
            shutter_open: opts.shutter_open,
            shutter_close: opts.shutter_close,
            lens_radius: opts.lens_radius,
            focal_distance: opts.focal_distance,
            dx_camera,
            dy_camera,
            film,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn generate_ray(&self, sample: &CameraSample, ray: &mut Ray) -> Float {
        // Compute raster and camera sample positions.
        let film = Point3::new(sample.film.x, sample.film.y, 0.0);
        let camera = film.transform(&self.raster_to_camera);
        *ray = Ray::new(
            &Point3::default(),
            &Vec3::from(camera).normalize(),
            Float::INFINITY,
            0.0,
        );

        // Modify ray for depth of field.
        if self.lens_radius > 0.0 {
            // Sample point on lens.
            let lens = self.lens_radius * concentric_sample_disk(&sample.lens);

            // Compute point on plane of focus.
            let focus_point = ray.at(self.focal_distance / ray.direction.z);

            // Update ray for effect of lens.
            ray.origin = Point3::new(lens.x, lens.y, 0.0);
            ray.direction = (focus_point - ray.origin).normalize();
        }

        // Compute offset rays for ray differentials.
        if self.lens_radius > 0.0 {
            // Sample point on lens.
            let lens = self.lens_radius * concentric_sample_disk(&sample.lens);

            let dx = Vec3::from(camera + self.dx_camera).normalize();
            let focus = Point3::default() + ((self.focal_distance / dx.z) * dx);
            ray.rx_origin = Point3::new(lens.x, lens.y, 0.0);
            ray.rx_direction = (focus - ray.rx_origin).normalize();

            let dy = Vec3::from(camera + self.dy_camera).normalize();
            let focus = Point3::default() + ((self.focal_distance / dy.z) * dy);
            ray.ry_origin = Point3::new(lens.x, lens.y, 0.0);
            ray.ry_direction = (focus - ray.ry_origin).normalize();
        } else {
            ray.rx_origin = ray.origin;
            ray.ry_origin = ray.origin;
            ray.rx_direction = (Vec3::from(camera) + self.dx_camera).normalize();
            ray.ry_direction = (Vec3::from(camera) + self.dy_camera).normalize();
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
