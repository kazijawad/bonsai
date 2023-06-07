use crate::{
    base::{
        interaction::Interaction, light::AreaLight, material::Material, material::TransportMode,
    },
    geometries::{bounds3::Bounds3, ray::Ray},
};

pub trait Primitive: Send + Sync {
    fn bounds(&self) -> Bounds3;

    fn intersect(&self, ray: &mut Ray, it: &mut Interaction) -> bool;
    fn intersect_test(&self, ray: &Ray) -> bool;

    fn compute_scattering_functions(
        &self,
        it: &mut Interaction,
        transport_mode: TransportMode,
        allow_multiple_lobes: bool,
    );

    fn material(&self) -> Option<&dyn Material>;
    fn area_light(&self) -> Option<&dyn AreaLight>;
}
