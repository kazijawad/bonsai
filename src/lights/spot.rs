use crate::{
    base::{
        bxdf::cos_theta,
        constants::{Float, PI},
        interaction::Interaction,
        light::{
            Light, LightFlag, LightPointSample, LightRaySample, VisibilityTester,
            DELTA_POSITION_LIGHT,
        },
        sampling::{uniform_cone_pdf, uniform_sample_cone},
        transform::Transform,
    },
    geometries::{
        mat4::Mat4, normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3,
    },
    spectra::rgb::RGBSpectrum,
};

pub struct SpotLight {
    light_to_world: Transform,
    world_to_light: Transform,
    position: Point3,
    intensity: RGBSpectrum,
    cos_total_width: Float,
    cos_falloff_start: Float,
    flag: LightFlag,
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
            flag: DELTA_POSITION_LIGHT,
        }
    }

    fn falloff(&self, w: &Vec3) -> Float {
        let w_light = w.transform(&self.world_to_light).normalize();

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

    fn sample_point(&self, it: &Interaction, _sample: &Point2F) -> LightPointSample {
        let wi = (self.position - it.point).normalize();
        LightPointSample {
            radiance: self.intensity * self.falloff(&-wi)
                / self.position.distance_squared(&it.point),
            wi,
            pdf: 1.0,
            visibility: Some(VisibilityTester::new(
                Interaction {
                    point: it.point,
                    point_error: it.point_error,
                    time: it.time,
                    direction: it.direction,
                    normal: it.normal,
                    surface: None,
                },
                Interaction {
                    point: self.position,
                    time: it.time,
                    ..Default::default()
                },
            )),
        }
    }

    fn sample_ray(&self, u1: &Point2F, _: &Point2F, time: Float) -> LightRaySample {
        let w = uniform_sample_cone(u1, self.cos_total_width);

        let ray = Ray::new(
            &self.position,
            &w.transform(&self.light_to_world),
            Float::INFINITY,
            time,
        );

        LightRaySample {
            radiance: self.intensity * self.falloff(&ray.direction),
            ray,
            light_normal: Normal::from(ray.direction),
            position_pdf: 1.0,
            direction_pdf: uniform_cone_pdf(self.cos_total_width),
        }
    }

    fn ray_pdf(
        &self,
        ray: &Ray,
        _light_normal: &Normal,
        position_pdf: &mut Float,
        direction_pdf: &mut Float,
    ) {
        *position_pdf = 0.0;
        *direction_pdf =
            if cos_theta(&ray.direction.transform(&self.world_to_light)) >= self.cos_total_width {
                uniform_cone_pdf(self.cos_total_width)
            } else {
                0.0
            };
    }

    fn flag(&self) -> LightFlag {
        self.flag
    }
}
