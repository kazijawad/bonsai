use crate::{
    base::{
        constants::{Float, PI},
        interaction::Interaction,
        light::{Light, VisibilityTester},
        sampling::{uniform_sample_sphere, uniform_sphere_pdf},
        transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
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

    fn sample_point(
        &self,
        it: &dyn Interaction,
        _sample: &Point2,
    ) -> (RGBSpectrum, Vec3, Float, VisibilityTester) {
        (
            self.intensity / self.position.distance_squared(&it.position()),
            (self.position - it.position()).normalize(),
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
        let ray = Ray::new(
            &self.position,
            &uniform_sample_sphere(origin_sample),
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

    fn pdf_ray(&self, _ray: &Ray, _surface_normal: &Normal) -> (Float, Float) {
        (0.0, uniform_sphere_pdf())
    }
}
