use crate::{
    base::{light::Light, primitive::Primitive},
    geometries::{bounds3::Bounds3, ray::Ray},
    interactions::surface::SurfaceInteraction,
};

pub struct Scene<'a> {
    pub lights: Vec<&'a (dyn Light + 'a)>,
    pub infinite_lights: Vec<&'a (dyn Light + 'a)>,
    aggregate: &'a (dyn Primitive<'a> + 'a),
    bounds: Bounds3,
}

impl<'a> Scene<'a> {
    pub fn new(aggregate: &'a (dyn Primitive<'a> + 'a), lights: Vec<&'a (dyn Light + 'a)>) -> Self {
        let bounds = aggregate.world_bound();

        let infinite_lights = lights
            .iter()
            .map(|v| *v)
            .filter(|v| v.is_infinite())
            .collect();

        Self {
            bounds,
            lights,
            infinite_lights,
            aggregate,
        }
    }

    pub fn world_bound(&self) -> Bounds3 {
        self.bounds
    }

    pub fn intersect(&self, ray: &mut Ray, si: &mut SurfaceInteraction<'a>) -> bool {
        self.aggregate.intersect(ray, si)
    }

    pub fn intersect_test(&self, ray: &Ray) -> bool {
        self.aggregate.intersect_test(ray)
    }
}
