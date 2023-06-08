use crate::{
    base::{constants::Float, interaction::Interaction},
    geometries::{bounds3::Bounds3, normal::Normal, point2::Point2F, ray::Ray, vec3::Vec3},
};

pub trait Shape: Send + Sync {
    fn object_bounds(&self) -> Bounds3;

    fn world_bounds(&self) -> Bounds3;

    fn intersect(&self, ray: &Ray, t_hit: &mut Float, it: &mut Interaction) -> bool;

    fn intersect_test(&self, ray: &Ray) -> bool;

    fn sample(&self, u: &Point2F, pdf: &mut Float) -> Interaction;

    fn sample_from_it(&self, it: &Interaction, u: &Point2F, pdf: &mut Float) -> Interaction {
        let output_it = self.sample(u, pdf);

        let mut wi = output_it.point - it.point;
        if wi.length_squared() == 0.0 {
            *pdf = 0.0;
        } else {
            wi = wi.normalize();
            // Convert from area measure to solid angle measure.
            *pdf = it.point.distance_squared(&output_it.point)
                / output_it.normal.abs_dot(&Normal::from(-wi));
            if pdf.is_infinite() {
                *pdf = 0.0;
            }
        }

        output_it
    }

    fn pdf(&self, _: &Interaction) -> Float {
        1.0 / self.area()
    }

    fn pdf_from_it(&self, it: &Interaction, wi: &Vec3) -> Float {
        let ray = it.spawn_ray(wi);

        let mut t_hit = 0.0;
        let mut intersection_it = Interaction::default();
        if !self.intersect(&ray, &mut t_hit, &mut intersection_it) {
            return 0.0;
        }

        // Convert light sample weight to solid angle measure.
        let mut pdf = it.point.distance_squared(&intersection_it.point)
            / (intersection_it.normal.abs_dot(&Normal::from(-wi)) * self.area());
        if pdf.is_infinite() {
            pdf = 0.0;
        }

        pdf
    }

    fn area(&self) -> Float;
}
