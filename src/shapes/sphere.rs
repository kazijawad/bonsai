use crate::{
    base::{
        constants::{Float, PI},
        interaction::Interaction,
        sampling::{uniform_cone_pdf, uniform_sample_sphere},
        shape::Shape,
        transform::Transform,
    },
    geometries::{
        bounds3::Bounds3, normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3,
    },
    interactions::{base::BaseInteraction, surface::SurfaceInteraction},
    utils::{efloat::EFloat, math::gamma},
};

pub struct Sphere {
    object_to_world: Transform,
    world_to_object: Transform,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
    radius: Float,
    z_min: Float,
    z_max: Float,
    theta_min: Float,
    theta_max: Float,
    phi_max: Float,
}

pub struct SphereOptions {
    pub transform: Transform,
    pub reverse_orientation: bool,
    pub radius: Float,
    pub z_min: Float,
    pub z_max: Float,
    pub phi_max: Float,
}

impl Sphere {
    pub fn new(opts: SphereOptions) -> Self {
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
            z_min: opts.z_min.min(opts.z_max).clamp(-opts.radius, opts.radius),
            z_max: opts.z_min.max(opts.z_max).clamp(-opts.radius, opts.radius),
            theta_min: (opts.z_min.min(opts.z_max) / opts.radius)
                .clamp(-1.0, 1.0)
                .acos(),
            theta_max: (opts.z_min.max(opts.z_max) / opts.radius)
                .clamp(-1.0, 1.0)
                .acos(),
            phi_max: opts.phi_max.clamp(0.0, 360.0).to_radians(),
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
        self.object_to_world.transform_bounds(&self.object_bound())
    }

    fn intersect(&self, ray: &Ray, t_hit: &mut Float, si: &mut SurfaceInteraction) -> bool {
        // Transform ray to object space.
        let mut o_error = Vec3::default();
        let mut d_error = Vec3::default();
        let ray = ray.transform_with_error(&self.world_to_object, &mut o_error, &mut d_error);

        // Initialize ray coordinate values.
        let ox = EFloat::new(ray.origin.x, o_error.x);
        let oy = EFloat::new(ray.origin.y, o_error.y);
        let oz = EFloat::new(ray.origin.z, o_error.z);

        let dx = EFloat::new(ray.direction.x, d_error.x);
        let dy = EFloat::new(ray.direction.y, d_error.y);
        let dz = EFloat::new(ray.direction.z, d_error.z);

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

            // Recompute sphere hit position and phi.
            t_shape_hit = t1;
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
        let theta = (p_hit.z / self.radius).clamp(-1.0, 1.0).acos();
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
        *si = SurfaceInteraction::new(
            p_hit,
            p_error,
            Point2F::new(u, v),
            -ray.direction,
            dpdu,
            dpdv,
            dndu,
            dndv,
            ray.time,
            self.reverse_orientation,
            self.transform_swaps_handedness,
        );
        si.transform(&self.object_to_world);

        // Update hit for quadric intersection.
        *t_hit = Float::from(t_shape_hit);

        true
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        // Transform ray to object space.
        let mut o_error = Vec3::default();
        let mut d_error = Vec3::default();
        let ray = ray.transform_with_error(&self.world_to_object, &mut o_error, &mut d_error);

        // Initialize ray coordinate values.
        let ox = EFloat::new(ray.origin.x, o_error.x);
        let oy = EFloat::new(ray.origin.y, o_error.y);
        let oz = EFloat::new(ray.origin.z, o_error.z);

        let dx = EFloat::new(ray.direction.x, d_error.x);
        let dy = EFloat::new(ray.direction.y, d_error.y);
        let dz = EFloat::new(ray.direction.z, d_error.z);

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

    fn sample(&self, u: &Point2F, pdf: &mut Float) -> Box<dyn Interaction> {
        let mut object = Point3::default() + self.radius * uniform_sample_sphere(u);

        let mut n = Normal::from(object).transform(&self.object_to_world);
        if self.reverse_orientation {
            n *= -1.0;
        }

        // Reproject to sphere surface and compute error.
        object *= self.radius / object.distance(&Point3::default());
        let object_error = gamma(5.0) * Vec3::from(object.abs());

        let mut p_error = Vec3::default();
        let p =
            object.transform_with_point_error(&self.object_to_world, &object_error, &mut p_error);

        *pdf = 1.0 / self.area();

        let mut it = Box::new(BaseInteraction::new(&p, 0.0));
        it.n = n;
        it.p_error = p_error;

        it
    }

    fn sample_from_ref(
        &self,
        _it: &dyn Interaction,
        _u: &Point2F,
        _pdf: &mut Float,
    ) -> Box<dyn Interaction> {
        unimplemented!()
    }

    fn pdf_from_ref(&self, reference: &dyn Interaction, wi: &Vec3) -> Float {
        let center = Point3::default().transform(&self.object_to_world);

        // Return uniform PDF if point is inside sphere.
        let origin = reference.position().offset_ray_origin(
            &reference.position_error(),
            &reference.normal(),
            &(center - reference.position()),
        );
        if origin.distance_squared(&center) <= self.radius * self.radius {
            return Shape::pdf_from_ref(self, reference, wi);
        }

        // Compute general sphere PDF.
        let sin_theta_2 =
            self.radius * self.radius / reference.position().distance_squared(&center);
        let cos_theta_max = Float::max(0.0, 1.0 - sin_theta_2).sqrt();

        uniform_cone_pdf(cos_theta_max)
    }

    fn area(&self) -> Float {
        self.phi_max * self.radius * (self.z_max - self.z_min)
    }

    fn solid_angle(&self, p: &Point3, _num_samples: u32) -> Float {
        let center = Point3::default().transform(&self.object_to_world);
        if p.distance_squared(&center) <= self.radius * self.radius {
            return 4.0 * PI;
        }

        let sin_theta_2 = self.radius * self.radius / p.distance_squared(&center);
        let cos_theta = Float::max(0.0, 1.0 - sin_theta_2).sqrt();

        2.0 * PI * (1.0 - cos_theta)
    }
}
