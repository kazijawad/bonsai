use std::sync::Arc;

use crate::{
    geometries::{bounds3::Bounds3, ray::Ray},
    interaction::SurfaceInteraction,
    light::AreaLight,
    material::Material,
};

pub trait Primitive<'a>: Send + Sync {
    fn world_bound(&self) -> Bounds3;

    fn intersect(&self, ray: &mut Ray, interaction: &mut SurfaceInteraction<'a>) -> bool;
    fn intersect_test(&self, ray: &Ray) -> bool;

    fn area_light(&self) -> Option<Arc<AreaLight>>;

    fn get_material(&self) -> Option<Arc<dyn Material + 'a>>;

    fn compute_scattering_function(&self, interaction: &mut SurfaceInteraction);
}

pub struct AggregatePrimitive<'a> {
    primitives: Vec<Arc<dyn Primitive<'a> + 'a>>,
}

impl<'a> AggregatePrimitive<'a> {
    pub fn new(primitives: Vec<Arc<dyn Primitive<'a> + 'a>>) -> Arc<Self> {
        Arc::new(Self { primitives })
    }
}

impl<'a> Primitive<'a> for AggregatePrimitive<'a> {
    fn world_bound(&self) -> Bounds3 {
        todo!()
    }

    fn intersect(&self, ray: &mut Ray, interaction: &mut SurfaceInteraction) -> bool {
        todo!()
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        todo!()
    }

    fn area_light(&self) -> Option<Arc<AreaLight>> {
        todo!()
    }

    fn get_material(&self) -> Option<Arc<dyn Material>> {
        todo!()
    }

    fn compute_scattering_function(&self, interaction: &mut SurfaceInteraction) {
        todo!()
    }
}
