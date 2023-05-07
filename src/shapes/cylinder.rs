use crate::{
    base::{
        constants::{Float, PI},
        interaction::Interaction,
        shape::Shape,
        transform::Transform,
    },
    geometries::{
        bounds3::Bounds3, normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3,
    },
    interactions::{base::BaseInteraction, surface::SurfaceInteraction},
    utils::{
        efloat::EFloat,
        math::{gamma, lerp},
    },
};

pub struct Cylinder {
    object_to_world: Transform,
    world_to_object: Transform,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
    radius: Float,
    z_min: Float,
    z_max: Float,
    phi_max: Float,
}

pub struct CylinderOptions {
    pub transform: Transform,
    pub reverse_orientation: bool,
    pub radius: Float,
    pub z_min: Float,
    pub z_max: Float,
    pub phi_max: Float,
}

impl Cylinder {
    pub fn new(opts: CylinderOptions) -> Self {
        let object_to_world = opts.transform;
        let world_to_object = if object_to_world.is_identity() {
            object_to_world.clone()
        } else {
            object_to_world.inverse()
        };

        let transform_swaps_handedness = object_to_world.swaps_handedness();

        Self {
            object_to_world,
            world_to_object,
            reverse_orientation: opts.reverse_orientation,
            transform_swaps_handedness,
            radius: opts.radius,
            z_min: opts.z_min.min(opts.z_max),
            z_max: opts.z_min.max(opts.z_max),
            phi_max: opts.phi_max.clamp(0.0, 360.0).to_radians(),
        }
    }
}

impl Shape for Cylinder {
    fn object_bound(&self) -> Bounds3 {
        Bounds3::new(
            &Point3::new(-self.radius, -self.radius, self.z_min),
            &Point3::new(self.radius, self.radius, self.z_max),
        )
    }

    fn world_bound(&self) -> Bounds3 {
        self.object_to_world.transform_bounds(&self.object_bound())
    }

    fn intersect(
        &self,
        r: &Ray,
        t_hit: &mut Float,
        interaction: &mut SurfaceInteraction,
        _include_alpha: bool,
    ) -> bool {
        // Transform ray to object space.
        let mut origin_error = Vec3::default();
        let mut direction_error = Vec3::default();
        let ray = r.transform_with_error(
            &self.world_to_object,
            &mut origin_error,
            &mut direction_error,
        );

        // Initialize ray coordinate values.
        let ox = EFloat::new(ray.origin.x, origin_error.x);
        let oy = EFloat::new(ray.origin.y, origin_error.y);
        let dx = EFloat::new(ray.direction.x, direction_error.x);
        let dy = EFloat::new(ray.direction.y, direction_error.y);
        let a = dx * dx + dy * dy;
        let b = 2.0 * (dx * ox + dy * oy);
        let c = ox * ox + oy * oy - EFloat::new(self.radius, 0.0) * EFloat::new(self.radius, 0.0);

        // Solve quadratic equation for t values.
        let mut t0 = EFloat::default();
        let mut t1 = EFloat::default();
        if !EFloat::quadratic(a, b, c, &mut t0, &mut t1) {
            return false;
        }

        // Check quadric shape for nearest intersection.
        if t0.upper_bound() > ray.t_max || t1.lower_bound() <= 0.0 {
            return false;
        }
        let mut t_shape_hit = t0;
        if t_shape_hit.lower_bound() <= 0.0 {
            t_shape_hit = t1;
            if t_shape_hit.upper_bound() > ray.t_max {
                return false;
            }
        }

        // Compute cylinder hit position and phi.
        let mut p_hit = ray.at(Float::from(t_shape_hit));

        // Refine cylinder intersection point.
        let hit_radius = (p_hit.x * p_hit.x + p_hit.y * p_hit.y).sqrt();
        p_hit.x *= self.radius / hit_radius;
        p_hit.y *= self.radius / hit_radius;

        let mut phi = p_hit.y.atan2(p_hit.x);
        if phi < 0.0 {
            phi += 2.0 * PI
        }

        // Test cylinder intersection against clipping parameters.
        if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
            if t_shape_hit == t1 {
                return false;
            }
            if t1.upper_bound() > ray.t_max {
                return false;
            }

            // Recompute cylinder hit position and phi.
            t_shape_hit = t1;
            p_hit = ray.at(Float::from(t_shape_hit));

            // Refine cylinder intersection point.
            let hit_radius = (p_hit.x * p_hit.x + p_hit.y * p_hit.y).sqrt();
            p_hit.x *= self.radius / hit_radius;
            p_hit.y *= self.radius / hit_radius;
            phi = p_hit.y.atan2(p_hit.x);
            if phi < 0.0 {
                phi += 2.0 * PI;
            }
            if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
                return false;
            }
        }

        // Find parametric representation of cylinder hit.
        let u = phi / self.phi_max;
        let v = (p_hit.z - self.z_min) / (self.z_max - self.z_min);

        // Compute cylinder UV derivatives.
        let dpdu = Vec3::new(-self.phi_max * p_hit.y, self.phi_max * p_hit.x, 0.0);
        let dpdv = Vec3::new(0.0, 0.0, self.z_max - self.z_min);

        // Compute cylinder normal derivatives.
        let d2pduu = -self.phi_max * self.phi_max * Vec3::new(p_hit.x, p_hit.y, 0.0);
        let d2pduv = Vec3::default();
        let d2pdvv = Vec3::default();

        // Compute coefficients for fundamental forms.
        let first_e = dpdu.dot(&dpdu);
        let first_f = dpdu.dot(&dpdv);
        let first_g = dpdv.dot(&dpdv);
        let first_n = dpdu.cross(&dpdv).normalize();
        let second_e = first_n.dot(&d2pduu);
        let second_f = first_n.dot(&d2pduv);
        let second_g = first_n.dot(&d2pdvv);

        // Compute derivatives from fundamental form coefficients.
        let inverted_second_egf = 1.0 / (first_e * first_g - first_f * first_f);
        let dndu = Normal::from(
            (second_f * first_f - second_e * first_g) * inverted_second_egf * dpdu
                + (second_e * first_f - second_f * first_e) * inverted_second_egf * dpdv,
        );
        let dndv = Normal::from(
            (second_g * first_f - second_f * first_g) * inverted_second_egf * dpdu
                + (second_f * first_f - second_g * first_e) * inverted_second_egf * dpdv,
        );

        // Compute error bounds for cylinder intersection.
        let p_error = gamma(3.0) * Vec3::new(p_hit.x, p_hit.y, 0.0).abs();

        // Initialize interaction from parametric information.
        *interaction = SurfaceInteraction::new(
            p_hit,
            p_error,
            Point2::new(u, v),
            -ray.direction,
            dpdu,
            dpdv,
            dndu,
            dndv,
            ray.time,
            self.reverse_orientation,
            self.transform_swaps_handedness,
        );
        interaction.transform(&self.object_to_world);

        // Update hit for quadric intersection.
        *t_hit = Float::from(t_shape_hit);

        true
    }

    fn intersect_test(&self, r: &Ray, _include_alpha: bool) -> bool {
        // Transform ray to object space.
        let mut origin_error = Vec3::default();
        let mut direction_error = Vec3::default();
        let ray = r.transform_with_error(
            &self.world_to_object,
            &mut origin_error,
            &mut direction_error,
        );

        // Initialize ray coordinate values.
        let ox = EFloat::new(ray.origin.x, origin_error.x);
        let oy = EFloat::new(ray.origin.y, origin_error.y);
        let dx = EFloat::new(ray.direction.x, direction_error.x);
        let dy = EFloat::new(ray.direction.y, direction_error.y);
        let a = dx * dx + dy * dy;
        let b = 2.0 * (dx * ox + dy * oy);
        let c = ox * ox + oy * oy - EFloat::new(self.radius, 0.0) * EFloat::new(self.radius, 0.0);

        // Solve quadratic equation for t values.
        let mut t0 = EFloat::default();
        let mut t1 = EFloat::default();
        if !EFloat::quadratic(a, b, c, &mut t0, &mut t1) {
            return false;
        }

        // Check quadric shape for nearest intersection.
        if t0.upper_bound() > ray.t_max || t1.lower_bound() <= 0.0 {
            return false;
        }
        let mut t_shape_hit = t0;
        if t_shape_hit.lower_bound() <= 0.0 {
            t_shape_hit = t1;
            if t_shape_hit.upper_bound() > ray.t_max {
                return false;
            }
        }

        // Compute cylinder hit position and phi.
        let mut p_hit = ray.at(Float::from(t_shape_hit));

        // Refine cylinder intersection point.
        let hit_radius = (p_hit.x * p_hit.x + p_hit.y * p_hit.y).sqrt();
        p_hit.x *= self.radius / hit_radius;
        p_hit.y *= self.radius / hit_radius;

        let mut phi = p_hit.y.atan2(p_hit.x);
        if phi < 0.0 {
            phi += 2.0 * PI
        }

        // Test cylinder intersection against clipping parameters.
        if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
            if t_shape_hit == t1 {
                return false;
            }
            if t1.upper_bound() > ray.t_max {
                return false;
            }

            // Recompute cylinder hit position and phi.
            t_shape_hit = t1;
            p_hit = ray.at(Float::from(t_shape_hit));

            // Refine cylinder intersection point.
            let hit_radius = (p_hit.x * p_hit.x + p_hit.y * p_hit.y).sqrt();
            p_hit.x *= self.radius / hit_radius;
            p_hit.y *= self.radius / hit_radius;
            phi = p_hit.y.atan2(p_hit.x);
            if phi < 0.0 {
                phi += 2.0 * PI;
            }
            if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
                return false;
            }
        }

        true
    }

    fn sample(&self, u: &Point2, pdf: &mut Float) -> Box<dyn Interaction> {
        let z = lerp(u.x, self.z_min, self.z_max);
        let phi = u.y * self.phi_max;
        let mut object_point = Point3::new(self.radius * phi.cos(), self.radius * phi.sin(), z);

        let mut n =
            Normal::new(object_point.x, object_point.y, 0.0).transform(&self.object_to_world);
        if self.reverse_orientation {
            n *= -1.0;
        }

        // Reproject point to cylinder surface and compute error.
        let hit_radius = (object_point.x * object_point.x + object_point.y * object_point.y).sqrt();
        object_point.x *= self.radius / hit_radius;
        object_point.y *= self.radius / hit_radius;

        let object_point_error = gamma(3.0) * Vec3::new(object_point.x, object_point.y, 0.0).abs();
        let mut p_error = Vec3::default();
        let p = object_point.transform_with_point_error(
            &self.object_to_world,
            &object_point_error,
            &mut p_error,
        );

        *pdf = 1.0 / self.area();

        let mut it = Box::new(BaseInteraction::new(&p, 0.0));
        it.n = n;
        it.p_error = p_error;

        it
    }

    fn area(&self) -> Float {
        (self.z_max - self.z_min) * self.radius * self.phi_max
    }
}
