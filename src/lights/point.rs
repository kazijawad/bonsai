use crate::{
    base::{
        constants::{Float, PI},
        interaction::Interaction,
        light::{
            Light, LightFlag, LightPointSample, LightRaySample, VisibilityTester,
            DELTA_POSITION_LIGHT,
        },
        sampling::{uniform_sample_sphere, uniform_sphere_pdf},
        transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2F, point3::Point3, ray::Ray},
    spectra::rgb::RGBSpectrum,
};

pub struct PointLight {
    position: Point3,
    intensity: RGBSpectrum,
    flag: LightFlag,
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
            flag: DELTA_POSITION_LIGHT,
        }
    }
}

impl Light for PointLight {
    fn power(&self) -> RGBSpectrum {
        4.0 * PI * self.intensity
    }

    fn sample_point(&self, it: &Interaction, _: &Point2F) -> LightPointSample {
        LightPointSample {
            radiance: self.intensity / self.position.distance_squared(&it.point),
            wi: (self.position - it.point).normalize(),
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
        let ray = Ray::new(
            &self.position,
            &uniform_sample_sphere(u1),
            Float::INFINITY,
            time,
        );
        LightRaySample {
            radiance: self.intensity,
            ray,
            light_normal: Normal::from(ray.direction),
            position_pdf: 1.0,
            direction_pdf: uniform_sphere_pdf(),
        }
    }

    fn ray_pdf(&self, _ray: &Ray, _surface_normal: &Normal) -> (Float, Float) {
        (0.0, uniform_sphere_pdf())
    }

    fn flag(&self) -> LightFlag {
        self.flag
    }
}
