use crate::{
    efloat::EFloat,
    geometries::{
        bounds3::Bounds3, normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3,
    },
    interaction::{Interaction, SurfaceInteraction},
    math::{clamp, gamma, Float, PI},
    shape::Shape,
    transform::Transform,
};

pub struct Sphere {
    pub object_transform: Box<Transform>,
    pub world_transform: Box<Transform>,
    pub reverse_orientation: bool,
    pub transform_swaps_handedness: bool,
    pub radius: Float,
    pub z_min: Float,
    pub z_max: Float,
    pub theta_min: Float,
    pub theta_max: Float,
    pub phi_max: Float,
}

impl Sphere {
    pub fn new(
        object_transform: &Transform,
        world_transform: &Transform,
        reverse_orientation: bool,
        radius: Float,
        z_min: Float,
        z_max: Float,
        phi_max: Float,
    ) -> Self {
        let transform_swaps_handedness = object_transform.swaps_handedness();

        Self {
            object_transform: Box::new(object_transform.clone()),
            world_transform: Box::new(world_transform.clone()),
            reverse_orientation,
            transform_swaps_handedness,
            radius,
            z_min: z_min.min(z_max).clamp(-radius, radius),
            z_max: z_min.max(z_max).clamp(-radius, radius),
            theta_min: (z_min.min(z_max) / radius).clamp(-1.0, 1.0).acos(),
            theta_max: (z_min.max(z_max) / radius).clamp(-1.0, 1.0).acos(),
            phi_max: phi_max.clamp(0.0, 360.0).to_radians(),
        }
    }
}

impl Shape for Sphere {
    fn object_bound(&self) -> Bounds3 {
        Bounds3::new(
            &Point3::new(-self.radius, -self.radius, self.z_min),
            &Point3::new(self.radius, self.radius, self.z_max),
        )
    }

    fn world_bound(&self) -> Bounds3 {
        self.object_transform.transform_bounds(&self.object_bound())
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
        let ray = self.world_transform.transform_ray_with_error(
            r,
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
        let a = dx * dx + dy * dy + dz * dz;
        let b = 2.0 * (dx * ox + dy * oy + dz * oz);
        let c = ox * ox + oy * oy + oz * oz
            - EFloat::new(self.radius, 0.0) * EFloat::new(self.radius, 0.0);

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

        // Compute sphere hit position and phi.
        let mut p_hit = ray.at(Float::from(t_shape_hit));

        // Refine sphere intersection point.
        p_hit *= self.radius / p_hit.distance(&Point3::default());
        if p_hit.x == 0.0 && p_hit.y == 0.0 {
            p_hit.x = 1e-5 * self.radius;
        }

        let mut phi = p_hit.y.atan2(p_hit.x);
        if phi < 0.0 {
            phi += 2.0 * PI;
        }

        // Test sphere intersection against clipping parameters.
        if (self.z_min > -self.radius && p_hit.z < self.z_min)
            || (self.z_max < self.radius && p_hit.z > self.z_max)
            || phi > self.phi_max
        {
            if t_shape_hit == t1 {
                return false;
            }
            if t1.upper_bound() > ray.t_max {
                return false;
            }

            t_shape_hit = t1;
            // Recompute sphere hit position and phi.
            p_hit = ray.at(Float::from(t_shape_hit));

            // Refine sphere intersection point.
            p_hit *= self.radius / p_hit.distance(&Point3::default());
            if p_hit.x == 0.0 && p_hit.y == 0.0 {
                p_hit.x = 1e-5 * self.radius;
            }
            phi = p_hit.y.atan2(p_hit.x);
            if phi < 0.0 {
                phi += 2.0 * PI;
            }
            if (self.z_min > -self.radius && p_hit.z < self.z_min)
                || (self.z_max < self.radius && p_hit.z > self.z_max)
                || phi > self.phi_max
            {
                return false;
            }
        }

        // Find parametric representation of sphere hit.
        let u = phi / self.phi_max;
        let theta = clamp(p_hit.z / self.radius, -1.0, 1.0).acos();
        let v = (theta - self.theta_min) / (self.theta_max - self.theta_min);

        // Compute sphere UV derivatives.
        let z_radius = (p_hit.x * p_hit.x + p_hit.y * p_hit.y).sqrt();
        let inverted_z_radius = 1.0 / z_radius;
        let cos_phi = p_hit.x * inverted_z_radius;
        let sin_phi = p_hit.y * inverted_z_radius;
        let dpdu = Vec3::new(-self.phi_max * p_hit.y, self.phi_max * p_hit.x, 0.0);
        let dpdv = (self.theta_max - self.theta_min)
            * Vec3::new(
                p_hit.z * cos_phi,
                p_hit.z * sin_phi,
                -self.radius * theta.sin(),
            );

        // Compute sphere normal derivatives.
        let d2pduu = -self.phi_max * self.phi_max * Vec3::new(p_hit.x, p_hit.y, 0.0);
        let d2pduv = (self.theta_max - self.theta_min)
            * p_hit.z
            * self.phi_max
            * Vec3::new(-sin_phi, cos_phi, 0.0);
        let d2pdvv = -(self.theta_max - self.theta_min)
            * (self.theta_max - self.theta_min)
            * Vec3::new(p_hit.x, p_hit.y, p_hit.z);

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

        // Compute error bounds for sphere intersection.
        let p_error = gamma(5.0) * Vec3::from(p_hit).abs();

        // Initialize interaction from parametric information.
        *interaction =
            self.object_transform
                .transform_surface_interaction(&SurfaceInteraction::new(
                    p_hit,
                    p_error,
                    Point2::new(u, v),
                    -ray.direction,
                    dpdu,
                    dpdv,
                    dndu,
                    dndv,
                    ray.time,
                    0,
                    self.reverse_orientation,
                    self.transform_swaps_handedness,
                ));

        // Update hit for quadric intersection.
        *t_hit = Float::from(t_shape_hit);

        true
    }

    fn intersect_test(&self, r: &Ray, _include_alpha: bool) -> bool {
        // Transform ray to object space.
        let mut origin_error = Vec3::default();
        let mut direction_error = Vec3::default();
        let ray = self.world_transform.transform_ray_with_error(
            r,
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
        let a = dx * dx + dy * dy + dz * dz;
        let b = 2.0 * (dx * ox + dy * oy + dz * oz);
        let c = ox * ox + oy * oy + oz * oz
            - EFloat::new(self.radius, 0.0) * EFloat::new(self.radius, 0.0);

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

        // Compute sphere hit position and phi.
        let mut p_hit = ray.at(Float::from(t_shape_hit));

        // Refine sphere intersection point.
        p_hit *= self.radius / p_hit.distance(&Point3::default());
        if p_hit.x == 0.0 && p_hit.y == 0.0 {
            p_hit.x = 1e-5 * self.radius;
        }

        let mut phi = p_hit.y.atan2(p_hit.x);
        if phi < 0.0 {
            phi += 2.0 * PI;
        }

        // Test sphere intersection against clipping parameters.
        if (self.z_min > -self.radius && p_hit.z < self.z_min)
            || (self.z_max < self.radius && p_hit.z > self.z_max)
            || phi > self.phi_max
        {
            if t_shape_hit == t1 {
                return false;
            }
            if t1.upper_bound() > ray.t_max {
                return false;
            }

            t_shape_hit = t1;
            // Recompute sphere hit position and phi.
            p_hit = ray.at(Float::from(t_shape_hit));

            // Refine sphere intersection point.
            p_hit *= self.radius / p_hit.distance(&Point3::default());
            if p_hit.x == 0.0 && p_hit.y == 0.0 {
                p_hit.x = 1e-5 * self.radius;
            }
            phi = p_hit.y.atan2(p_hit.x);
            if phi < 0.0 {
                phi += 2.0 * PI;
            }
            if (self.z_min > -self.radius && p_hit.z < self.z_min)
                || (self.z_max < self.radius && p_hit.z > self.z_max)
                || phi > self.phi_max
            {
                return false;
            }
        }

        true
    }

    fn sample(&self, u: &Point2, pdf: &mut Float) -> Box<dyn Interaction> {
        todo!()
    }

    fn sample_from_ref(
        &self,
        reference: Box<dyn Interaction>,
        u: &Point2,
        pdf: &mut Float,
    ) -> Box<dyn Interaction> {
        todo!()
    }

    fn pdf(&self, interaction: Box<dyn Interaction>) -> Float {
        todo!()
    }

    fn pdf_from_ref(&self, reference: Box<dyn Interaction>, wi: &Vec3) -> Float {
        todo!()
    }

    fn area(&self) -> Float {
        self.phi_max * self.radius * (self.z_max - self.z_min)
    }

    fn solid_angle(&self, p: &Point3, n_samples: u32) -> Float {
        todo!()
    }
}
