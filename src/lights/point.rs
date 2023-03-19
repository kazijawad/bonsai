use crate::{
    base::{
        constants::{Float, PI},
        interaction::Interaction,
        light::{Light, VisibilityTester},
        transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
    utils::sampling::{uniform_sample_sphere, uniform_sphere_pdf},
};

#[derive(Debug, Clone)]
pub struct PointLight {
    position: Point3,
    intensity: RGBSpectrum,
}

impl PointLight {
    pub fn new(light_to_world: Transform, intensity: RGBSpectrum) -> Self {
        let position = Point3::default().transform(&light_to_world);
        Self {
            position,
            intensity,
        }
    }
}

impl Light for PointLight {
    fn power(&self) -> RGBSpectrum {
        4.0 * PI * self.intensity
    }

    fn sample_li(
        &self,
        it: &dyn Interaction,
        _u: &Point2,
        wi: &mut Vec3,
        pdf: &mut Float,
    ) -> (RGBSpectrum, VisibilityTester) {
        *wi = (self.position - it.position()).normalize();
        *pdf = 1.0;
        (
            self.intensity / self.position.distance_squared(&it.position()),
            VisibilityTester::new(
                BaseInteraction::new(&it.position(), it.time()),
                BaseInteraction::new(&self.position, it.time()),
            ),
        )
    }

    fn pdf_li(&self, _it: &dyn Interaction, _wi: &Vec3) -> Float {
        0.0
    }

    fn sample_le(
        &self,
        u1: &Point2,
        _u2: &Point2,
        time: Float,
        ray: &mut Ray,
        light_norm: &mut Normal,
        pdf_pos: &mut Float,
        pdf_dir: &mut Float,
    ) -> RGBSpectrum {
        *ray = Ray::new(
            &self.position,
            &uniform_sample_sphere(u1),
            Float::INFINITY,
            time,
        );
        *light_norm = Normal::from(ray.direction);
        *pdf_pos = 1.0;
        *pdf_dir = uniform_sphere_pdf();
        self.intensity
    }

    fn pdf_le(&self, _ray: &Ray, _light_norm: Normal, pdf_pos: &mut Float, pdf_dir: &mut Float) {
        *pdf_pos = 0.0;
        *pdf_dir = uniform_sphere_pdf();
    }
}
