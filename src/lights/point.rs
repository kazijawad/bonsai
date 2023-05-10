use crate::{
    base::{
        constants::{Float, PI},
        interaction::Interaction,
        light::{Light, LightPointSample, VisibilityTester},
        sampling::{uniform_sample_sphere, uniform_sphere_pdf},
        transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
};

pub struct PointLight {
    position: Point3,
    intensity: RGBSpectrum,
}

pub struct PointLightOptions {
    pub transform: Transform,
    pub intensity: RGBSpectrum,
}

impl PointLight {
    pub fn new(opts: PointLightOptions) -> Self {
        let position = Point3::default().transform(&opts.transform);
        Self {
            position,
            intensity: opts.intensity,
        }
    }
}

impl Light for PointLight {
    fn power(&self) -> RGBSpectrum {
        4.0 * PI * self.intensity
    }

    fn sample_point(&self, it: &dyn Interaction, _: &Point2F) -> LightPointSample {
        LightPointSample {
            radiance: self.intensity / self.position.distance_squared(&it.p()),
            wi: (self.position - it.p()).normalize(),
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
        let ray = Ray::new(
            &self.position,
            &uniform_sample_sphere(u1),
            Float::INFINITY,
            time,
        );
        (
            self.intensity,
            ray,
            Normal::from(ray.direction),
            1.0,
            uniform_sphere_pdf(),
        )
    }

    fn ray_pdf(&self, _ray: &Ray, _surface_normal: &Normal) -> (Float, Float) {
        (0.0, uniform_sphere_pdf())
    }
}
