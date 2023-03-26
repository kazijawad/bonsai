use crate::{
    base::{
        constants::{Float, PI},
        interaction::Interaction,
        light::{Light, VisibilityTester},
        sampling::{uniform_cone_pdf, uniform_sample_cone},
        transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
    utils::bxdf::cos_theta,
};

pub struct SpotLight {
    light_to_world: Transform,
    world_to_light: Transform,
    position: Point3,
    intensity: RGBSpectrum,
    cos_total_width: Float,
    cos_falloff_start: Float,
}

impl SpotLight {
    pub fn new(
        light_to_world: Transform,
        intensity: RGBSpectrum,
        total_width: Float,
        falloff_start: Float,
    ) -> Self {
        let world_to_light = light_to_world.inverse();
        let position = Point3::default().transform(&light_to_world);
        Self {
            light_to_world,
            world_to_light,
            position,
            intensity,
            cos_total_width: total_width.to_radians().cos(),
            cos_falloff_start: falloff_start.to_radians().cos(),
        }
    }

    fn falloff(&self, w: &Vec3) -> Float {
        let wl = self.world_to_light.transform_vec(w).normalize();

        let cos_theta = wl.z;
        if cos_theta < self.cos_total_width {
            return 0.0;
        }
        if cos_theta >= self.cos_falloff_start {
            return 1.0;
        }

        // Compute falloff inside cone.
        let delta =
            (cos_theta - self.cos_total_width) / (self.cos_falloff_start - self.cos_total_width);
        (delta * delta) * (delta * delta)
    }
}

impl Light for SpotLight {
    fn power(&self) -> RGBSpectrum {
        self.intensity * 2.0 * PI * (1.0 - 0.5 * (self.cos_falloff_start + self.cos_total_width))
    }

    fn sample_point(
        &self,
        it: &dyn Interaction,
        _sample: &Point2,
    ) -> (RGBSpectrum, Vec3, Float, VisibilityTester) {
        let wi = (self.position - it.position()).normalize();
        (
            self.intensity * self.falloff(&-wi) / self.position.distance_squared(&it.position()),
            wi,
            1.0,
            VisibilityTester::new(
                BaseInteraction::new(&it.position(), it.time()),
                BaseInteraction::new(&self.position, it.time()),
            ),
        )
    }

    fn sample_ray(
        &self,
        origin_sample: &Point2,
        _direction_sample: &Point2,
        time: Float,
    ) -> (RGBSpectrum, Ray, Normal, Float, Float) {
        let w = uniform_sample_cone(origin_sample, self.cos_total_width);
        let ray = Ray::new(
            &self.position,
            &self.light_to_world.transform_vec(&w),
            Float::INFINITY,
            time,
        );
        (
            self.intensity * self.falloff(&ray.direction),
            ray,
            Normal::from(ray.direction),
            1.0,
            uniform_cone_pdf(self.cos_total_width),
        )
    }

    fn pdf_ray(&self, ray: &Ray, _surface_normal: &Normal) -> (Float, Float) {
        (
            0.0,
            if cos_theta(&self.world_to_light.transform_vec(&ray.direction)) >= self.cos_total_width
            {
                uniform_cone_pdf(self.cos_total_width)
            } else {
                0.0
            },
        )
    }
}
