use crate::{
    base::{
        light::Light,
        material::{Material, TransportMode},
        primitive::Primitive,
    },
    geometries::{bounds3::Bounds3, ray::Ray},
    interactions::surface::SurfaceInteraction,
};

pub struct Scene {
    pub lights: Vec<Box<dyn Light>>,
    pub infinite_lights: Vec<Box<dyn Light>>,
    aggregate: Box<dyn Primitive>,
    bounds: Bounds3,
}

impl Scene {
    pub fn new(aggregate: Box<dyn Primitive>, mut lights: Vec<Box<dyn Light>>) -> Self {
        let bounds = aggregate.world_bound();
        let mut scene = Self {
            bounds,
            lights: vec![],
            infinite_lights: vec![],
            aggregate,
        };

        for light in lights.iter_mut() {
            light.preprocess(&mut scene);
            if light.is_infinite() {
                scene.infinite_lights.push(light.clone());
            }
        }
        scene.lights = lights;

        scene
    }
}

impl Primitive for Scene {
    fn world_bound(&self) -> Bounds3 {
        self.bounds
    }

    fn intersect(&self, ray: &mut Ray, interaction: &mut SurfaceInteraction) -> bool {
        self.aggregate.intersect(ray, interaction)
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        self.aggregate.intersect_test(ray)
    }

    fn compute_scattering_functions(
        &self,
        _interaction: &mut SurfaceInteraction,
        _transport_mode: TransportMode,
        _allow_multiple_lobes: bool,
    ) {
        panic!("Scene::compute_scattering_function should not be called")
    }

    fn material(&self) -> Option<&dyn Material> {
        None
    }
}
