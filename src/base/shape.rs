use crate::{
    base::{
        constants::{Float, ONE_MINUS_EPSILON, PRIMES},
        interaction::Interaction,
    },
    geometries::{
        bounds3::Bounds3, normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3,
    },
};

pub trait Shape: Send + Sync {
    fn object_bounds(&self) -> Bounds3;

    fn world_bounds(&self) -> Bounds3;

    fn intersect(&self, ray: &Ray, t_hit: &mut Float, si: &mut Interaction) -> bool;

    fn intersect_test(&self, ray: &Ray) -> bool;

    fn sample(&self, u: &Point2F, pdf: &mut Float) -> Interaction;

    fn sample_from_ref(&self, r: &Interaction, u: &Point2F, pdf: &mut Float) -> Interaction {
        let it = self.sample(u, pdf);

        let mut wi = it.point - r.point;
        if wi.length_squared() == 0.0 {
            *pdf = 0.0;
        } else {
            wi = wi.normalize();
            // Convert from area measure to solid angle measure.
            *pdf = r.point.distance_squared(&it.point) / it.normal.abs_dot_vec(&-wi);
            if pdf.is_infinite() {
                *pdf = 0.0;
            }
        }

        it
    }

    fn pdf(&self, _: &Interaction) -> Float {
        1.0 / self.area()
    }

    fn pdf_from_ref(&self, it: &Interaction, wi: &Vec3) -> Float {
        let ray = it.spawn_ray(wi);
        let mut t_hit = 0.0;

        let mut si = Interaction::default();
        if !self.intersect(&ray, &mut t_hit, &mut si) {
            return 0.0;
        }

        // Convert light sample weight to solid angle measure.
        let mut pdf =
            it.point.distance_squared(&si.point) / (si.normal.abs_dot_vec(&-wi) * self.area());
        if pdf.is_infinite() {
            pdf = 0.0;
        }

        pdf
    }

    fn area(&self) -> Float;

    fn solid_angle(&self, p: &Point3, num_samples: u32) -> Float {
        let it = Interaction {
            point: p.clone(),
            point_error: Vec3::default(),
            time: 0.0,
            direction: Vec3::new(0.0, 0.0, 1.0),
            normal: Normal::default(),
            surface: None,
        };

        let mut solid_angle = 0.0;
        for i in 0..(num_samples as u64) {
            let u = Point2F::new(radical_inverse(0, i), radical_inverse(1, i));

            let mut pdf = 0.0;
            let sample = self.sample_from_ref(&it, &u, &mut pdf);

            if pdf > 0.0 && !self.intersect_test(&Ray::new(p, &(&sample.point - p), 0.999, 0.0)) {
                solid_angle += 1.0 / pdf;
            }
        }

        solid_angle / num_samples as Float
    }
}

fn radical_inverse(base_index: usize, mut a: u64) -> Float {
    let base = PRIMES[base_index];
    // We have to stop once reversed_digits is >= limit otherwise the
    // next digit of |a| may cause reversed_digits to overflow.
    let limit: u64 = !0 / base - base;
    let inverse_base = 1.0 / base as Float;
    let mut inverse_base_m = 1.0;
    let mut reversed_digits: u64 = 0;
    while a != 0 && reversed_digits < limit {
        // Extract least significant digit.
        let next = a / base;
        let digit = a - next * base;
        reversed_digits = reversed_digits * base + digit;
        inverse_base_m *= inverse_base;
        a = next;
    }
    Float::min(reversed_digits as Float * inverse_base_m, ONE_MINUS_EPSILON)
}
