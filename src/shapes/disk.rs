use crate::{
    geometries::{
        bounds3::Bounds3, normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3,
    },
    interaction::{Interaction, SurfaceInteraction},
    math::{Float, PI},
    shape::Shape,
    transform::Transform,
};

pub struct Disk {
    object_to_world: Box<Transform>,
    world_to_object: Box<Transform>,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
    height: Float,
    radius: Float,
    inner_radius: Float,
    phi_max: Float,
}

impl Disk {
    pub fn new(
        object_to_world: &Transform,
        world_to_object: &Transform,
        reverse_orientation: bool,
        height: Float,
        radius: Float,
        inner_radius: Float,
        phi_max: Float,
    ) -> Self {
        let transform_swaps_handedness = object_to_world.swaps_handedness();

        Self {
            object_to_world: Box::new(object_to_world.clone()),
            world_to_object: Box::new(world_to_object.clone()),
            reverse_orientation,
            transform_swaps_handedness,
            height,
            radius,
            inner_radius,
            phi_max: phi_max.clamp(0.0, 360.0).to_radians(),
        }
    }
}

impl Shape for Disk {
    fn object_bound(&self) -> Bounds3 {
        Bounds3::new(
            &Point3::new(-self.radius, -self.radius, self.height),
            &Point3::new(self.radius, self.radius, self.height),
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
        let ray = self.world_to_object.transform_ray_with_error(
            r,
            &mut origin_error,
            &mut direction_error,
        );

        // Reject disk intersections for rays parallel to the disk's plane.
        if ray.direction.z == 0.0 {
            return false;
        }
        let t_shape_hit = (self.height - ray.origin.z) / ray.direction.z;
        if t_shape_hit <= 0.0 || t_shape_hit >= ray.t_max {
            return false;
        }

        // Check if hit point is inside disk radii and max phi.
        let mut p_hit = ray.at(t_shape_hit);
        let dist = p_hit.x * p_hit.x + p_hit.y * p_hit.y;
        if dist > self.radius * self.radius || dist < self.inner_radius * self.inner_radius {
            return false;
        }

        // Test disk phi value against max phi.
        let mut phi = p_hit.y.atan2(p_hit.x);
        if phi < 0.0 {
            phi += 2.0 * PI;
        }
        if phi > self.phi_max {
            return false;
        }

        // Find parametric representation of disk hit.
        let u = phi / self.phi_max;
        let r_hit = dist.sqrt();
        let v = (self.radius - r_hit) / (self.radius - self.inner_radius);
        let dpdu = Vec3::new(-self.phi_max * p_hit.y, self.phi_max * p_hit.x, 0.0);
        let dpdv = Vec3::new(p_hit.x, p_hit.y, 0.0) * (self.inner_radius - self.radius) / r_hit;
        let dndu = Normal::default();
        let dndv = Normal::default();

        // Refine disk intersection point.
        p_hit.z = self.height;

        // Compute error bounds for disk intersection.
        let p_error = Vec3::default();

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

        // Reject disk intersections for rays parallel to the disk's plane.
        if ray.direction.z == 0.0 {
            return false;
        }
        let t_shape_hit = (self.height - ray.origin.z) / ray.direction.z;
        if t_shape_hit <= 0.0 || t_shape_hit >= ray.t_max {
            return false;
        }

        // Check if hit point is inside disk radii and max phi.
        let p_hit = ray.at(t_shape_hit);
        let dist = p_hit.x * p_hit.x + p_hit.y * p_hit.y;
        if dist > self.radius * self.radius || dist < self.inner_radius * self.inner_radius {
            return false;
        }

        // Test disk phi value against max phi.
        let mut phi = p_hit.y.atan2(p_hit.x);
        if phi < 0.0 {
            phi += 2.0 * PI;
        }
        if phi > self.phi_max {
            return false;
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
        self.phi_max * 0.5 * (self.radius * self.radius - self.inner_radius * self.inner_radius)
    }

    fn solid_angle(&self, p: &Point3, n_samples: u32) -> Float {
        todo!()
    }
}
