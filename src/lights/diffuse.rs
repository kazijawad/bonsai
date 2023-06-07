use std::sync::Arc;

use crate::{
    base::{
        constants::{Float, ONE_MINUS_EPSILON, PI},
        interaction::Interaction,
        light::{
            AreaLight, Light, LightFlag, LightPointSample, LightRaySample, VisibilityTester,
            AREA_LIGHT,
        },
        sampling::{cosine_hemisphere_pdf, cosine_sample_hemisphere},
        shape::Shape,
    },
    geometries::{normal::Normal, point2::Point2F, ray::Ray, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
};

pub struct DiffuseAreaLight {
    intensity: RGBSpectrum,
    shape: Arc<dyn Shape>,
    double_sided: bool,
    area: Float,
    flag: LightFlag,
}

pub struct DiffuseAreaLightOptions {
    pub intensity: RGBSpectrum,
    pub shape: Arc<dyn Shape>,
    pub double_sided: bool,
}

impl DiffuseAreaLight {
    pub fn new(opts: DiffuseAreaLightOptions) -> Self {
        let area = opts.shape.area();
        Self {
            intensity: opts.intensity,
            shape: opts.shape,
            double_sided: opts.double_sided,
            area,
            flag: AREA_LIGHT,
        }
    }
}

impl Light for DiffuseAreaLight {
    fn power(&self) -> RGBSpectrum {
        if self.double_sided {
            2.0 * self.intensity * self.area * PI
        } else {
            self.intensity * self.area * PI
        }
    }

    fn sample_point(&self, it: &Interaction, sample: &Point2F) -> LightPointSample {
        let mut pdf = 0.0;
        let point_it = self.shape.sample_from_ref(it, sample, &mut pdf);
        if pdf == 0.0 || (point_it.point - it.point).length_squared() == 0.0 {
            return LightPointSample {
                radiance: RGBSpectrum::default(),
                wi: Vec3::default(),
                pdf: 0.0,
                visibility: None,
            };
        }

        let wi = (point_it.point - it.point).normalize();
        let radiance = self.emission(&point_it, &wi);

        let visibility = VisibilityTester::new(
            Interaction {
                point: it.point,
                point_error: it.point_error,
                time: it.time,
                direction: it.direction,
                normal: it.normal,
                surface: None,
            },
            point_it,
        );

        LightPointSample {
            radiance,
            wi,
            pdf,
            visibility: Some(visibility),
        }
    }

    fn point_pdf(&self, it: &Interaction, dir: &Vec3) -> Float {
        self.shape.pdf_from_ref(it, dir)
    }

    fn sample_ray(
        &self,
        origin_sample: &Point2F,
        direction_sample: &Point2F,
        _: Float,
    ) -> LightRaySample {
        let mut position_pdf = 0.0;
        let point_it = self.shape.sample(origin_sample, &mut position_pdf);

        // Sample a cosine-weighted outgoing direction.
        let mut direction: Vec3;
        let direction_pdf: Float;
        if self.double_sided {
            let mut sample = direction_sample.clone();
            // Choose a side to sample and then remap the sample to [0,1] before
            // applying cosine-weighted hemisphere sampling for the chosen side.
            if sample.x < 0.5 {
                sample.x = (sample.x * 2.0).min(ONE_MINUS_EPSILON);
                direction = cosine_sample_hemisphere(&sample);
            } else {
                sample.x = ((sample.x - 0.5) * 2.0).min(ONE_MINUS_EPSILON);
                direction = cosine_sample_hemisphere(&sample);
                direction.z *= -1.0;
            }
            direction_pdf = 0.5 * cosine_hemisphere_pdf(direction.z.abs());
        } else {
            direction = cosine_sample_hemisphere(direction_sample);
            direction_pdf = cosine_hemisphere_pdf(direction.z);
        }

        let normal = Vec3::from(point_it.normal);
        let (v1, v2) = Vec3::coordinate_system(&normal);
        direction = direction.x * v1 + direction.y * v2 + direction.z * normal;

        LightRaySample {
            radiance: self.emission(&point_it, &direction),
            ray: point_it.spawn_ray(&direction),
            light_normal: point_it.normal,
            position_pdf,
            direction_pdf,
        }
    }

    fn ray_pdf(&self, ray: &Ray, surface_normal: &Normal) -> (Float, Float) {
        let interaction = Interaction {
            point: ray.origin,
            point_error: Vec3::default(),
            time: ray.time,
            direction: Vec3::from(*surface_normal),
            normal: surface_normal.clone(),
            surface: None,
        };
        (
            self.shape.pdf(&interaction),
            if self.double_sided {
                0.5 * cosine_hemisphere_pdf(surface_normal.abs_dot(&Normal::from(ray.direction)))
            } else {
                cosine_hemisphere_pdf(surface_normal.dot(&Normal::from(ray.direction)))
            },
        )
    }

    fn flag(&self) -> LightFlag {
        self.flag
    }
}

impl AreaLight for DiffuseAreaLight {
    fn emission(&self, it: &Interaction, dir: &Vec3) -> RGBSpectrum {
        if self.double_sided || it.normal.dot(&Normal::from(*dir)) > 0.0 {
            self.intensity
        } else {
            RGBSpectrum::default()
        }
    }
}
