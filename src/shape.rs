use std::sync::Arc;

use crate::{
    geometries::{bounds3::Bounds3, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    interaction::{Interaction, SurfaceInteraction},
    math::Float,
    transform::Transform,
};

pub trait Shape: Send + Sync {
    fn object_bound(&self) -> Bounds3;
    fn world_bound(&self) -> Bounds3;

    fn intersect(
        &self,
        ray: &Ray,
        t_hit: &mut Float,
        surface_interaction: &mut SurfaceInteraction,
        test_alpha_texture: bool,
    ) -> bool;
    fn intersect_occurs(&self, ray: &Ray, test_alpha_texture: bool) -> bool;

    fn sample(&self, u: &Point2, pdf: &mut Float) -> Arc<dyn Interaction + Send + Sync>;
    fn pdf(&self, _interaction: &dyn Interaction) -> Float {
        1.0 / self.area()
    }

    fn sample_from_ref(
        &self,
        reference: Arc<dyn Interaction + Send + Sync>,
        u: &Point2,
        pdf: &mut Float,
    ) -> Arc<dyn Interaction + Send + Sync>;
    fn pdf_from_ref(&self, reference: &dyn Interaction, wi: &Vec3) -> Float;

    fn area(&self) -> Float;
    fn solid_angle(&self, p: &Point3, n_samples: u32) -> Float;

    fn reverse_orientation(&self) -> bool;
    fn transform_swaps_handedness(&self) -> bool;
}

pub struct ShapeProperties {
    pub object_to_world: Arc<Transform>,
    pub world_to_object: Arc<Transform>,
    pub reverse_orientation: bool,
    pub transform_swaps_handedness: bool,
}
