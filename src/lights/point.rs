use crate::{
    base::{
        interaction::Interaction,
        light::{Light, LightFlag, VisibilityTester},
        transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
    utils::{
        math::{Float, PI},
        sampling::{uniform_sample_sphere, uniform_sphere_pdf},
    },
};

#[derive(Debug, Clone)]
pub struct PointLight {
    light_to_world: Transform,
    world_to_light: Transform,
    position: Point3,
    intensity: RGBSpectrum,
    flag: LightFlag,
}

#[derive(Debug)]
pub struct PointLightDescriptor {
    pub intensity: RGBSpectrum,
    pub scale: RGBSpectrum,
    pub from: Point3,
}

impl PointLight {
    pub fn create(desc: &PointLightDescriptor, light_to_world: Transform) -> Self {
        let light_to_world = Transform::translate(&Vec3::from(desc.from)) * light_to_world;
        Self::new(light_to_world, desc.intensity * desc.scale)
    }

    pub fn new(light_to_world: Transform, intensity: RGBSpectrum) -> Self {
        let world_to_light = light_to_world.inverse();
        let position = Point3::default().transform(&light_to_world);
        Self {
            light_to_world,
            world_to_light,
            position,
            intensity,
            flag: LightFlag::DeltaPosition,
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
        u: &Point2,
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

    fn pdf_li(&self, it: &dyn Interaction, wi: &Vec3) -> Float {
        0.0
    }

    fn sample_le(
        &self,
        u1: &Point2,
        u2: &Point2,
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

    fn pdf_le(&self, ray: &Ray, light_norm: Normal, pdf_pos: &mut Float, pdf_dir: &mut Float) {
        *pdf_pos = 0.0;
        *pdf_dir = uniform_sphere_pdf();
    }

    fn flag(&self) -> LightFlag {
        self.flag
    }
}

impl Default for PointLightDescriptor {
    fn default() -> Self {
        Self {
            intensity: RGBSpectrum::new(1.0),
            scale: RGBSpectrum::new(1.0),
            from: Point3::new(0.0, 0.0, 0.0),
        }
    }
}
