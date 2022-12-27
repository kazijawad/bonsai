use crate::{
    geometries::{bounds3::Bounds3, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    interaction::{Interaction, SurfaceInteraction},
    utils::math::Float,
};

pub trait Shape<'a>: Send + Sync {
    fn object_bound(&self) -> Bounds3;
    fn world_bound(&self) -> Bounds3;

    fn intersect(
        &self,
        ray: &Ray,
        t_hit: &mut Float,
        interaction: &mut SurfaceInteraction<'a>,
        include_alpha: bool,
    ) -> bool;
    fn intersect_test(&self, ray: &Ray, include_alpha: bool) -> bool;

    fn sample(&self, u: &Point2, pdf: &mut Float) -> Box<dyn Interaction>;
    fn sample_from_ref(
        &self,
        reference: Box<dyn Interaction>,
        u: &Point2,
        pdf: &mut Float,
    ) -> Box<dyn Interaction>;

    fn pdf(&self, _interaction: Box<dyn Interaction>) -> Float {
        1.0 / self.area()
    }
    fn pdf_from_ref(&self, reference: Box<dyn Interaction>, wi: &Vec3) -> Float;

    fn area(&self) -> Float;
    fn solid_angle(&self, p: &Point3, n_samples: u32) -> Float;
}
