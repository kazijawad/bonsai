use crate::{
    base::material::TransportMode,
    geometries::{bounds3::Bounds3, ray::Ray},
    interactions::surface::SurfaceInteraction,
};

pub trait Primitive: Send + Sync {
    fn world_bound(&self) -> Bounds3;

    fn intersect(&self, ray: &mut Ray, interaction: &mut SurfaceInteraction) -> bool;
    fn intersect_test(&self, ray: &Ray) -> bool;

    fn compute_scattering_functions(
        &self,
        interaction: &mut SurfaceInteraction,
        transport_mode: TransportMode,
        allow_multiple_lobes: bool,
    );
}
