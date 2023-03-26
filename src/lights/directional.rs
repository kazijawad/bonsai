use crate::{
    base::{
        constants::{Float, PI},
        interaction::Interaction,
        light::{Light, VisibilityTester},
        sampling::concentric_sample_disk,
        scene::Scene,
    },
    geometries::{normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
};

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

    fn sample_point(
        &self,
        it: &dyn Interaction,
        _sample: &Point2,
    ) -> (RGBSpectrum, Vec3, Float, VisibilityTester) {
        let outside_point = it.position() + self.direction * (2.0 * self.world_radius);
        (
            self.intensity,
            self.direction,
            1.0,
            VisibilityTester::new(
                BaseInteraction::new(&it.position(), it.time()),
                BaseInteraction::new(&outside_point, it.time()),
            ),
        )
    }

    fn sample_ray(
        &self,
        origin_sample: &Point2,
        _direction_sample: &Point2,
        time: Float,
    ) -> (RGBSpectrum, Ray, Normal, Float, Float) {
        // Choose point on disk oriented toward infinite light direction.
        let (v1, v2) = Vec3::coordinate_system(&self.direction);
        let concentric_disk = concentric_sample_disk(origin_sample);
        let disk_point = self.world_center
            + self.world_radius * (concentric_disk.x * v1 + concentric_disk.y * v2);

        // Set ray origin and direction for infinite light ray.
        let ray = Ray::new(
            &(disk_point + self.world_radius * self.direction),
            &-self.direction,
            Float::INFINITY,
            time,
        );

        (
            self.intensity,
            ray,
            Normal::from(ray.direction),
            1.0 / (PI * self.world_radius * self.world_radius),
            1.0,
        )
    }

    fn pdf_ray(&self, _ray: &Ray, _surface_normal: &Normal) -> (Float, Float) {
        (1.0 / (PI * self.world_radius * self.world_radius), 0.0)
    }
}
