use std::{mem, sync::Arc};

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
    interactions::surface::SurfaceInteraction,
    utils::math::gamma,
};

pub struct TriangleMesh {
    vertex_indices: Vec<usize>,
    positions: Vec<Point3>,
    tangents: Vec<Vec3>,
    normals: Vec<Normal>,
    uvs: Option<Vec<Point2>>,
}

pub struct Triangle {
    world_to_object: Arc<Transform>,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
    mesh: Arc<TriangleMesh>,
    vertex_indices: Vec<usize>,
}

impl TriangleMesh {
    pub fn new(
        object_to_world: &Transform,
        vertex_indices: Vec<usize>,
        positions: Vec<Point3>,
        tangents: Vec<Vec3>,
        normals: Vec<Normal>,
        uvs: Option<Vec<Point2>>,
    ) -> Self {
        // Convert mesh vertices to world space.
        let mut world_positions = Vec::with_capacity(positions.len());
        for position in positions.iter() {
            world_positions.push(position.transform(object_to_world));
        }

        let mut world_tangents = Vec::with_capacity(tangents.len());
        for tangent in tangents.iter() {
            world_tangents.push(object_to_world.transform_vec(tangent));
        }

        let mut world_normals = Vec::with_capacity(normals.len());
        for normal in normals.iter() {
            world_normals.push(normal.transform(object_to_world));
        }

        Self {
            vertex_indices,
            positions: world_positions,
            tangents: world_tangents,
            normals: world_normals,
            uvs,
        }
    }
}

impl Triangle {
    pub fn new(
        object_to_world: Arc<Transform>,
        world_to_object: Arc<Transform>,
        reverse_orientation: bool,
        mesh: Arc<TriangleMesh>,
        triangle_index: usize,
    ) -> Self {
        let transform_swaps_handedness = object_to_world.swaps_handedness();

        let vertex_offset = 3 * triangle_index;
        let vertex_indices = mesh.vertex_indices[vertex_offset..vertex_offset + 3].to_vec();

        Self {
            world_to_object,
            reverse_orientation,
            transform_swaps_handedness,
            mesh,
            vertex_indices,
        }
    }

    fn get_uvs(&self) -> [Point2; 3] {
        if let Some(uvs) = self.mesh.uvs.as_ref() {
            [
                uvs[self.vertex_indices[0]],
                uvs[self.vertex_indices[1]],
                uvs[self.vertex_indices[2]],
            ]
        } else {
            [
                Point2::default(),
                Point2::new(1.0, 0.0),
                Point2::new(1.0, 1.0),
            ]
        }
    }
}

impl Shape for Triangle {
    fn object_bound(&self) -> Bounds3 {
        let p0 = self.mesh.positions[self.vertex_indices[0]].transform(&self.world_to_object);
        let p1 = self.mesh.positions[self.vertex_indices[1]].transform(&self.world_to_object);
        let p2 = self.mesh.positions[self.vertex_indices[2]].transform(&self.world_to_object);
        Bounds3::new(&p0, &p1).union_point(&p2)
    }

    fn world_bound(&self) -> Bounds3 {
        let p0 = self.mesh.positions[self.vertex_indices[0]];
        let p1 = self.mesh.positions[self.vertex_indices[1]];
        let p2 = self.mesh.positions[self.vertex_indices[2]];
        Bounds3::new(&p0, &p1).union_point(&p2)
    }

    fn intersect(
        &self,
        ray: &Ray,
        t_hit: &mut Float,
        si: &mut SurfaceInteraction,
        _include_alpha: bool,
    ) -> bool {
        // Get triangle vertices.
        let p0 = self.mesh.positions[self.vertex_indices[0]];
        let p1 = self.mesh.positions[self.vertex_indices[1]];
        let p2 = self.mesh.positions[self.vertex_indices[2]];

        // Translate vertices based on ray origin.
        let ray_origin = Vec3::from(ray.origin);
        let mut p0t = p0 - ray_origin;
        let mut p1t = p1 - ray_origin;
        let mut p2t = p2 - ray_origin;

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
        si.base.n = new_normal;
        si.shading.n = new_normal;
        if self.reverse_orientation ^ self.transform_swaps_handedness {
            let new_normal = -si.base.n;
            si.base.n = new_normal;
            si.shading.n = new_normal;
        }

        if self.mesh.normals.len() > 0 || self.mesh.tangents.len() > 0 {
            // Compute shading normal for triangle.
            let shading_normal = if self.mesh.normals.len() > 0 {
                let new_normal = b0 * self.mesh.normals[self.vertex_indices[0]]
                    + b1 * self.mesh.normals[self.vertex_indices[1]]
                    + b2 * self.mesh.normals[self.vertex_indices[2]];
                if new_normal.length_squared() > 0.0 {
                    new_normal.normalize()
                } else {
                    si.base.n
                }
            } else {
                si.base.n
            };

            // Compute shading tangent for triangle.
            let mut shading_tangent = if self.mesh.tangents.len() > 0 {
                let new_tangent = b0 * self.mesh.tangents[self.vertex_indices[0]]
                    + b1 * self.mesh.tangents[self.vertex_indices[1]]
                    + b2 * self.mesh.tangents[self.vertex_indices[2]];
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
            if self.mesh.normals.len() > 0 {
                // Compute deltas for triangle partial derivatives of normal.
                let duv02 = uvs[0] - uvs[2];
                let duv12 = uvs[1] - uvs[2];
                let dn1 = self.mesh.normals[self.vertex_indices[0]]
                    - self.mesh.normals[self.vertex_indices[2]];
                let dn2 = self.mesh.normals[self.vertex_indices[1]]
                    - self.mesh.normals[self.vertex_indices[2]];
                let determinant = duv02[0] * duv12[1] - duv02[1] * duv12[0];
                let degenerate_uv = determinant.abs() < 1e-8;
                if degenerate_uv {
                    // We can still compute dndu and dndv, with respect to the
                    // same arbitrary coordinate system we use to compute dpdu
                    // and dpdv when this happens. It's important to do this
                    // so that ray differentials for rays reflected from triangles
                    // with degenerate parameterizations are still reasonable.
                    let dn = Vec3::from(
                        self.mesh.normals[self.vertex_indices[2]]
                            - self.mesh.normals[self.vertex_indices[0]],
                    )
                    .cross(&Vec3::from(
                        self.mesh.normals[self.vertex_indices[1]]
                            - self.mesh.normals[self.vertex_indices[0]],
                    ));
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

    fn intersect_test(&self, ray: &Ray, _include_alpha: bool) -> bool {
        // Get triangle vertices.
        let p0 = self.mesh.positions[self.vertex_indices[0]];
        let p1 = self.mesh.positions[self.vertex_indices[1]];
        let p2 = self.mesh.positions[self.vertex_indices[2]];

        // Translate vertices based on ray origin.
        let ray_origin = Vec3::from(ray.origin);
        let mut p0t = p0 - ray_origin;
        let mut p1t = p1 - ray_origin;
        let mut p2t = p2 - ray_origin;

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

    fn sample(&self, _u: &Point2, _pdf: &mut Float) -> Box<dyn Interaction> {
        todo!()
    }

    fn area(&self) -> Float {
        let p0 = self.mesh.positions[self.vertex_indices[0]];
        let p1 = self.mesh.positions[self.vertex_indices[1]];
        let p2 = self.mesh.positions[self.vertex_indices[2]];
        0.5 * (p1 - p0).cross(&(p2 - p0)).length()
    }

    fn solid_angle(&self, p: &Point3, _num_samples: u32) -> Float {
        // Project the vertices into the unit sphere around p.
        let points = [
            &self.mesh.positions[self.vertex_indices[0]] - p,
            &self.mesh.positions[self.vertex_indices[1]] - p,
            &self.mesh.positions[self.vertex_indices[2]] - p,
        ];

        let mut cross_01 = points[0].cross(&points[1]);
        let mut cross_12 = points[1].cross(&points[2]);
        let mut cross_20 = points[2].cross(&points[0]);

        if cross_01.length_squared() > 0.0 {
            cross_01 = cross_01.normalize();
        }
        if cross_12.length_squared() > 0.0 {
            cross_12 = cross_12.normalize();
        }
        if cross_20.length_squared() > 0.0 {
            cross_20 = cross_20.normalize();
        }

        (cross_01.dot(&-cross_12).clamp(-1.0, 1.0).acos()
            + cross_12.dot(&-cross_20).clamp(-1.0, 1.0).acos()
            + cross_20.dot(&-cross_01).clamp(-1.0, 1.0).acos()
            - PI)
            .abs()
    }
}
