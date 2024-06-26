use crate::{
    base::{
        constants::{Float, PI},
        efloat::EFloat,
        interaction::{Interaction, SurfaceOptions},
        shape::Shape,
        transform::Transform,
    },
    geometries::{
        bounds3::Bounds3, normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3,
    },
};

pub struct Cone {
    object_to_world: Transform,
    world_to_object: Transform,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
    height: Float,
    radius: Float,
    phi_max: Float,
}

pub struct ConeOptions {
    pub transform: Transform,
    pub reverse_orientation: bool,
    pub height: Float,
    pub radius: Float,
    pub phi_max: Float,
}

impl Cone {
    pub fn new(opts: ConeOptions) -> Self {
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
            height: opts.height,
            radius: opts.radius,
            phi_max: opts.phi_max.clamp(0.0, 360.0).to_radians(),
        }
    }
}

impl Shape for Cone {
    fn object_bounds(&self) -> Bounds3 {
        Bounds3::new(
            &Point3::new(-self.radius, -self.radius, 0.0),
            &Point3::new(self.radius, self.radius, self.height),
        )
    }

    fn world_bounds(&self) -> Bounds3 {
        self.object_to_world.transform_bounds(&self.object_bounds())
    }

    fn intersect(&self, ray: &Ray, t_hit: &mut Float, si: &mut Interaction) -> bool {
        // Transform ray to object space.
        let mut origin_error = Vec3::default();
        let mut direction_error = Vec3::default();
        let ray = ray.transform_with_error(
            &self.world_to_object,
            &mut origin_error,
            &mut direction_error,
        );

        // Initialize ray coordinate values.
        let ox = EFloat::new(ray.origin.x, origin_error.x);
        let oy = EFloat::new(ray.origin.y, origin_error.y);
        let oz = EFloat::new(ray.origin.z, origin_error.z);
        let dx = EFloat::new(ray.direction.x, direction_error.x);
        let dy = EFloat::new(ray.direction.y, direction_error.y);
        let dz = EFloat::new(ray.direction.z, direction_error.z);

        let mut k = EFloat::new(self.radius, 0.0) / EFloat::new(self.height, 0.0);
        k = k * k;

        let a = dx * dx + dy * dy - k * dz * dz;
        let b = 2.0 * (dx * ox + dy * oy - k * dz * (oz - self.height));
        let c = ox * ox + oy * oy - k * (oz - self.height) * (oz - self.height);

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

        // Compute cone inverse mapping.
        let mut point_hit = ray.at(Float::from(t_shape_hit));
        let mut phi = point_hit.y.atan2(point_hit.x);
        if phi < 0.0 {
            phi += 2.0 * PI;
        }

        // Test cone intersection against clipping parameters.
        if point_hit.z < 0.0 || point_hit.z > self.height || phi > self.phi_max {
            if t_shape_hit == t1 {
                return false;
            }
            if t1.upper_bound() > ray.t_max {
                return false;
            }

            // Recompute cone inverse mapping.
            t_shape_hit = t1;
            point_hit = ray.at(Float::from(t_shape_hit));
            phi = point_hit.y.atan2(point_hit.x);
            if phi < 0.0 {
                phi += 2.0 * PI;
            }

            if point_hit.z < 0.0 || point_hit.z > self.height || phi > self.phi_max {
                return false;
            }
        }

        // Find parametric representation of cone hit.
        let u = phi / self.phi_max;
        let v = point_hit.z / self.height;

        // Compute cone UV derivatives.
        let dpdu = Vec3::new(-self.phi_max * point_hit.y, self.phi_max * point_hit.x, 0.0);
        let dpdv = Vec3::new(
            -point_hit.x / (1.0 - v),
            -point_hit.y / (1.0 - v),
            self.height,
        );

        // Compute cone normal derivatives.
        let d2pduu = -self.phi_max * self.phi_max * Vec3::new(point_hit.x, point_hit.y, 0.0);
        let d2pduv = self.phi_max / (1.0 - v) * Vec3::new(point_hit.y, -point_hit.x, 0.0);
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

        // Compute error bounds for intersection computed with ray equation.
        let px = ox + t_shape_hit * dx;
        let py = oy + t_shape_hit * dy;
        let pz = oz + t_shape_hit * dz;
        let point_error = Vec3::new(
            px.absolute_error(),
            py.absolute_error(),
            pz.absolute_error(),
        );

        // Initialize interaction from parametric information.
        *si = Interaction::new(
            point_hit,
            point_error,
            ray.time,
            -ray.direction,
            None,
            Some(SurfaceOptions {
                uv: Point2F::new(u, v),
                dpdu,
                dpdv,
                dndu,
                dndv,
                reverse_orientation: self.reverse_orientation,
                transform_swaps_handedness: self.transform_swaps_handedness,
            }),
        );
        si.transform(&self.object_to_world);

        // Update hit for quadric intersection.
        *t_hit = Float::from(t_shape_hit);

        true
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        // Transform ray to object space.
        let mut origin_error = Vec3::default();
        let mut direction_error = Vec3::default();
        let ray = ray.transform_with_error(
            &self.world_to_object,
            &mut origin_error,
            &mut direction_error,
        );

        // Initialize ray coordinate values.
        let ox = EFloat::new(ray.origin.x, origin_error.x);
        let oy = EFloat::new(ray.origin.y, origin_error.y);
        let oz = EFloat::new(ray.origin.z, origin_error.z);
        let dx = EFloat::new(ray.direction.x, direction_error.x);
        let dy = EFloat::new(ray.direction.y, direction_error.y);
        let dz = EFloat::new(ray.direction.z, direction_error.z);

        let mut k = EFloat::new(self.radius, 0.0) / EFloat::new(self.height, 0.0);
        k = k * k;

        let a = dx * dx + dy * dy - k * dz * dz;
        let b = 2.0 * (dx * ox + dy * oy - k * dz * (oz - self.height));
        let c = ox * ox + oy * oy - k * (oz - self.height) * (oz - self.height);

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

        // Compute cone inverse mapping.
        let mut p_hit = ray.at(Float::from(t_shape_hit));
        let mut phi = p_hit.y.atan2(p_hit.x);
        if phi < 0.0 {
            phi += 2.0 * PI;
        }

        // Test cone intersection against clipping parameters.
        if p_hit.z < 0.0 || p_hit.z > self.height || phi > self.phi_max {
            if t_shape_hit == t1 {
                return false;
            }
            if t1.upper_bound() > ray.t_max {
                return false;
            }

            // Recompute cone inverse mapping.
            t_shape_hit = t1;
            p_hit = ray.at(Float::from(t_shape_hit));
            phi = p_hit.y.atan2(p_hit.x);
            if phi < 0.0 {
                phi += 2.0 * PI;
            }

            if p_hit.z < 0.0 || p_hit.z > self.height || phi > self.phi_max {
                return false;
            }
        }

        true
    }

    fn sample(&self, _u: &Point2F, _pdf: &mut Float) -> Interaction {
        unimplemented!();
    }

    fn area(&self) -> Float {
        self.radius
            * ((self.height * self.height) + (self.radius * self.radius)).sqrt()
            * self.phi_max
            / 2.0
    }
}
