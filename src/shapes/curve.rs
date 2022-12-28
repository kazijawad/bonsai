use std::sync::Arc;

use crate::{
    base::{interaction::Interaction, shape::Shape},
    geometries::{
        bounds3::Bounds3, normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec2::Vec2,
        vec3::Vec3,
    },
    interactions::surface::SurfaceInteraction,
    transform::Transform,
    utils::math::{lerp, Float},
};

pub enum CurveType {
    Flat,
    Cylinder,
    Ribbon,
}

pub struct CurveCommon {
    curve_type: CurveType,
    control_points: [Point3; 4],
    width: [Float; 2],
    normals: Option<[Normal; 2]>,
    normal_angle: Option<Float>,
    inverse_sine_normal_angle: Option<Float>,
}

pub struct Curve {
    object_to_world: Arc<Transform>,
    world_to_object: Arc<Transform>,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
    common: Arc<CurveCommon>,
    u_min: Float,
    u_max: Float,
}

impl CurveCommon {
    pub fn new(
        curve_type: CurveType,
        control_points: [Point3; 4],
        width: [Float; 2],
        normals: Option<[Normal; 2]>,
    ) -> Arc<Self> {
        let mut curve_common = Self {
            curve_type,
            control_points,
            width,
            normals: None,
            normal_angle: None,
            inverse_sine_normal_angle: None,
        };

        if let Some(normals) = normals {
            let n0 = normals[0].normalize();
            let n1 = normals[1].normalize();
            curve_common.normals = Some([n0, n1]);
            curve_common.normal_angle = Some(n0.dot(&n1).clamp(0.0, 1.0));
            curve_common.inverse_sine_normal_angle =
                Some(1.0 / curve_common.normal_angle.unwrap().sin());
        }

        Arc::new(curve_common)
    }
}

impl Curve {
    pub fn new(
        object_to_world: Arc<Transform>,
        world_to_object: Arc<Transform>,
        reverse_orientation: bool,
        common: Arc<CurveCommon>,
        u_min: Float,
        u_max: Float,
    ) -> Self {
        let transform_swaps_handedness = object_to_world.swaps_handedness();

        Self {
            object_to_world,
            world_to_object,
            reverse_orientation,
            transform_swaps_handedness,
            common,
            u_min,
            u_max,
        }
    }

    pub fn blossom_bezier(p: [Point3; 4], u0: Float, u1: Float, u2: Float) -> Point3 {
        let a = [
            Point3::lerp(u0, &p[0], &p[1]),
            Point3::lerp(u0, &p[1], &p[2]),
            Point3::lerp(u0, &p[2], &p[3]),
        ];
        let b = [
            Point3::lerp(u1, &a[0], &a[1]),
            Point3::lerp(u1, &a[1], &a[2]),
        ];
        return Point3::lerp(u2, &b[0], &b[1]);
    }

    pub fn subdivide_bezier(cp: [Point3; 4]) -> [Point3; 7] {
        [
            cp[0],
            (cp[0] + cp[1]) / 2.0,
            (cp[0] + 2.0 * cp[1] + cp[2]) / 4.0,
            (cp[0] + 3.0 * cp[1] + 3.0 * cp[2] + cp[3]) / 8.0,
            (cp[1] + 2.0 * cp[2] + cp[3]) / 4.0,
            (cp[2] + cp[3]) / 2.0,
            cp[3],
        ]
    }

    pub fn eval_bezier(cp: [Point3; 4], u: Float, derivative: &mut Vec3) -> Point3 {
        let cp1 = [
            Point3::lerp(u, &cp[0], &cp[1]),
            Point3::lerp(u, &cp[1], &cp[2]),
            Point3::lerp(u, &cp[2], &cp[3]),
        ];
        let cp2 = [
            Point3::lerp(u, &cp1[0], &cp1[1]),
            Point3::lerp(u, &cp1[1], &cp1[2]),
        ];

        if (cp2[1] - cp2[0]).length_squared() > 0.0 {
            *derivative = 3.0 * (cp2[1] - cp2[0]);
        } else {
            // For a cubic Bezier, if the first three control points are
            // coincident, then the derivative of the curve is legitimately zeroed
            // at u = 0. This is problematic for us, because we'd like to be
            // able to compute a surface normal there. In that case, just punt and
            // take the difference between the first and last control points, which
            // isn't great, but will hopefully do.
            *derivative = cp[3] - cp[0];
        }

        Point3::lerp(u, &cp2[0], &cp2[1])
    }

    fn recursive_intersect(
        &self,
        ray: &Ray,
        t_hit: &mut Float,
        interaction: &mut SurfaceInteraction,
        cp: [Point3; 4],
        ray_to_object: &Transform,
        u0: Float,
        u1: Float,
        depth: u32,
    ) -> bool {
        let ray_length = ray.direction.length();

        if depth > 0 {
            // Split curve segments into sub-segments and test for intersection.
            let cp_split = Curve::subdivide_bezier(cp);

            // For each of the two segments, see if the ray's bounding box
            // overlaps the segment before recursively checking for
            // intersection with it.
            let mut hit = false;
            let u = [u0, (u0 + u1) / 2.0, u1];
            let mut current_cp = &cp_split[0..4];
            for seg in 0..2 {
                let max_width = lerp(u[seg], self.common.width[0], self.common.width[1]).max(lerp(
                    u[seg + 1],
                    self.common.width[0],
                    self.common.width[1],
                ));

                // Check y first because we can exit early.
                if current_cp[0]
                    .y
                    .max(current_cp[1].y)
                    .max(current_cp[2].y)
                    .max(current_cp[3].y)
                    + 0.5 * max_width
                    < 0.0
                    || current_cp[0]
                        .y
                        .min(current_cp[1].y)
                        .min(current_cp[2].y)
                        .min(current_cp[3].y)
                        - 0.5 * max_width
                        > 0.0
                {
                    continue;
                }

                if current_cp[0]
                    .x
                    .max(current_cp[1].x)
                    .max(current_cp[2].x)
                    .max(current_cp[3].x)
                    + 0.5 * max_width
                    < 0.0
                    || current_cp[0]
                        .x
                        .min(current_cp[1].x)
                        .min(current_cp[2].x)
                        .min(current_cp[3].x)
                        - 0.5 * max_width
                        > 0.0
                {
                    continue;
                }

                let z_max = ray_length * ray.t_max;
                if current_cp[0]
                    .z
                    .max(current_cp[1].z)
                    .max(current_cp[2].z)
                    .max(current_cp[3].z)
                    + 0.5 * max_width
                    < 0.0
                    || current_cp[0]
                        .z
                        .min(current_cp[1].z)
                        .min(current_cp[2].z)
                        .min(current_cp[3].z)
                        - 0.5 * max_width
                        > z_max
                {
                    continue;
                }

                hit |= self.recursive_intersect(
                    ray,
                    t_hit,
                    interaction,
                    [current_cp[0], current_cp[1], current_cp[2], current_cp[3]],
                    ray_to_object,
                    u[seg],
                    u[seg + 1],
                    depth - 1,
                );

                // If we found an intersection and this is a shadow ray,
                // we can exit out immediately.
                if hit && t_hit.is_nan() {
                    return true;
                }

                current_cp = &cp_split[3..7];
            }

            hit
        } else {
            // Intersect ray with curve segment and test
            // ray against segment endpoint boundaries. Test
            // sample point against tangent perpendicular
            // at curve start.
            let edge = (cp[1].y - cp[0].y) * -cp[0].y + cp[0].x * (cp[0].x - cp[1].x);
            if edge < 0.0 {
                return false;
            }

            // Test sample point against tangent perpendicular at curve end.
            let edge = (cp[2].y - cp[3].y) * -cp[3].y + cp[3].x * (cp[3].x - cp[2].x);
            if edge < 0.0 {
                return false;
            }

            // Compute line that gives minimum distance to sample point.
            let segment_direction = Point2::from(cp[3]) - Point2::from(cp[0]);
            let denominator = segment_direction.length_squared();
            if denominator == 0.0 {
                return false;
            }
            let w = (-Vec2::from(cp[0])).dot(&segment_direction) / denominator;

            // Compute u coordinate of curve intersection point and hit width.
            let u = lerp(w, u0, u1).clamp(u0, u1);
            let mut hit_width = lerp(u, self.common.width[0], self.common.width[1]);
            let mut n_hit = Normal::default();
            if let CurveType::Ribbon = self.common.curve_type {
                // Scale hit width based on ribbon orientation.
                let normal_angle = self.common.normal_angle.unwrap();
                let inverse_sine_normal_angle = self.common.inverse_sine_normal_angle.unwrap();
                let normals = self.common.normals.unwrap();
                let sin0 = ((1.0 - u) * normal_angle).sin() * inverse_sine_normal_angle;
                let sin1 = (u * normal_angle).sin() * inverse_sine_normal_angle;
                n_hit = sin0 * normals[0] + sin1 * normals[1];
                hit_width *= n_hit.abs_dot(&Normal::from(ray.direction)) / ray_length;
            }

            // Test intersection point against curve width.
            let mut dpcdw = Vec3::default();
            let pc = Curve::eval_bezier(cp, w.clamp(0.0, 1.0), &mut dpcdw);
            let pt_curve_dist2 = pc.x * pc.x + pc.y * pc.y;
            if pt_curve_dist2 > hit_width * hit_width * 0.25 {
                return false;
            }
            let z_max = ray_length * ray.t_max;
            if pc.z < 0.0 || pc.z > z_max {
                return false;
            }

            // Compute v coordinate of curve intersection point.
            let pt_curve_dist = pt_curve_dist2.sqrt();
            let edge_func = dpcdw.x * -pc.y + pc.x * dpcdw.y;
            let v = if edge_func > 0.0 {
                0.5 + pt_curve_dist / hit_width
            } else {
                0.5 + pt_curve_dist / hit_width
            };

            // Compute hit t and partial derivatives for curve intersection.
            if !t_hit.is_nan() {
                // This t hit is not exact for ribbons.
                *t_hit = pc.z / ray_length;

                // Compute error bounds for curve intersection.
                let p_error = Vec3::new(2.0 * hit_width, 2.0 * hit_width, 2.0 * hit_width);

                // Compute dpdu and dpdv for curve intersection.
                let mut dpdu = Vec3::default();
                Curve::eval_bezier(self.common.control_points, u, &mut dpdu);
                assert_ne!(dpdu, Vec3::default());

                let dpdv = if let CurveType::Ribbon = self.common.curve_type {
                    Vec3::from(n_hit).cross(&dpdu).normalize() * hit_width
                } else {
                    // Compute curve dpdv for flat and cylinder curves.
                    let dpdu_plane = ray_to_object.inverse().transform_vec(&dpdu);
                    let mut dpdv_plane =
                        Vec3::new(-dpdu_plane.y, dpdu_plane.x, 0.0).normalize() * hit_width;
                    if let CurveType::Cylinder = self.common.curve_type {
                        // Rotate dpdv plane to give cylindrical appearance.
                        let theta = lerp(v, -90.0, 90.0);
                        let rot = Transform::rotate(-theta, &dpdu_plane);
                        dpdv_plane = rot.transform_vec(&dpdv_plane);
                    }
                    ray_to_object.transform_vec(&dpdv_plane)
                };

                *interaction =
                    self.object_to_world
                        .transform_surface_interaction(&SurfaceInteraction::new(
                            ray.at(*t_hit),
                            p_error,
                            Point2::new(u, v),
                            -ray.direction,
                            dpdu,
                            dpdv,
                            Normal::default(),
                            Normal::default(),
                            ray.time,
                            0,
                            self.reverse_orientation,
                            self.transform_swaps_handedness,
                        ))
            }

            true
        }
    }
}

impl Shape for Curve {
    fn object_bound(&self) -> Bounds3 {
        let cp = [
            Curve::blossom_bezier(
                self.common.control_points,
                self.u_min,
                self.u_min,
                self.u_min,
            ),
            Curve::blossom_bezier(
                self.common.control_points,
                self.u_min,
                self.u_min,
                self.u_max,
            ),
            Curve::blossom_bezier(
                self.common.control_points,
                self.u_min,
                self.u_max,
                self.u_max,
            ),
            Curve::blossom_bezier(
                self.common.control_points,
                self.u_max,
                self.u_max,
                self.u_max,
            ),
        ];

        let b = Bounds3::new(&cp[0], &cp[1]).union(&Bounds3::new(&cp[2], &cp[3]));
        let width = [
            lerp(self.u_min, self.common.width[0], self.common.width[1]),
            lerp(self.u_max, self.common.width[0], self.common.width[1]),
        ];

        b.expand(width[0].max(width[1]) * 0.5)
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
        // Transform ray to object-space.
        let mut origin_error = Vec3::default();
        let mut direction_error = Vec3::default();
        let ray = self.world_to_object.transform_ray_with_error(
            r,
            &mut origin_error,
            &mut direction_error,
        );

        // Compute object-space control points for curve segment.
        let cp = [
            Curve::blossom_bezier(
                self.common.control_points,
                self.u_min,
                self.u_min,
                self.u_min,
            ),
            Curve::blossom_bezier(
                self.common.control_points,
                self.u_min,
                self.u_min,
                self.u_max,
            ),
            Curve::blossom_bezier(
                self.common.control_points,
                self.u_min,
                self.u_max,
                self.u_max,
            ),
            Curve::blossom_bezier(
                self.common.control_points,
                self.u_max,
                self.u_max,
                self.u_max,
            ),
        ];

        // Project curve control points to plane perpendicular to ray.
        // The "up" direction vector orients the curve to be roughly
        // parallel to the x axis in the ray coordinate system. This
        // allows curve bounds with minimal extent in y, which in turn
        // lets us early out earlier in the recursive test.
        let mut dx = ray.direction.cross(&(cp[3] - cp[0]));
        if dx.length_squared() == 0.0 {
            // If the ray and the vector between the first and last control
            // points are parallel, dx will be zero.  Generate an arbitrary xy
            // orientation for the ray coordinate system so that intersection
            // tests can proceeed in this unusual case.
            let mut dy = Vec3::default();
            Vec3::coordinate_system(&ray.direction, &mut dx, &mut dy);
        }

        let object_to_ray = Transform::look_at(&ray.origin, &(ray.origin + ray.direction), &dx);
        let cp = [
            object_to_ray.transform_point(&cp[0]),
            object_to_ray.transform_point(&cp[1]),
            object_to_ray.transform_point(&cp[2]),
            object_to_ray.transform_point(&cp[3]),
        ];

        // Before going any further, see if the ray's bounding box intersects
        // the curve's bounding box. We start with the y dimension, since the y
        // extent is generally the smallest (and is often tiny) due to our
        // orientation of the ray coordinate system above.
        let max_width = lerp(self.u_min, self.common.width[0], self.common.width[1]).max(lerp(
            self.u_max,
            self.common.width[0],
            self.common.width[1],
        ));
        if cp[0].y.max(cp[1].y).max(cp[2].y).max(cp[3].y) + 0.5 * max_width < 0.0
            || cp[0].y.min(cp[1].y).min(cp[2].y).min(cp[3].y) - 0.5 * max_width > 0.0
        {
            return false;
        }

        // Check for non-overlap in z.
        let ray_length = ray.direction.length();
        let z_max = ray_length * ray.t_max;
        if cp[0].z.max(cp[1].z).max(cp[2].z).max(cp[3].z) + 0.5 * max_width < 0.0
            || cp[0].z.min(cp[1].z).min(cp[2].z).min(cp[3].z) - 0.5 * max_width > z_max
        {
            return false;
        }

        // Compute refinement depth for curve.
        let mut l0: Float = 0.0;
        for i in 0..2 {
            l0 = l0
                .max((cp[i].x - 2.0 * cp[i + 1].x + cp[i + 2].x).abs())
                .max((cp[i].y - 2.0 * cp[i + 1].y + cp[i + 2].y).abs())
                .max((cp[i].z - 2.0 * cp[i + 1].z + cp[i + 2].z).abs());
        }

        let epsilon: Float = self.common.width[0].max(self.common.width[1]) * 0.05;
        // Compute log base 4 by dividing log2 in half.
        let r0 = (1.41421356237 * 6.0 * l0 / (8.0 * epsilon)).log2() as i32 / 2;
        let max_depth = r0.clamp(0, 10) as u32;

        self.recursive_intersect(
            &ray,
            t_hit,
            interaction,
            cp,
            &object_to_ray.inverse(),
            self.u_min,
            self.u_max,
            max_depth,
        )
    }

    fn intersect_test(&self, ray: &Ray, include_alpha: bool) -> bool {
        let mut t_hit = Float::NAN;
        self.intersect(
            ray,
            &mut t_hit,
            &mut SurfaceInteraction::default(),
            include_alpha,
        )
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

    fn pdf_from_ref(&self, reference: Box<dyn Interaction>, wi: &Vec3) -> Float {
        todo!()
    }

    fn area(&self) -> Float {
        // Compute object-space control points for curve segment.
        let cp = [
            Curve::blossom_bezier(
                self.common.control_points,
                self.u_min,
                self.u_min,
                self.u_min,
            ),
            Curve::blossom_bezier(
                self.common.control_points,
                self.u_min,
                self.u_min,
                self.u_max,
            ),
            Curve::blossom_bezier(
                self.common.control_points,
                self.u_min,
                self.u_max,
                self.u_max,
            ),
            Curve::blossom_bezier(
                self.common.control_points,
                self.u_max,
                self.u_max,
                self.u_max,
            ),
        ];

        let width = [
            lerp(self.u_min, self.common.width[0], self.common.width[1]),
            lerp(self.u_max, self.common.width[0], self.common.width[1]),
        ];
        let avg_width = (width[0] + width[1]) * 0.5;

        let mut approx_length = 0.0;
        for i in 0..3 {
            approx_length += cp[i].distance(&cp[i + 1]);
        }

        approx_length * avg_width
    }

    fn solid_angle(&self, p: &Point3, n_samples: u32) -> Float {
        todo!()
    }
}
