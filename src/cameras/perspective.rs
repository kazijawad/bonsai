use crate::{
    base::{
        camera::{CameraSample, CameraSystem},
        film::Film,
        transform::{AnimatedTransform, Transform},
    },
    cameras::projective::ProjectiveCamera,
    geometries::{
        bounds2::Bounds2,
        point3::Point3,
        ray::{Ray, RayDifferential},
        vec3::Vec3,
    },
    medium::Medium,
    utils::math::{lerp, Float},
};

pub struct PerspectiveCamera<'a> {
    projective_camera: ProjectiveCamera<'a>,
    dx_camera: Vec3,
    dy_camera: Vec3,
}

impl<'a> PerspectiveCamera<'a> {
    pub fn new(
        camera_to_world: &AnimatedTransform,
        screen_window: &Bounds2,
        shutter_open: Float,
        shutter_close: Float,
        lens_radius: Float,
        focal_distance: Float,
        fov: Float,
        film: &'a Film,
        medium: &'a Medium,
    ) -> Self {
        let projective_camera = ProjectiveCamera::new(
            camera_to_world,
            &Transform::orthographic(0.0, 1.0),
            screen_window,
            shutter_open,
            shutter_close,
            lens_radius,
            focal_distance,
            film,
            medium,
        );

        // Compute differential changes in origin for perspective camera rays.
        let dx_camera = projective_camera
            .raster_to_camera
            .transform_point(&Point3::new(1.0, 0.0, 0.0))
            - projective_camera
                .raster_to_camera
                .transform_point(&Point3::default());
        let dy_camera = projective_camera
            .raster_to_camera
            .transform_point(&Point3::new(0.0, 1.0, 0.0))
            - projective_camera
                .raster_to_camera
                .transform_point(&Point3::default());

        // Compute image plane bounds at z = 1.
        // let resolution = film.full_resolution;
        // let mut point_min = projective_camera.raster_to_camera.transform_point(&Point3::default());
        // let mut point_max = projective_camera.raster_to_camera.transform_point(&Point3::new(resolution.x, resolution.y, 0.0));
        // point_min /= point_min.z;
        // point_max /= point_max.z;

        Self {
            projective_camera: ProjectiveCamera::new(
                camera_to_world,
                &Transform::perspective(fov, 1e-2, 1000.0),
                screen_window,
                shutter_open,
                shutter_close,
                lens_radius,
                focal_distance,
                film,
                medium,
            ),
            dx_camera,
            dy_camera,
        }
    }
}

impl<'a> CameraSystem for PerspectiveCamera<'a> {
    fn generate_ray(&self, sample: &CameraSample, ray: &mut Ray) -> Float {
        // Compute raster and camera sample positions.
        let film_point = Point3::new(sample.film_point.x, sample.film_point.y, 0.0);
        let camera_point = self
            .projective_camera
            .raster_to_camera
            .transform_point(&film_point);
        *ray = Ray::new(
            &Point3::default(),
            &Vec3::from(camera_point).normalize(),
            Float::INFINITY,
            0.0,
            None,
        );

        // Modify ray for depth of field.
        if self.projective_camera.lens_radius > 0.0 {
            // Sample point on lens.
            let lens_point =
                self.projective_camera.lens_radius * sample.lens_point.concentric_disk_sample();

            // Compute point on plane of focus.
            let focus_t = self.projective_camera.focal_distance / ray.direction.z;
            let focus_point = ray.at(focus_t);

            // Update ray for effect of lens.
            ray.origin = Point3::new(lens_point.x, lens_point.y, 0.0);
            ray.direction = (focus_point - ray.origin).normalize();
        }

        ray.time = lerp(
            sample.time,
            self.projective_camera.camera.shutter_open,
            self.projective_camera.camera.shutter_close,
        );
        ray.medium = Some(self.projective_camera.camera.medium.to_owned());
        *ray = self
            .projective_camera
            .camera
            .camera_to_world
            .transform_ray(&ray);

        1.0
    }

    fn generate_ray_differential(&self, sample: &CameraSample, r: &mut RayDifferential) -> Float {
        // Compute raster and camera sample positions.
        let film_point = Point3::new(sample.film_point.x, sample.film_point.y, 0.0);
        let camera_point = self
            .projective_camera
            .raster_to_camera
            .transform_point(&film_point);
        *r = RayDifferential::new(
            &Point3::default(),
            &Vec3::from(camera_point).normalize(),
            Float::INFINITY,
            0.0,
            None,
        );

        // Modify ray for depth of field.
        if self.projective_camera.lens_radius > 0.0 {
            // Sample point on lens.
            let lens_point =
                self.projective_camera.lens_radius * sample.lens_point.concentric_disk_sample();

            // Compute point on plane of focus.
            let focus_t = self.projective_camera.focal_distance / r.ray.direction.z;
            let focus_point = r.ray.at(focus_t);

            // Update ray for effect of lens.
            r.ray.origin = Point3::new(lens_point.x, lens_point.y, 0.0);
            r.ray.direction = (focus_point - r.ray.origin).normalize();
        }

        // Compute offset rays for ray differentials.
        if self.projective_camera.lens_radius > 0.0 {
            // Sample point on lens.
            let lens_point =
                self.projective_camera.lens_radius * sample.lens_point.concentric_disk_sample();

            let dx = Vec3::from(camera_point + self.dx_camera).normalize();
            let focus_t = self.projective_camera.focal_distance / dx.z;
            let focus_point = Point3::default() + (focus_t * dx);
            r.rx_origin = Point3::new(lens_point.x, lens_point.y, 0.0);
            r.rx_direction = (focus_point - r.rx_origin).normalize();

            let dy = Vec3::from(camera_point + self.dy_camera).normalize();
            let focus_t = self.projective_camera.focal_distance / dy.z;
            let focus_point = Point3::default() + (focus_t * dy);
            r.ry_origin = Point3::new(lens_point.x, lens_point.y, 0.0);
            r.ry_direction = (focus_point - r.ry_origin).normalize();
        } else {
            r.rx_origin = r.ray.origin;
            r.ry_origin = r.ray.origin;
            r.rx_direction = (Vec3::from(camera_point) + self.dx_camera).normalize();
            r.ry_direction = (Vec3::from(camera_point) + self.dy_camera).normalize();
        }

        r.ray.time = lerp(
            sample.time,
            self.projective_camera.camera.shutter_open,
            self.projective_camera.camera.shutter_close,
        );
        r.ray.medium = Some(self.projective_camera.camera.medium.to_owned());
        r.ray = self
            .projective_camera
            .camera
            .camera_to_world
            .transform_ray(&r.ray);
        r.has_differentials = true;

        1.0
    }
}
