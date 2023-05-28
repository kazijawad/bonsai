use std::{mem, sync::Arc};

use crate::{
    base::{
        constants::{Float, PI},
        math::gamma,
        sampling::uniform_sample_triangle,
        shape::Shape,
        transform::Transform,
    },
    geometries::{
        bounds3::Bounds3, normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3,
    },
    interactions::{base::BaseInteraction, surface::SurfaceInteraction},
};

pub struct TriangleMesh {
    pub position: Vec<Point3>,
    tangent: Option<Vec<Vec3>>,
    normal: Option<Vec<Normal>>,
    uv: Option<Vec<Point2F>>,
}

pub struct TriangleMeshOptions {
    pub object_to_world: Transform,
    pub position: Vec<Point3>,
    pub tangent: Option<Vec<Vec3>>,
    pub normal: Option<Vec<Normal>>,
    pub uv: Option<Vec<Point2F>>,
}

pub struct Triangle {
    world_to_object: Arc<Transform>,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
    mesh: Arc<TriangleMesh>,
    offset: usize,
}

pub struct TriangleOptions {
    pub world_to_object: Arc<Transform>,
    pub reverse_orientation: bool,
    pub mesh: Arc<TriangleMesh>,
    pub index: usize,
}

impl TriangleMesh {
    pub fn new(opts: TriangleMeshOptions) -> Self {
        let position = if opts.object_to_world.is_identity() {
            opts.position.clone()
        } else {
            opts.position
                .iter()
                .map(|v| v.transform(&opts.object_to_world))
                .collect()
        };

        let tangent = if let Some(tangent) = opts.tangent {
            Some(if opts.object_to_world.is_identity() {
                tangent
            } else {
                tangent
                    .iter()
                    .map(|v| v.transform(&opts.object_to_world))
                    .collect()
            })
        } else {
            None
        };

        let normal = if let Some(normal) = opts.normal {
            Some(if opts.object_to_world.is_identity() {
                normal
            } else {
                normal
                    .iter()
                    .map(|v| v.transform(&opts.object_to_world))
                    .collect()
            })
        } else {
            None
        };

        Self {
            position,
            tangent,
            normal,
            uv: opts.uv,
        }
    }
}

impl Triangle {
    pub fn new(opts: TriangleOptions) -> Self {
        let transform_swaps_handedness = opts.world_to_object.swaps_handedness();
        let offset = 3 * opts.index;

        Self {
            world_to_object: opts.world_to_object,
            reverse_orientation: opts.reverse_orientation,
            transform_swaps_handedness,
            mesh: opts.mesh,
            offset,
        }
    }

    fn get_uvs(&self) -> [Point2F; 3] {
        if let Some(uv) = &self.mesh.uv {
            [uv[self.offset], uv[self.offset + 1], uv[self.offset + 2]]
        } else {
            [
                Point2F::default(),
                Point2F::new(1.0, 0.0),
                Point2F::new(1.0, 1.0),
            ]
        }
    }
}

impl Shape for Triangle {
    fn object_bounds(&self) -> Bounds3 {
        let p0 = self.mesh.position[self.offset].transform(&self.world_to_object);
        let p1 = self.mesh.position[self.offset + 1].transform(&self.world_to_object);
        let p2 = self.mesh.position[self.offset + 2].transform(&self.world_to_object);
        Bounds3::new(&p0, &p1).union_point(&p2)
    }

    fn world_bounds(&self) -> Bounds3 {
        let p0 = self.mesh.position[self.offset];
        let p1 = self.mesh.position[self.offset + 1];
        let p2 = self.mesh.position[self.offset + 2];
        Bounds3::new(&p0, &p1).union_point(&p2)
    }

    fn intersect(&self, ray: &Ray, t_hit: &mut Float, si: &mut SurfaceInteraction) -> bool {
        // Get triangle vertices.
        let p0 = &self.mesh.position[self.offset];
        let p1 = &self.mesh.position[self.offset + 1];
        let p2 = &self.mesh.position[self.offset + 2];

        // Translate vertices based on ray origin.
        let ray_origin = Vec3::from(ray.origin);
        let mut p0t = p0 - &ray_origin;
        let mut p1t = p1 - &ray_origin;
        let mut p2t = p2 - &ray_origin;

        // Permutate components of triangle vertices and ray direction.
        let kz = ray.direction.abs().max_dimension();
        let mut kx = kz + 1;
        if kx == 3 {
            kx = 0;
        }
        let mut ky = kx + 1;
        if ky == 3 {
            ky = 0;
        }
        let d = ray.direction.permute(kx, ky, kz);
        p0t = p0t.permute(kx, ky, kz);
        p1t = p1t.permute(kx, ky, kz);
        p2t = p2t.permute(kx, ky, kz);

        // Apply shear transformation to translated vertex positions.
        let sx = -d.x / d.z;
        let sy = -d.y / d.z;
        let sz = 1.0 / d.z;
        p0t.x += sx * p0t.z;
        p0t.y += sy * p0t.z;
        p1t.x += sx * p1t.z;
        p1t.y += sy * p1t.z;
        p2t.x += sx * p2t.z;
        p2t.y += sy * p2t.z;

        // Compute edge function coefficients.
        let mut e0 = p1t.x * p2t.y - p1t.y * p2t.x;
        let mut e1 = p2t.x * p0t.y - p2t.y * p0t.x;
        let mut e2 = p0t.x * p1t.y - p0t.y * p1t.x;

        // Fall back to double precision test at triangle edges.
        if mem::size_of::<Float>() == mem::size_of::<f32>() && (e0 == 0.0 || e1 == 0.0 || e2 == 0.0)
        {
            let p2txp1ty = p2t.x as f64 * p1t.y as f64;
            let p2typ1tx = p2t.y as f64 * p1t.x as f64;
            e0 = (p2typ1tx - p2txp1ty) as Float;
            let p0txp2ty = p0t.x as f64 * p2t.y as f64;
            let p0typ2tx = p0t.y as f64 * p2t.x as f64;
            e1 = (p0typ2tx - p0txp2ty) as Float;
            let p1txp0ty = p1t.x as f64 * p0t.y as f64;
            let p1typ0tx = p1t.y as f64 * p0t.x as f64;
            e2 = (p1typ0tx - p1txp0ty) as Float;
        }

        // Perform triangle edge and determinant tests.
        if (e0 < 0.0 || e1 < 0.0 || e2 < 0.0) && (e0 > 0.0 || e1 > 0.0 || e2 > 0.0) {
            return false;
        }
        let determinant = e0 + e1 + e2;
        if determinant == 0.0 {
            return false;
        }

        // Compute scaled hit distance to triangle and test against ray t range.
        p0t.z *= sz;
        p1t.z *= sz;
        p2t.z *= sz;
        let t_scaled = e0 * p0t.z + e1 * p1t.z + e2 * p2t.z;
        if determinant < 0.0 && (t_scaled >= 0.0 || t_scaled < ray.t_max * determinant) {
            return false;
        } else if determinant > 0.0 && (t_scaled <= 0.0 || t_scaled > ray.t_max * determinant) {
            return false;
        }

        // Compute barycentric coordinates and t values for triangle intersection.
        let inverted_determinant = 1.0 / determinant;
        let b0 = e0 * inverted_determinant;
        let b1 = e1 * inverted_determinant;
        let b2 = e2 * inverted_determinant;
        let t = t_scaled * inverted_determinant;

        // Compute delta z term for triangle t error bounds.
        let max_zt = Vec3::new(p0t.z, p1t.z, p2t.z).abs().max_component();
        let delta_z = gamma(3.0) * max_zt;

        // Compute delta x and y terms for triangle t error bounds.
        let max_xt = Vec3::new(p0t.x, p1t.x, p2t.x).abs().max_component();
        let max_yt = Vec3::new(p0t.y, p1t.y, p2t.y).abs().max_component();
        let delta_x = gamma(5.0) * (max_xt + max_zt);
        let delta_y = gamma(5.0) * (max_yt + max_zt);

        // Compute delta e term for triangle t error bounds.
        let delta_e = 2.0 * (gamma(2.0) * max_xt * max_yt + delta_y * max_xt + delta_x * max_yt);

        // Compute delta t term for triangle t error bounds.
        let max_e = Vec3::new(e0, e1, e2).abs().max_component();
        let delta_t = 3.0
            * (gamma(3.0) * max_e * max_zt + delta_e * max_zt + delta_z * max_e)
            * inverted_determinant.abs();
        if t <= delta_t {
            return false;
        }

        // Compute triangle partial derivatives.
        let mut dpdu = Vec3::default();
        let mut dpdv = Vec3::default();
        let uvs = self.get_uvs();

        // Compute deltas for triangle partial derivatives.
        let duv02 = uvs[0] - uvs[2];
        let duv12 = uvs[1] - uvs[2];
        let dp02 = p0 - p2;
        let dp12 = p1 - p2;
        let determinant = duv02[0] * duv12[1] - duv02[1] * duv12[0];
        let degenerate_uv = determinant.abs() < 1e-8;
        if !degenerate_uv {
            let inverted_determinant = 1.0 / determinant;
            dpdu = (duv12[1] * dp02 - duv02[1] * dp12) * inverted_determinant;
            dpdv = (-duv12[0] * dp02 + duv02[0] * dp12) * inverted_determinant;
        }
        if degenerate_uv || dpdu.cross(&dpdv).length_squared() == 0.0 {
            // Handle zero determinant for triangle partial derivative matrix.
            let ng = (p2 - p0).cross(&(p1 - p0));
            if ng.length_squared() == 0.0 {
                // Exit early for degenerate triangle.
                return false;
            }
            let cs = Vec3::coordinate_system(&ng.normalize());
            dpdu = cs.0;
            dpdv = cs.1;
        }

        // Compute error bounds for triangle intersection.
        let x_abs_sum = (b0 * p0.x).abs() + (b1 * p1.x).abs() + (b2 * p2.x).abs();
        let y_abs_sum = (b0 * p0.y).abs() + (b1 * p1.y).abs() + (b2 * p2.y).abs();
        let z_abs_sum = (b0 * p0.z).abs() + (b1 * p1.z).abs() + (b2 * p2.z).abs();
        let p_error = gamma(7.0) * Vec3::new(x_abs_sum, y_abs_sum, z_abs_sum);

        // Interpolate UV parametric coordinates and hit point.
        let p_hit = b0 * p0 + b1 * p1 + b2 * p2;
        let uv_hit = b0 * uvs[0] + b1 * uvs[1] + b2 * uvs[2];

        // Fill in interaction from triangle hit.
        *si = SurfaceInteraction::new(
            p_hit,
            p_error,
            uv_hit,
            -ray.direction,
            dpdu,
            dpdv,
            Normal::default(),
            Normal::default(),
            ray.time,
            self.reverse_orientation,
            self.transform_swaps_handedness,
        );

        // Override surface normals in interaction for triangle.
        let new_normal = Normal::from(dp02.cross(&dp12).normalize());
        si.n = new_normal;
        si.shading.n = new_normal;
        if self.reverse_orientation ^ self.transform_swaps_handedness {
            let new_normal = -si.n;
            si.n = new_normal;
            si.shading.n = new_normal;
        }

        if self.mesh.normal.is_some() || self.mesh.tangent.is_some() {
            // Compute shading normal for triangle.
            let shading_normal = if let Some(normal) = &self.mesh.normal {
                let new_normal = b0 * normal[self.offset]
                    + b1 * normal[self.offset + 1]
                    + b2 * normal[self.offset + 2];
                if new_normal.length_squared() > 0.0 {
                    new_normal.normalize()
                } else {
                    si.n
                }
            } else {
                si.n
            };

            // Compute shading tangent for triangle.
            let mut shading_tangent = if let Some(tangent) = &self.mesh.tangent {
                let new_tangent = b0 * tangent[self.offset]
                    + b1 * tangent[self.offset + 1]
                    + b2 * tangent[self.offset + 2];
                if new_tangent.length_squared() > 0.0 {
                    new_tangent.normalize()
                } else {
                    si.dpdu.normalize()
                }
            } else {
                si.dpdu.normalize()
            };

            // Compute shading bitangent for triangle and adjust shading tangent.
            let mut shading_bitangent = shading_tangent.cross(&Vec3::from(shading_normal));
            if shading_bitangent.length_squared() > 0.0 {
                shading_bitangent = shading_bitangent.normalize();
                shading_tangent = shading_bitangent.cross(&Vec3::from(shading_normal));
            } else {
                let cs = Vec3::coordinate_system(&Vec3::from(shading_normal));
                shading_tangent = cs.0;
                shading_bitangent = cs.1;
            }

            // Compute normal partial derivatives for triangle shading geometry.
            let mut dndu = Normal::default();
            let mut dndv = Normal::default();
            if let Some(normal) = &self.mesh.normal {
                // Compute deltas for triangle partial derivatives of normal.
                let duv02 = uvs[0] - uvs[2];
                let duv12 = uvs[1] - uvs[2];
                let dn1 = normal[self.offset] - normal[self.offset + 2];
                let dn2 = normal[self.offset + 1] - normal[self.offset + 2];
                let determinant = duv02[0] * duv12[1] - duv02[1] * duv12[0];
                let degenerate_uv = determinant.abs() < 1e-8;
                if degenerate_uv {
                    // We can still compute dndu and dndv, with respect to the
                    // same arbitrary coordinate system we use to compute dpdu
                    // and dpdv when this happens. It's important to do this
                    // so that ray differentials for rays reflected from triangles
                    // with degenerate parameterizations are still reasonable.
                    let dn = Vec3::from(normal[self.offset + 2] - normal[self.offset])
                        .cross(&Vec3::from(normal[self.offset + 1] - normal[self.offset]));
                    if dn.length_squared() != 0.0 {
                        let (dnu, dnv) = Vec3::coordinate_system(&dn);
                        dndu = Normal::from(dnu);
                        dndv = Normal::from(dnv);
                    }
                } else {
                    let inverted_determinant = 1.0 / determinant;
                    dndu = (duv12[1] * dn1 - duv02[1] * dn2) * inverted_determinant;
                    dndv = (-duv12[0] * dn1 + duv02[0] * dn2) * inverted_determinant;
                }
            }

            if self.reverse_orientation {
                shading_bitangent = -shading_bitangent;
            }
            si.set_shading_geometry(&shading_tangent, &shading_bitangent, &dndu, &dndv, true);
        }

        *t_hit = t;
        true
    }

    fn intersect_test(&self, ray: &Ray) -> bool {
        // Get triangle vertices.
        let p0 = &self.mesh.position[self.offset];
        let p1 = &self.mesh.position[self.offset + 1];
        let p2 = &self.mesh.position[self.offset + 2];

        // Translate vertices based on ray origin.
        let ray_origin = Vec3::from(ray.origin);
        let mut p0t = p0 - &ray_origin;
        let mut p1t = p1 - &ray_origin;
        let mut p2t = p2 - &ray_origin;

        // Permutate components of triangle vertices and ray direction.
        let kz = ray.direction.abs().max_dimension();
        let mut kx = kz + 1;
        if kx == 3 {
            kx = 0;
        }
        let mut ky = kx + 1;
        if ky == 3 {
            ky = 0;
        }
        let d = ray.direction.permute(kx, ky, kz);
        p0t = p0t.permute(kx, ky, kz);
        p1t = p1t.permute(kx, ky, kz);
        p2t = p2t.permute(kx, ky, kz);

        // Apply shear transformation to translated vertex positions.
        let sx = -d.x / d.z;
        let sy = -d.y / d.z;
        let sz = 1.0 / d.z;
        p0t.x += sx * p0t.z;
        p0t.y += sy * p0t.z;
        p1t.x += sx * p1t.z;
        p1t.y += sy * p1t.z;
        p2t.x += sx * p2t.z;
        p2t.y += sy * p2t.z;

        // Compute edge function coefficients.
        let mut e0 = p1t.x * p2t.y - p1t.y * p2t.x;
        let mut e1 = p2t.x * p0t.y - p2t.y * p0t.x;
        let mut e2 = p0t.x * p1t.y - p0t.y * p1t.x;

        // Fall back to double precision test at triangle edges.
        if mem::size_of::<Float>() == mem::size_of::<f32>() && (e0 == 0.0 || e1 == 0.0 || e2 == 0.0)
        {
            let p2txp1ty = p2t.x as f64 * p1t.y as f64;
            let p2typ1tx = p2t.y as f64 * p1t.x as f64;
            e0 = (p2typ1tx - p2txp1ty) as Float;
            let p0txp2ty = p0t.x as f64 * p2t.y as f64;
            let p0typ2tx = p0t.y as f64 * p2t.x as f64;
            e1 = (p0typ2tx - p0txp2ty) as Float;
            let p1txp0ty = p1t.x as f64 * p0t.y as f64;
            let p1typ0tx = p1t.y as f64 * p0t.x as f64;
            e2 = (p1typ0tx - p1txp0ty) as Float;
        }

        // Perform triangle edge and determinant tests.
        if (e0 < 0.0 || e1 < 0.0 || e2 < 0.0) && (e0 > 0.0 || e1 > 0.0 || e2 > 0.0) {
            return false;
        }
        let determinant = e0 + e1 + e2;
        if determinant == 0.0 {
            return false;
        }

        // Compute scaled hit distance to triangle and test against ray t range.
        p0t.z *= sz;
        p1t.z *= sz;
        p2t.z *= sz;
        let t_scaled = e0 * p0t.z + e1 * p1t.z + e2 * p2t.z;
        if determinant < 0.0 && (t_scaled >= 0.0 || t_scaled < ray.t_max * determinant) {
            return false;
        } else if determinant > 0.0 && (t_scaled <= 0.0 || t_scaled > ray.t_max * determinant) {
            return false;
        }

        // Compute barycentric coordinates and t values for triangle intersection.
        let inverted_determinant = 1.0 / determinant;
        let t = t_scaled * inverted_determinant;

        // Compute delta z term for triangle t error bounds.
        let max_zt = Vec3::new(p0t.z, p1t.z, p2t.z).abs().max_component();
        let delta_z = gamma(3.0) * max_zt;

        // Compute delta x and y terms for triangle t error bounds.
        let max_xt = Vec3::new(p0t.x, p1t.x, p2t.x).abs().max_component();
        let max_yt = Vec3::new(p0t.y, p1t.y, p2t.y).abs().max_component();
        let delta_x = gamma(5.0) * (max_xt + max_zt);
        let delta_y = gamma(5.0) * (max_yt + max_zt);

        // Compute delta e term for triangle t error bounds.
        let delta_e = 2.0 * (gamma(2.0) * max_xt * max_yt + delta_y * max_xt + delta_x * max_yt);

        // Compute delta t term for triangle t error bounds.
        let max_e = Vec3::new(e0, e1, e2).abs().max_component();
        let delta_t = 3.0
            * (gamma(3.0) * max_e * max_zt + delta_e * max_zt + delta_z * max_e)
            * inverted_determinant.abs();
        if t <= delta_t {
            return false;
        }

        // Compute triangle partial derivatives.
        let mut dpdu = Vec3::default();
        let mut dpdv = Vec3::default();
        let uvs = self.get_uvs();

        // Compute deltas for triangle partial derivatives.
        let duv02 = uvs[0] - uvs[2];
        let duv12 = uvs[1] - uvs[2];
        let dp02 = p0 - p2;
        let dp12 = p1 - p2;
        let determinant = duv02[0] * duv12[1] - duv02[1] * duv12[0];
        let degenerate_uv = determinant.abs() < 1e-8;
        if !degenerate_uv {
            let inverted_determinant = 1.0 / determinant;
            dpdu = (duv12[1] * dp02 - duv02[1] * dp12) * inverted_determinant;
            dpdv = (-duv12[0] * dp02 + duv02[0] * dp12) * inverted_determinant;
        }
        if degenerate_uv || dpdu.cross(&dpdv).length_squared() == 0.0 {
            // Handle zero determinant for triangle partial derivative matrix.
            let ng = (p2 - p0).cross(&(p1 - p0));
            if ng.length_squared() == 0.0 {
                // Exit early for degenerate triangle.
                return false;
            }
        }

        true
    }

    fn sample(&self, u: &Point2F, pdf: &mut Float) -> BaseInteraction {
        let b = uniform_sample_triangle(u);

        // Query triangle vertices.
        let p0 = &self.mesh.position[self.offset];
        let p1 = &self.mesh.position[self.offset + 1];
        let p2 = &self.mesh.position[self.offset + 2];

        let p = b[0] * p0 + b[1] * p1 + (1.0 - b[0] - b[1]) * p2;

        // Compute surface normal for sampled point on triangle.
        let mut n = Normal::from((p1 - p0).cross(&(p2 - p0)).normalize());
        // Ensure correct orientation of the geometric normal.
        if let Some(normal) = &self.mesh.normal {
            let ns = Normal::from(
                b[0] * normal[self.offset]
                    + b[1] * normal[self.offset + 1]
                    + (1.0 - b[0] - b[1]) * normal[self.offset + 2],
            );
            n = n.face_forward(&ns);
        } else if self.reverse_orientation ^ self.transform_swaps_handedness {
            n *= -1.0;
        }

        // Compute error bounds for sampled point on triangle.
        let p_abs_sum = (b[0] * p0).abs() + (b[1] * p1).abs() + ((1.0 - b[0] - b[1]) * p2).abs();
        let p_error = gamma(6.0) * Vec3::from(p_abs_sum);

        *pdf = 1.0 / self.area();

        BaseInteraction {
            p,
            p_error,
            time: 0.0,
            wo: Vec3::default(),
            n,
        }
    }

    fn area(&self) -> Float {
        let p0 = &self.mesh.position[self.offset];
        let p1 = &self.mesh.position[self.offset + 1];
        let p2 = &self.mesh.position[self.offset + 2];
        0.5 * (p1 - p0).cross(&(p2 - p0)).length()
    }

    fn solid_angle(&self, p: &Point3, _num_samples: u32) -> Float {
        // Project the vertices into the unit sphere around p.
        let p1 = &self.mesh.position[self.offset] - p;
        let p2 = &self.mesh.position[self.offset + 1] - p;
        let p3 = &self.mesh.position[self.offset + 2] - p;

        let mut p1p2_cross = p1.cross(&p2);
        let mut p2p3_cross = p2.cross(&p3);
        let mut p3p1_cross = p3.cross(&p1);

        if p1p2_cross.length_squared() > 0.0 {
            p1p2_cross = p1p2_cross.normalize();
        }
        if p2p3_cross.length_squared() > 0.0 {
            p2p3_cross = p2p3_cross.normalize();
        }
        if p3p1_cross.length_squared() > 0.0 {
            p3p1_cross = p3p1_cross.normalize();
        }

        (p1p2_cross.dot(&-p2p3_cross).clamp(-1.0, 1.0).acos()
            + p2p3_cross.dot(&-p3p1_cross).clamp(-1.0, 1.0).acos()
            + p3p1_cross.dot(&-p1p2_cross).clamp(-1.0, 1.0).acos()
            - PI)
            .abs()
    }
}
