use crate::{
    base::{
        light::{Light, INFINITE_LIGHT},
        primitive::Primitive,
    },
    geometries::{bounds3::Bounds3, ray::Ray},
    interactions::surface::SurfaceInteraction,
};

pub struct Scene {
    pub lights: Vec<Box<dyn Light>>,
    pub infinite_lights_index: Vec<usize>,
    pub aggregate: Box<dyn Primitive>,
    pub bounds: Bounds3,
}

impl Scene {
    pub fn new(aggregate: Box<dyn Primitive>, lights: Vec<Box<dyn Light>>) -> Self {
        let bounds = aggregate.bounds();

        let mut infinite_lights_index = Vec::new();
        for (i, light) in lights.iter().enumerate() {
            if light.flag() & INFINITE_LIGHT != 0 {
                infinite_lights_index.push(i)
            }
        }

        Self {
            bounds,
            lights,
            infinite_lights_index,
            aggregate,
        }
    }

    pub fn intersect(&self, ray: &mut Ray, si: &mut SurfaceInteraction) -> bool {
        self.aggregate.intersect(ray, si)
    }

    pub fn intersect_test(&self, ray: &Ray) -> bool {
        self.aggregate.intersect_test(ray)
    }
}
