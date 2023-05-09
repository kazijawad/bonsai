use crate::{
    base::{
        constants::{Float, PI},
        interaction::Interaction,
        light::{AreaLight, Light, VisibilityTester},
        sampling::{cosine_hemisphere_pdf, cosine_sample_hemisphere},
        shape::Shape,
    },
    geometries::{normal::Normal, point2::Point2F, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
};

pub struct DiffuseAreaLight<'a> {
    intensity: RGBSpectrum,
    shape: &'a (dyn Shape + 'a),
    double_sided: bool,
    area: Float,
}

pub struct DiffuseAreaLightOptions<'a> {
    pub intensity: RGBSpectrum,
    pub shape: &'a (dyn Shape + 'a),
    pub double_sided: bool,
}

impl<'a> DiffuseAreaLight<'a> {
    pub fn new(opts: DiffuseAreaLightOptions<'a>) -> Self {
        let area = opts.shape.area();
        Self {
            intensity: opts.intensity,
            shape: opts.shape,
            double_sided: opts.double_sided,
            area,
        }
    }
}

impl<'a> Light for DiffuseAreaLight<'a> {
    fn power(&self) -> RGBSpectrum {
        if self.double_sided {
            2.0 * self.intensity * self.area * PI
        } else {
            self.intensity * self.area * PI
        }
    }

    fn sample_point(
        &self,
        it: &dyn Interaction,
        sample: &Point2F,
    ) -> (RGBSpectrum, Vec3, Float, VisibilityTester) {
        let mut pdf = 0.0;
        let point_it = self.shape.sample_from_ref(it, sample, &mut pdf);

        let tester = VisibilityTester::new(
            BaseInteraction::new(&it.position(), it.time()),
            BaseInteraction::new(&point_it.position(), point_it.time()),
        );

        if pdf == 0.0 || (point_it.position() - it.position()).length_squared() == 0.0 {
            return (RGBSpectrum::default(), Vec3::default(), 0.0, tester);
        }

        let wi = (point_it.position() - it.position()).normalize();
        (self.emission(point_it.as_ref(), &wi), wi, pdf, tester)
    }

    fn pdf(&self, interaction: &dyn Interaction, incident_direction: &Vec3) -> Float {
        self.shape.pdf_from_ref(interaction, incident_direction)
    }

    fn sample_ray(
        &self,
        origin_sample: &Point2F,
        direction_sample: &Point2F,
        _time: Float,
    ) -> (RGBSpectrum, Ray, Normal, Float, Float) {
        let mut pdf_position = 0.0;
        let point_it = self.shape.sample(origin_sample, &mut pdf_position);

        // Sample a cosine-weighted outgoing direction.
        let mut direction: Vec3;
        let pdf_direction: Float;
        if self.double_sided {
            let mut sample = direction_sample.clone();
            // Choose a side to sample and then remap the sample to [0,1] before
            // applying cosine-weighted hemisphere sampling for the chosen side.
            if sample.x < 0.5 {
                sample.x = (sample.x * 2.0).min(1.0 - Float::EPSILON);
                direction = cosine_sample_hemisphere(&sample);
            } else {
                sample.x = ((sample.x - 0.5) * 2.0).min(1.0 - Float::EPSILON);
                direction = cosine_sample_hemisphere(&sample);
                direction.z *= -1.0;
            }
            pdf_direction = 0.5 * cosine_hemisphere_pdf(direction.z.abs());
        } else {
            direction = cosine_sample_hemisphere(direction_sample);
            pdf_direction = cosine_hemisphere_pdf(direction.z);
        }

        let normal = Vec3::from(point_it.normal());
        let (v1, v2) = Vec3::coordinate_system(&normal);
        direction = direction.x * v1 + direction.y * v2 + direction.z * normal;

        (
            self.emission(point_it.as_ref(), &direction),
            point_it.spawn_ray(&direction),
            point_it.normal(),
            pdf_position,
            pdf_direction,
        )
    }

    fn pdf_ray(&self, ray: &Ray, surface_normal: &Normal) -> (Float, Float) {
        let interaction = BaseInteraction {
            p: ray.origin,
            p_error: Vec3::default(),
            time: ray.time,
            wo: Vec3::from(*surface_normal),
            n: surface_normal.clone(),
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
}

impl<'a> AreaLight for DiffuseAreaLight<'a> {
    fn emission(&self, interaction: &dyn Interaction, direction: &Vec3) -> RGBSpectrum {
        if self.double_sided || interaction.normal().dot(&Normal::from(*direction)) > 0.0 {
            self.intensity
        } else {
            RGBSpectrum::default()
        }
    }
}
