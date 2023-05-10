use crate::{
    base::{
        bxdf::cos_theta,
        constants::{Float, PI},
        interaction::Interaction,
        light::{Light, LightPointSample, VisibilityTester},
        sampling::{uniform_cone_pdf, uniform_sample_cone},
        transform::Transform,
    },
    geometries::{
        mat4::Mat4, normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3,
    },
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
};

pub struct SpotLight {
    light_to_world: Transform,
    world_to_light: Transform,
    position: Point3,
    intensity: RGBSpectrum,
    cos_total_width: Float,
    cos_falloff_start: Float,
}

pub struct SpotLightOptions {
    pub transform: Transform,
    pub from: Point3,
    pub to: Point3,
    pub intensity: RGBSpectrum,
    pub cone_angle: Float,
    pub cone_delta_angle: Float,
}

impl SpotLight {
    pub fn new(opts: SpotLightOptions) -> Self {
        let dir = (opts.to - opts.from).normalize();
        let (du, dv) = Vec3::coordinate_system(&dir);

        let light_to_world = opts.transform
            * Transform::translate(&opts.from.into())
            * Transform::from(Mat4::new(
                du.x, du.y, du.z, 0.0, dv.x, dv.y, dv.z, 0.0, dir.x, dir.y, dir.z, 0.0, 0.0, 0.0,
                0.0, 1.0,
            ))
            .inverse();

        let world_to_light = light_to_world.inverse();

        let position = Point3::default().transform(&light_to_world);

        Self {
            light_to_world,
            world_to_light,
            position,
            intensity: opts.intensity,
            cos_total_width: opts.cone_angle.to_radians().cos(),
            cos_falloff_start: (opts.cone_angle - opts.cone_delta_angle).to_radians().cos(),
        }
    }

    fn falloff(&self, w: &Vec3) -> Float {
        let w_light = w.transform(&self.world_to_light, false).0.normalize();

        let cos_theta = w_light.z;
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

    fn sample_point(&self, it: &dyn Interaction, _sample: &Point2F) -> LightPointSample {
        let wi = (self.position - it.p()).normalize();
        LightPointSample {
            radiance: self.intensity * self.falloff(&-wi) / self.position.distance_squared(&it.p()),
            wi,
            pdf: 1.0,
            visibility: Some(VisibilityTester::new(
                BaseInteraction::from(it),
                BaseInteraction {
                    p: self.position,
                    p_error: Vec3::default(),
                    time: it.time(),
                    wo: Vec3::default(),
                    n: Normal::default(),
                },
            )),
        }
    }

    fn sample_ray(
        &self,
        u1: &Point2F,
        _: &Point2F,
        time: Float,
    ) -> (RGBSpectrum, Ray, Normal, Float, Float) {
        let w = uniform_sample_cone(u1, self.cos_total_width);
        let ray = Ray::new(
            &self.position,
            &w.transform(&self.light_to_world, false).0,
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

    fn ray_pdf(&self, ray: &Ray, _: &Normal) -> (Float, Float) {
        (
            0.0,
            if cos_theta(&ray.direction.transform(&self.world_to_light, false).0)
                >= self.cos_total_width
            {
                uniform_cone_pdf(self.cos_total_width)
            } else {
                0.0
            },
        )
    }
}
