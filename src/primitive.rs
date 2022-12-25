use std::sync::Arc;

use crate::{
    geometries::ray::Ray, interaction::SurfaceInteraction, material::Material, math::Float,
    shape::Shape,
};

pub trait Primitive: Send + Sync {
    fn intersect(&self, ray: &Ray, t_hit: &mut Float, interaction: &mut SurfaceInteraction)
        -> bool;
    fn intersect_test(&self, ray: &Ray) -> bool;
}

pub struct AggregatePrimitive<'a> {
    primitives: Vec<Arc<dyn Primitive + 'a>>,
}

pub struct GeometricPrimitive<'a> {
    pub shape: Arc<dyn Shape + 'a>,
    pub material: Arc<dyn Material + 'a>,
}

impl<'a> AggregatePrimitive<'a> {
    pub fn new(primitives: Vec<Arc<dyn Primitive + 'a>>) -> Arc<Self> {
        Arc::new(Self { primitives })
    }
}

impl<'a> GeometricPrimitive<'a> {
    pub fn new(shape: Arc<dyn Shape + 'a>, material: Arc<dyn Material + 'a>) -> Arc<Self> {
        Arc::new(Self {
            shape: shape.clone(),
            material: material.clone(),
        })
    }
}

impl<'a> Primitive for AggregatePrimitive<'a> {
    fn intersect(
        &self,
        ray: &Ray,
        t_hit: &mut Float,
        interaction: &mut SurfaceInteraction,
    ) -> bool {
        let mut hit = false;
        for primitive in self.primitives.iter() {
            if primitive.intersect(ray, t_hit, interaction) {
                hit = true;
            }
        }
        hit
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        let mut hit = false;
        for primitive in self.primitives.iter() {
            if primitive.intersect_test(ray) {
                hit = true;
            }
        }
        hit
    }
}

impl<'a> Primitive for GeometricPrimitive<'a> {
    fn intersect(
        &self,
        ray: &Ray,
        t_hit: &mut Float,
        interaction: &mut SurfaceInteraction,
    ) -> bool {
        self.shape.intersect(ray, t_hit, interaction, false)
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        self.shape.intersect_test(ray, false)
    }
}
