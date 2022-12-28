use std::sync::Arc;

use crate::{
    base::material::{Material, TransportMode},
    geometries::{bounds3::Bounds3, ray::Ray},
    interactions::surface::SurfaceInteraction,
    light::AreaLight,
};

pub trait Primitive<'a>: Send + Sync {
    fn world_bound(&self) -> Bounds3;

    fn intersect(&self, ray: &mut Ray, interaction: &mut SurfaceInteraction) -> bool;
    fn intersect_test(&self, ray: &Ray) -> bool;

    fn get_area_light(&self) -> Option<Arc<AreaLight>>;

    fn get_material(&self) -> Option<Arc<dyn Material + 'a>>;

    fn compute_scattering_functions(
        &self,
        interaction: &mut SurfaceInteraction,
        transport_mode: TransportMode,
        allow_multiple_lobes: bool,
    );
}
