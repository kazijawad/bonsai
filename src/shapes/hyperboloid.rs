use std::{mem, sync::Arc};

use crate::{
    base::shape::Shape,
    geometries::{
        bounds3::Bounds3, normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3,
    },
    interaction::{Interaction, SurfaceInteraction},
    transform::Transform,
    utils::{
        efloat::EFloat,
        math::{Float, PI},
    },
};

pub struct Hyperboloid<'a> {
    object_to_world: &'a Transform,
    world_to_object: &'a Transform,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
    p1: Point3,
    p2: Point3,
    z_min: Float,
    z_max: Float,
    phi_max: Float,
    radius_max: Float,
    ah: Float,
    ch: Float,
}

impl<'a> Hyperboloid<'a> {
    pub fn new(
        object_to_world: &'a Transform,
        world_to_object: &'a Transform,
        reverse_orientation: bool,
        mut p1: Point3,
        mut p2: Point3,
        phi_max: Float,
    ) -> Arc<Self> {
        let transform_swaps_handedness = object_to_world.swaps_handedness();

        let radius1 = (p1.x * p1.x + p1.y * p1.y).sqrt();
        let radius2 = (p2.x * p2.x + p2.y * p2.y).sqrt();

        // Compute implicit function coefficients for hyperboloid
        if p2.z == 0.0 {
            mem::swap(&mut p1, &mut p2);
        }
        let mut pp = p1;

        let mut xy1;
        let mut xy2;
        let mut ah = Float::INFINITY;
        let mut ch = 0.0;
        while ah.is_infinite() || ah.is_nan() {
            pp += 2.0 * (p2 - p1);
            xy1 = pp.x * pp.x + pp.y * pp.y;
            xy2 = p2.x * p2.x + p2.y * p2.y;
            ah = (1.0 / xy1 - (pp.z * pp.z) / (xy1 * p2.z * p2.z))
                / (1.0 - (xy2 * pp.z * pp.z) / (xy1 * p2.z * p2.z));
            ch = (ah * xy2 - 1.0) / (p2.z * p2.z);
        }

        Arc::new(Self {
            object_to_world,
            world_to_object,
            reverse_orientation,
            transform_swaps_handedness,
            p1,
            p2,
            z_min: p1.z.min(p2.z),
            z_max: p1.z.max(p2.z),
            phi_max: phi_max.clamp(0.0, 360.0).to_radians(),
            radius_max: radius1.max(radius2),
            ah,
            ch,
        })
    }
}

impl<'a> Shape<'a> for Hyperboloid<'a> {
    fn object_bound(&self) -> Bounds3 {
        Bounds3::new(
            &Point3::new(-self.radius_max, -self.radius_max, self.z_min),
            &Point3::new(self.radius_max, self.radius_max, self.z_max),
        )
    }

    fn world_bound(&self) -> Bounds3 {
        self.object_to_world.transform_bounds(&self.object_bound())
    }

    fn intersect(
        &self,
        r: &Ray,
        t_hit: &mut Float,
        interaction: &mut SurfaceInteraction<'a>,
        _include_alpha: bool,
    ) -> bool {
        // Transform ray to object space.
        let mut origin_error = Vec3::default();
        let mut direction_error = Vec3::default();
        let ray = self.world_to_object.transform_ray_with_error(
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

        let a = self.ah * dx * dx + self.ah * dy * dy - self.ch * dz * dz;
        let b = 2.0 * (self.ah * dx * ox + self.ah * dy * oy - self.ch * dz * oz);
        let c = self.ah * ox * ox + self.ah * oy * oy - self.ch * oz * oz - 1.0;

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

        // Compute hyperboloid inverse mapping.
        let mut p_hit = ray.at(Float::from(t_shape_hit));
        let mut v = (p_hit.z - self.p1.z) / (self.p2.z - self.p1.z);
        let mut pr = (1.0 - v) * self.p1 + v * self.p2;
        let mut phi = (pr.x * p_hit.y - p_hit.x * pr.y).atan2(p_hit.x * pr.x + p_hit.y * pr.y);
        if phi < 0.0 {
            phi += 2.0 * PI;
        }

        // Test hyperboloid intersection against clipping parameters.
        if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
            if t_shape_hit == t1 {
                return false;
            }
            if t1.upper_bound() > ray.t_max {
                return false;
            }

            // Recompute hyperboloid inverse mapping.
            t_shape_hit = t1;
            p_hit = ray.at(Float::from(t_shape_hit));
            v = (p_hit.z - self.p1.z) / (self.p2.z - self.p1.z);
            pr = (1.0 - v) * self.p1 + v * self.p2;
            phi = (pr.x * p_hit.y - p_hit.x * pr.y).atan2(p_hit.x * pr.x + p_hit.y * pr.y);
            if phi < 0.0 {
                phi += 2.0 * PI;
            }

            if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
                return false;
            }
        }

        // Find parametric representation of hyperboloid hit.
        let u = phi / self.phi_max;

        // Compute hyperboloid UV derivatives.
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();
        let dpdu = Vec3::new(-self.phi_max * p_hit.y, self.phi_max * p_hit.x, 0.0);
        let dpdv = Vec3::new(
            (self.p2.x - self.p1.x) * cos_phi - (self.p2.y - self.p1.y) * sin_phi,
            (self.p2.x - self.p1.x) * sin_phi + (self.p2.y - self.p1.y) * cos_phi,
            self.p2.z - self.p1.z,
        );

        // Compute hyperboloid normal derivatives.
        let d2pduu = -self.phi_max * self.phi_max * Vec3::new(p_hit.x, p_hit.y, 0.0);
        let d2pduv = self.phi_max * Vec3::new(-dpdv.y, dpdv.x, 0.0);
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
        let p_error = Vec3::new(
            px.absolute_error(),
            py.absolute_error(),
            pz.absolute_error(),
        );

        // Initialize interaction from parametric information.
        *interaction =
            self.object_to_world
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
        let ray = self.world_to_object.transform_ray_with_error(
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

        let a = self.ah * dx * dx + self.ah * dy * dy - self.ch * dz * dz;
        let b = 2.0 * (self.ah * dx * ox + self.ah * dy * oy - self.ch * dz * oz);
        let c = self.ah * ox * ox + self.ah * oy * oy - self.ch * oz * oz - 1.0;

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

        // Compute hyperboloid inverse mapping.
        let mut p_hit = ray.at(Float::from(t_shape_hit));
        let mut v = (p_hit.z - self.p1.z) / (self.p2.z - self.p1.z);
        let mut pr = (1.0 - v) * self.p1 + v * self.p2;
        let mut phi = (pr.x * p_hit.y - p_hit.x * pr.y).atan2(p_hit.x * pr.x + p_hit.y * pr.y);
        if phi < 0.0 {
            phi += 2.0 * PI;
        }

        // Test hyperboloid intersection against clipping parameters.
        if p_hit.z < self.z_min || p_hit.z > self.z_max || phi > self.phi_max {
            if t_shape_hit == t1 {
                return false;
            }
            if t1.upper_bound() > ray.t_max {
                return false;
            }

            // Recompute hyperboloid inverse mapping.
            t_shape_hit = t1;
            p_hit = ray.at(Float::from(t_shape_hit));
            v = (p_hit.z - self.p1.z) / (self.p2.z - self.p1.z);
            pr = (1.0 - v) * self.p1 + v * self.p2;
            phi = (pr.x * p_hit.y - p_hit.x * pr.y).atan2(p_hit.x * pr.x + p_hit.y * pr.y);
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
        self.phi_max / 6.0
            * (2.0 * ((self.p1.x * self.p1.x) * (self.p1.x * self.p1.x))
                - 2.0 * self.p1.x * self.p1.x * self.p1.x * self.p2.x
                + 2.0 * ((self.p2.x * self.p2.x) * (self.p2.x * self.p2.x))
                + 2.0
                    * (self.p1.y * self.p1.y + self.p1.y * self.p2.y + self.p2.y * self.p2.y)
                    * (((self.p1.y - self.p2.y) * (self.p1.y - self.p2.y))
                        + ((self.p1.z - self.p2.z) * (self.p1.z - self.p2.z)))
                + self.p2.x
                    * self.p2.x
                    * (5.0 * self.p1.y * self.p1.y + 2.0 * self.p1.y * self.p2.y
                        - 4.0 * self.p2.y * self.p2.y
                        + 2.0 * ((self.p1.z - self.p2.z) * (self.p1.z - self.p2.z)))
                + self.p1.x
                    * self.p1.x
                    * (-4.0 * self.p1.y * self.p1.y
                        + 2.0 * self.p1.y * self.p2.y
                        + 5.0 * self.p2.y * self.p2.y
                        + 2.0 * ((self.p1.z - self.p2.z) * (self.p1.z - self.p2.z)))
                - 2.0
                    * self.p1.x
                    * self.p2.x
                    * (self.p2.x * self.p2.x - self.p1.y * self.p1.y + 5.0 * self.p1.y * self.p2.y
                        - self.p2.y * self.p2.y
                        - self.p1.z * self.p1.z
                        + 2.0 * self.p1.z * self.p2.z
                        - self.p2.z * self.p2.z))
    }

    fn solid_angle(&self, p: &Point3, n_samples: u32) -> Float {
        todo!()
    }
}
