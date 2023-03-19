use crate::{
    base::{
        constants::{Float, PI},
        interaction::Interaction,
        light::{Light, VisibilityTester},
        primitive::Primitive,
        scene::Scene,
        transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
    utils::sampling::concentric_sample_disk,
};

#[derive(Debug, Clone)]
pub struct DirectionalLight {
    intensity: RGBSpectrum,
    direction: Vec3,
    world_center: Point3,
    world_radius: Float,
}

impl DirectionalLight {
    pub fn new(intensity: RGBSpectrum, direction: Vec3) -> Self {
        Self {
            intensity,
            direction,
            world_center: Point3::default(),
            world_radius: 0.0,
        }
    }
}

impl Light for DirectionalLight {
    fn preprocess(&mut self, scene: &Scene) {
        scene
            .world_bound()
            .bounding_sphere(&mut self.world_center, &mut self.world_radius);
    }

    fn power(&self) -> RGBSpectrum {
        self.intensity * PI * self.world_radius * self.world_radius
    }

    fn sample_li(
        &self,
        it: &dyn Interaction,
        _u: &Point2,
        wi: &mut Vec3,
        pdf: &mut Float,
    ) -> (RGBSpectrum, VisibilityTester) {
        *wi = self.direction;
        *pdf = 1.0;
        let outside_point = it.position() + self.direction * (2.0 * self.world_radius);
        (
            self.intensity,
            VisibilityTester::new(
                BaseInteraction::new(&it.position(), it.time()),
                BaseInteraction::new(&outside_point, it.time()),
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
        // Choose point on disk oriented toward infinite light direction.
        let (v1, v2) = Vec3::coordinate_system(&self.direction);
        let concentric_disk = concentric_sample_disk(u1);
        let disk_point = self.world_center
            + self.world_radius * (concentric_disk.x * v1 + concentric_disk.y * v2);

        // Set ray origin and direction for infinite light ray.
        *ray = Ray::new(
            &(disk_point + self.world_radius * self.direction),
            &-self.direction,
            Float::INFINITY,
            time,
        );
        *light_norm = ray.direction.into();
        *pdf_pos = 1.0 / (PI * self.world_radius * self.world_radius);
        *pdf_dir = 1.0;

        self.intensity
    }

    fn pdf_le(&self, _ray: &Ray, _light_norm: Normal, pdf_pos: &mut Float, pdf_dir: &mut Float) {
        *pdf_pos = 1.0 / (PI * self.world_radius * self.world_radius);
        *pdf_dir = 0.0;
    }
}
