use crate::{
    base::{
        constants::{Float, PI},
        interaction::{Interaction, SurfaceOptions},
        sampling::concentric_sample_disk,
        shape::Shape,
        transform::Transform,
    },
    geometries::{
        bounds3::Bounds3, normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3,
    },
};

pub struct Disk {
    object_to_world: Transform,
    world_to_object: Transform,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
    height: Float,
    radius: Float,
    inner_radius: Float,
    phi_max: Float,
}

pub struct DiskOptions {
    pub transform: Transform,
    pub reverse_orientation: bool,
    pub height: Float,
    pub radius: Float,
    pub inner_radius: Float,
    pub phi_max: Float,
}

impl Disk {
    pub fn new(opts: DiskOptions) -> Self {
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
            inner_radius: opts.inner_radius,
            phi_max: opts.phi_max.clamp(0.0, 360.0).to_radians(),
        }
    }
}

impl Shape for Disk {
    fn object_bounds(&self) -> Bounds3 {
        Bounds3::new(
            &Point3::new(-self.radius, -self.radius, self.height),
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

        // Reject disk intersections for rays parallel to the disk's plane.
        if ray.direction.z == 0.0 {
            return false;
        }
        let t_shape_hit = (self.height - ray.origin.z) / ray.direction.z;
        if t_shape_hit <= 0.0 || t_shape_hit >= ray.t_max {
            return false;
        }

        // Check if hit point is inside disk radii and max phi.
        let mut point_hit = ray.at(t_shape_hit);
        let dist = point_hit.x * point_hit.x + point_hit.y * point_hit.y;
        if dist > self.radius * self.radius || dist < self.inner_radius * self.inner_radius {
            return false;
        }

        // Test disk phi value against max phi.
        let mut phi = point_hit.y.atan2(point_hit.x);
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
        let dpdu = Vec3::new(-self.phi_max * point_hit.y, self.phi_max * point_hit.x, 0.0);
        let dpdv =
            Vec3::new(point_hit.x, point_hit.y, 0.0) * (self.inner_radius - self.radius) / r_hit;
        let dndu = Normal::default();
        let dndv = Normal::default();

        // Refine disk intersection point.
        point_hit.z = self.height;

        // Compute error bounds for disk intersection.
        let point_error = Vec3::default();

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

    fn sample(&self, u: &Point2F, pdf: &mut Float) -> Interaction {
        let disk_point = concentric_sample_disk(u);
        let object_point = Point3::new(
            disk_point.x * self.radius,
            disk_point.y * self.radius,
            self.height,
        );

        let mut normal = Normal::new(0.0, 0.0, 1.0)
            .transform(&self.object_to_world)
            .normalize();
        if self.reverse_orientation {
            normal *= -1.0;
        }

        let mut point_error = Vec3::default();
        let point = object_point.transform_with_point_error(
            &self.object_to_world,
            &Vec3::default(),
            &mut point_error,
        );

        *pdf = 1.0 / self.area();

        Interaction {
            point,
            point_error,
            normal,
            ..Default::default()
        }
    }

    fn area(&self) -> Float {
        self.phi_max * 0.5 * (self.radius * self.radius - self.inner_radius * self.inner_radius)
    }
}
