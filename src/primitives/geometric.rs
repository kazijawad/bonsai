use crate::{
    base::{
        constants::Float,
        light::AreaLight,
        material::{Material, TransportMode},
        primitive::Primitive,
        shape::Shape,
    },
    geometries::{bounds3::Bounds3, ray::Ray},
    interactions::surface::SurfaceInteraction,
};

pub struct GeometricPrimitive<'a> {
    pub shape: &'a (dyn Shape + 'a),
    pub material: &'a (dyn Material + 'a),
    pub area_light: Option<&'a (dyn AreaLight + 'a)>,
}

impl<'a> Primitive<'a> for GeometricPrimitive<'a> {
    fn world_bound(&self) -> Bounds3 {
        self.shape.world_bound()
    }

    fn intersect(&'a self, ray: &mut Ray, si: &mut SurfaceInteraction<'a>) -> bool {
        let mut t_hit: Float = 0.0;
        if !self.shape.intersect(ray, &mut t_hit, si) {
            return false;
        }
        ray.t_max = t_hit;
        si.primitive = Some(self);
        true
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        self.shape.intersect_test(ray)
    }

    fn compute_scattering_functions(
        &self,
        si: &mut SurfaceInteraction,
        mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        self.material
            .compute_scattering_functions(si, mode, allow_multiple_lobes);
    }

    fn material(&self) -> Option<&dyn Material> {
        Some(self.material)
    }

    fn area_light(&self) -> Option<&dyn AreaLight> {
        self.area_light
    }
}
