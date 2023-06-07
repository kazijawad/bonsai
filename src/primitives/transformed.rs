use crate::{
    base::{
        interaction::Interaction,
        light::AreaLight,
        material::{Material, TransportMode},
        primitive::Primitive,
        transform::{AnimatedTransform, Transform},
    },
    geometries::{bounds3::Bounds3, ray::Ray},
};

pub struct TransformedPrimitive {
    primitive: Box<dyn Primitive>,
    primitive_to_world: AnimatedTransform,
}

impl Primitive for TransformedPrimitive {
    fn bounds(&self) -> Bounds3 {
        self.primitive_to_world
            .motion_bounds(&self.primitive.bounds())
    }

    fn intersect(&self, ray: &mut Ray, si: &mut Interaction) -> bool {
        let mut interpolated_primitive_to_world = Transform::default();

        // Compute ray after transformation applied by primitive_to_world.
        self.primitive_to_world
            .interpolate(ray.time, &mut interpolated_primitive_to_world);
        let mut ray = ray.transform(&interpolated_primitive_to_world.inverse());
        if !self.primitive.intersect(&mut ray, si) {
            return false;
        }
        ray.t_max = ray.t_max;

        // Transform instance's intersection data to world space.
        if !interpolated_primitive_to_world.is_identity() {
            si.transform(&interpolated_primitive_to_world);
        }

        true
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        let mut interpolated_primitive_to_world = Transform::default();
        self.primitive_to_world
            .interpolate(ray.time, &mut interpolated_primitive_to_world);

        let interpolated_world_to_primitive = interpolated_primitive_to_world.inverse();
        self.primitive
            .intersect_test(&ray.transform(&interpolated_world_to_primitive))
    }

    fn compute_scattering_functions(
        &self,
        si: &mut Interaction,
        mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        self.primitive
            .compute_scattering_functions(si, mode, allow_multiple_lobes)
    }

    fn material(&self) -> Option<&dyn Material> {
        self.primitive.material()
    }

    fn area_light(&self) -> Option<&dyn AreaLight> {
        self.primitive.area_light()
    }
}
