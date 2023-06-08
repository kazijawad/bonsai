use crate::{
    base::{
        camera::{Camera, CameraLensSample, CameraRaySample},
        constants::{Float, PI},
        film::Film,
        interaction::Interaction,
        light::VisibilityTester,
        math::lerp,
        sampling::concentric_sample_disk,
        transform::{AnimatedTransform, Transform},
    },
    geometries::{
        bounds2::Bounds2F,
        normal::Normal,
        point2::Point2F,
        point3::Point3,
        ray::{Ray, RayDifferentials},
        vec3::Vec3,
    },
    spectra::rgb::RGBSpectrum,
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
    area: Float,
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
        let resolution = film.full_resolution;

        let shutter_open = opts.shutter_open;
        let shutter_close = opts.shutter_close;

        let lens_radius = opts.lens_radius;
        let focal_distance = opts.focal_distance;

        let camera_to_world = opts.animated_transform;
        let camera_to_screen = Transform::perspective(opts.fov, opts.near, opts.far);

        let mut screen_window = Bounds2F::default();
        let frame = resolution.x / resolution.y;
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
        let screen_to_raster = Transform::scale(resolution.x, resolution.y, 1.0)
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

        // Compute image plane bounds at z=1.
        let mut image_min = Point3::default().transform(&raster_to_camera);
        let mut image_max =
            Point3::new(resolution.x, resolution.y, 0.0).transform(&raster_to_camera);
        image_min /= image_min.z;
        image_max /= image_max.z;

        let area = ((image_max.x - image_min.x) * (image_max.y - image_min.y)).abs();

        Self {
            camera_to_world,
            raster_to_camera,
            shutter_open,
            shutter_close,
            lens_radius,
            focal_distance,
            dx_camera,
            dy_camera,
            area,
            film,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn generate_ray(&self, sample: &CameraRaySample, ray: &mut Ray) -> Float {
        // Compute raster and camera sample positions.
        let film_point = Point3::new(sample.film.x, sample.film.y, 0.0);
        let camera_point = Vec3::from(film_point.transform(&self.raster_to_camera));
        *ray = Ray::new(
            &Point3::default(),
            &camera_point.normalize(),
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

            let dx = (camera_point + self.dx_camera).normalize();
            let focus = Point3::default() + ((self.focal_distance / dx.z) * dx);
            let rx_origin = Point3::new(lens.x, lens.y, 0.0);
            let rx_direction = (focus - rx_origin).normalize();

            let dy = (camera_point + self.dy_camera).normalize();
            let focus = Point3::default() + ((self.focal_distance / dy.z) * dy);
            let ry_origin = Point3::new(lens.x, lens.y, 0.0);
            let ry_direction = (focus - ry_origin).normalize();

            ray.differentials = Some(RayDifferentials {
                rx_origin,
                ry_origin,
                rx_direction,
                ry_direction,
            })
        } else {
            ray.differentials = Some(RayDifferentials {
                rx_origin: ray.origin,
                ry_origin: ray.origin,
                rx_direction: (camera_point + self.dx_camera).normalize(),
                ry_direction: (camera_point + self.dy_camera).normalize(),
            });
        }

        ray.time = lerp(sample.time, self.shutter_open, self.shutter_close);
        *ray = ray.animated_transform(&self.camera_to_world);

        1.0
    }

    fn importance_emission(&self, ray: &Ray, raster_position: Option<&mut Point2F>) -> RGBSpectrum {
        // Interpolate camera transform.
        let mut camera_to_world = Transform::default();
        self.camera_to_world
            .interpolate(ray.time, &mut camera_to_world);

        // Check if direction is forward-facing.
        let cos_theta = ray
            .direction
            .dot(&Vec3::new(0.0, 0.0, 1.0).transform(&camera_to_world));
        if cos_theta <= 0.0 {
            return RGBSpectrum::default();
        }

        // Map ray to raster plane.
        let focus_point = if self.lens_radius > 0.0 {
            ray.at(self.focal_distance / cos_theta)
        } else {
            ray.at(1.0 / cos_theta)
        };
        let raster_point = focus_point
            .transform(&camera_to_world.inverse())
            .transform(&self.raster_to_camera.inverse());

        // Set raster point if requested.
        if let Some(p) = raster_position {
            *p = Point2F::new(raster_point.x, raster_point.y);
        }

        // Exit early for out of bounds points.
        let sample_bounds = Bounds2F::from(self.film.sample_bounds());
        if raster_point.x < sample_bounds.min.x
            || raster_point.x >= sample_bounds.max.x
            || raster_point.y < sample_bounds.min.y
            || raster_point.y >= sample_bounds.max.y
        {
            return RGBSpectrum::default();
        }

        // Compute lens area of perspective camera.
        let lens_area = if self.lens_radius != 0.0 {
            PI * self.lens_radius * self.lens_radius
        } else {
            1.0
        };

        // Return importance for point on image plane.
        let cos_theta_2 = cos_theta * cos_theta;
        RGBSpectrum::new(1.0 / (self.area * lens_area * cos_theta_2 * cos_theta_2))
    }

    fn importance_pdf(&self, ray: &Ray, position_pdf: &mut Float, direction_pdf: &mut Float) {
        // Interpolate camera transform.
        let mut camera_to_world = Transform::default();
        self.camera_to_world
            .interpolate(ray.time, &mut camera_to_world);

        // Check if direction is forward-facing.
        let cos_theta = ray
            .direction
            .dot(&Vec3::new(0.0, 0.0, 1.0).transform(&camera_to_world));
        if cos_theta <= 0.0 {
            *position_pdf = 0.0;
            *direction_pdf = 0.0;
            return;
        }

        // Map ray to raster plane.
        let focus_point = if self.lens_radius > 0.0 {
            ray.at(self.focal_distance / cos_theta)
        } else {
            ray.at(1.0 / cos_theta)
        };
        let raster_point = focus_point
            .transform(&camera_to_world.inverse())
            .transform(&self.raster_to_camera.inverse());

        // Exit early for out of bounds points.
        let sample_bounds = Bounds2F::from(self.film.sample_bounds());
        if raster_point.x < sample_bounds.min.x
            || raster_point.x >= sample_bounds.max.x
            || raster_point.y < sample_bounds.min.y
            || raster_point.y >= sample_bounds.max.y
        {
            *position_pdf = 0.0;
            *direction_pdf = 0.0;
            return;
        }

        // Compute lens area of perspective camera.
        let lens_area = if self.lens_radius != 0.0 {
            PI * self.lens_radius * self.lens_radius
        } else {
            1.0
        };

        *position_pdf = 1.0 / lens_area;
        *direction_pdf = 1.0 / (self.area * cos_theta * cos_theta * cos_theta);
    }

    fn importance_sample(
        &self,
        it: &Interaction,
        u: &Point2F,
        raster_point: &mut Point2F,
    ) -> CameraLensSample {
        // Uniformly sample lens interaction.
        let lens_point = self.lens_radius * concentric_sample_disk(u);
        let lens_point_world = Point3::new(lens_point.x, lens_point.y, 0.0)
            .animated_transform(&self.camera_to_world, it.time);

        let lens_it = Interaction {
            point: lens_point_world,
            time: it.time,
            ..Default::default()
        };

        let lens_area = if self.lens_radius != 0.0 {
            PI * self.lens_radius * self.lens_radius
        } else {
            1.0
        };

        let mut wi = lens_it.point - it.point;
        let dist = wi.length();
        wi /= dist;

        let radiance = self.importance_emission(&lens_it.spawn_ray(&-wi), Some(raster_point));
        let pdf = (dist * dist) / (lens_it.normal.abs_dot(&Normal::from(wi)) * lens_area);
        let visibility = VisibilityTester::new(
            Interaction {
                point: it.point,
                point_error: it.point_error,
                time: it.time,
                direction: it.direction,
                normal: it.normal,
                surface: None,
            },
            lens_it,
        );

        // Populate arguments and compute the importance value.
        CameraLensSample {
            radiance,
            wi,
            pdf,
            visibility,
        }
    }

    fn film(&self) -> &Film {
        &self.film
    }
}
