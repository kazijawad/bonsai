use std::sync::Arc;

use crate::{
    base::{
        material::{Material, TransportMode},
        primitive::Primitive,
    },
    interactions::surface::SurfaceInteraction,
    light::AreaLight,
};

pub trait Aggregate<'a>: Primitive<'a> + Send + Sync {
    fn get_area_light(&self) -> Option<Arc<AreaLight>> {
        panic!("Aggregate::get_area_light should not be called")
    }

    fn get_material(&self) -> Option<Arc<dyn Material>> {
        panic!("Aggregate::get_material should not be called")
    }

    fn compute_scattering_functions(
        &self,
        interaction: &mut SurfaceInteraction,
        transport_mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        panic!("Aggregate::compute_scattering_function should not be called")
    }
}
