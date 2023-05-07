use crate::{
    base::{light::AreaLight, material::Material, material::TransportMode},
    geometries::{bounds3::Bounds3, ray::Ray},
    interactions::surface::SurfaceInteraction,
};

pub trait Primitive<'a>: Send + Sync {
    fn world_bound(&self) -> Bounds3;

    fn intersect(&'a self, ray: &mut Ray, interaction: &mut SurfaceInteraction<'a>) -> bool;
    fn intersect_test(&self, ray: &Ray) -> bool;

    fn compute_scattering_functions(
        &self,
        interaction: &mut SurfaceInteraction,
        transport_mode: TransportMode,
        allow_multiple_lobes: bool,
    );

    fn material(&self) -> Option<&dyn Material>;
    fn area_light(&self) -> Option<&dyn AreaLight>;
}
