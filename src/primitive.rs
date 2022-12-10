use crate::{
    geometries::ray::Ray, interaction::SurfaceInteraction, material::Material, math::Float,
    shape::Shape,
};

pub trait Primitive: Send + Sync {
    fn intersect(&self, ray: &Ray, t_hit: &mut Float, interaction: &mut SurfaceInteraction)
        -> bool;
}

pub struct AggregatePrimitive {
    primitives: Vec<Box<dyn Primitive>>,
}

pub struct GeometricPrimitive {
    pub shape: Box<dyn Shape>,
    pub material: Box<dyn Material>,
}

impl AggregatePrimitive {
    pub fn new() -> Self {
        Self { primitives: vec![] }
    }

    pub fn add<T: Primitive + 'static>(&mut self, object: T) {
        self.primitives.push(Box::new(object));
    }
}

impl GeometricPrimitive {
    pub fn new<T: Shape + 'static, U: Material + 'static>(shape: T, material: U) -> Self {
        Self {
            shape: Box::new(shape),
            material: Box::new(material),
        }
    }
}

impl Primitive for AggregatePrimitive {
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
}

impl Primitive for GeometricPrimitive {
    fn intersect(
        &self,
        ray: &Ray,
        t_hit: &mut Float,
        interaction: &mut SurfaceInteraction,
    ) -> bool {
        self.shape.intersect(ray, t_hit, interaction, false)
    }
}
