use crate::{
    base::{
        constants::{Float, PI},
        interaction::Interaction,
        light::{
            Light, LightFlag, LightPointSample, LightRaySample, VisibilityTester,
            DELTA_DIRECTION_LIGHT,
        },
        primitive::Primitive,
        sampling::concentric_sample_disk,
    },
    geometries::{normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
};

pub struct DirectionalLight {
    intensity: RGBSpectrum,
    direction: Vec3,
    world_center: Point3,
    world_radius: Float,
    flag: LightFlag,
}

pub struct DirectionalLightOptions<'a> {
    pub scene: &'a (dyn Primitive<'a> + 'a),
    pub from: Point3,
    pub to: Point3,
    pub intensity: RGBSpectrum,
}

impl DirectionalLight {
    pub fn new(opts: DirectionalLightOptions) -> Self {
        let direction = opts.from - opts.to;

        let mut world_center = Point3::default();
        let mut world_radius = 0.0;
        opts.scene
            .world_bound()
            .bounding_sphere(&mut world_center, &mut world_radius);

        Self {
            intensity: opts.intensity,
            direction,
            world_center,
            world_radius,
            flag: DELTA_DIRECTION_LIGHT,
        }
    }
}

impl Light for DirectionalLight {
    fn power(&self) -> RGBSpectrum {
        self.intensity * PI * self.world_radius * self.world_radius
    }

    fn sample_point(&self, it: &dyn Interaction, _: &Point2F) -> LightPointSample {
        let p_outside = it.p() + self.direction * (2.0 * self.world_radius);
        LightPointSample {
            radiance: self.intensity,
            wi: self.direction,
            pdf: 1.0,
            visibility: Some(VisibilityTester::new(
                BaseInteraction::from(it),
                BaseInteraction {
                    p: p_outside,
                    p_error: Vec3::default(),
                    time: it.time(),
                    wo: Vec3::default(),
                    n: Normal::default(),
                },
            )),
        }
    }

    fn sample_ray(&self, u1: &Point2F, _: &Point2F, time: Float) -> LightRaySample {
        // Choose point on disk oriented toward infinite light direction.
        let (v1, v2) = Vec3::coordinate_system(&self.direction);
        let concentric_disk = concentric_sample_disk(u1);
        let disk_point = self.world_center
            + self.world_radius * (concentric_disk.x * v1 + concentric_disk.y * v2);

        // Set ray origin and direction for infinite light ray.
        let ray = Ray::new(
            &(disk_point + self.world_radius * self.direction),
            &-self.direction,
            Float::INFINITY,
            time,
        );

        LightRaySample {
            radiance: self.intensity,
            ray,
            light_normal: Normal::from(ray.direction),
            position_pdf: 1.0 / (PI * self.world_radius * self.world_radius),
            direction_pdf: 1.0,
        }
    }

    fn ray_pdf(&self, _ray: &Ray, _surface_normal: &Normal) -> (Float, Float) {
        (1.0 / (PI * self.world_radius * self.world_radius), 0.0)
    }

    fn flag(&self) -> LightFlag {
        self.flag
    }
}
