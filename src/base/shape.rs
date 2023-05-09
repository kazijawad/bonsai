use crate::{
    base::{constants::Float, interaction::Interaction},
    geometries::{
        bounds3::Bounds3, normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3,
    },
    interactions::{base::BaseInteraction, surface::SurfaceInteraction},
    utils::discrepancy::radical_inverse,
};

pub trait Shape: Send + Sync {
    fn object_bound(&self) -> Bounds3;

    fn world_bound(&self) -> Bounds3;

    fn intersect(&self, ray: &Ray, t_hit: &mut Float, si: &mut SurfaceInteraction) -> bool;

    fn intersect_test(&self, ray: &Ray) -> bool;

    fn sample(&self, u: &Point2F, pdf: &mut Float) -> Box<dyn Interaction>;

    fn sample_from_ref(
        &self,
        reference: &dyn Interaction,
        u: &Point2F,
        pdf: &mut Float,
    ) -> Box<dyn Interaction> {
        let interaction = self.sample(u, pdf);

        let mut wi = interaction.position() - reference.position();
        if wi.length_squared() == 0.0 {
            *pdf = 0.0;
        } else {
            wi = wi.normalize();
            // Convert from area measure to solid angle measure.
            *pdf = reference
                .position()
                .distance_squared(&interaction.position())
                / interaction.normal().abs_dot(&Normal::from(-wi));
            if pdf.is_infinite() {
                *pdf = 0.0;
            }
        }

        interaction
    }

    fn pdf(&self, _interaction: &dyn Interaction) -> Float {
        1.0 / self.area()
    }

    fn pdf_from_ref(&self, it: &dyn Interaction, wi: &Vec3) -> Float {
        let ray = it.spawn_ray(wi);
        let mut t_hit = 0.0;

        let mut si = SurfaceInteraction::default();
        if !self.intersect(&ray, &mut t_hit, &mut si) {
            return 0.0;
        }

        // Convert light sample weight to solid angle measure.
        let mut pdf = it.position().distance_squared(&si.base.p)
            / (si.base.n.abs_dot(&Normal::from(-wi)) * self.area());
        if pdf.is_infinite() {
            pdf = 0.0;
        }

        pdf
    }

    fn area(&self) -> Float;

    fn solid_angle(&self, p: &Point3, num_samples: u32) -> Float {
        let mut it = Box::new(BaseInteraction::new(p, 0.0));
        it.wo.z = 1.0;

        let mut solid_angle = 0.0;
        for i in 0..(num_samples as u64) {
            let u = Point2F::new(radical_inverse(0, i), radical_inverse(1, i));

            let mut pdf = 0.0;
            let shape_point = self.sample_from_ref(it.as_ref(), &u, &mut pdf);

            if pdf > 0.0
                && !self.intersect_test(&Ray::new(p, &(&shape_point.position() - p), 0.999, 0.0))
            {
                solid_angle += 1.0 / pdf;
            }
        }

        solid_angle / num_samples as Float
    }
}
