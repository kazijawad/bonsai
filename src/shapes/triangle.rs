use std::mem;

use crate::{
    base::{interaction::Interaction, shape::Shape, transform::Transform},
    geometries::{
        bounds3::Bounds3, normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3,
    },
    interactions::surface::SurfaceInteraction,
    texture::Texture,
    utils::math::{gamma, Float},
};

pub struct TriangleMesh<'a> {
    num_triangles: u32,
    num_vertices: u32,
    vertex_indices: Vec<u32>,
    face_indices: Vec<u32>,
    positions: Vec<Point3>,
    normals: Vec<Normal>,
    tangents: Vec<Vec3>,
    uvs: Vec<Point2>,
    alpha_mask: Option<&'a dyn Texture<Float>>,
    shadow_alpha_mask: Option<&'a dyn Texture<Float>>,
}

pub struct Triangle<'a> {
    object_to_world: &'a Transform,
    world_to_object: &'a Transform,
    reverse_orientation: bool,
    transform_swaps_handedness: bool,
    mesh: &'a TriangleMesh<'a>,
    vertex_index: usize,
    face_index: usize,
}

impl<'a> TriangleMesh<'a> {
    pub fn new(
        object_to_world: &Transform,
        num_triangles: u32,
        num_vertices: u32,
        vertex_indices: Vec<u32>,
        positions: Vec<Point3>,
        normals: Vec<Normal>,
        uvs: Vec<Point2>,
        tangents: Vec<Vec3>,
        face_indices: Vec<u32>,
        alpha_mask: Option<&'a dyn Texture<Float>>,
        shadow_alpha_mask: Option<&'a dyn Texture<Float>>,
    ) -> Self {
        // Convert mesh vertices to world space.
        let mut world_positions = Vec::with_capacity(positions.len());
        for i in 0..positions.len() {
            world_positions[i] = object_to_world.transform_point(&positions[i]);
        }

        let mut world_normals = Vec::with_capacity(normals.len());
        for i in 0..normals.len() {
            world_normals[i] = object_to_world.transform_normal(&normals[i]);
        }

        let mut world_tangents = Vec::with_capacity(tangents.len());
        for i in 0..tangents.len() {
            world_tangents[i] = object_to_world.transform_vec(&tangents[i]);
        }

        Self {
            num_triangles,
            num_vertices,
            vertex_indices,
            positions: world_positions,
            normals: world_normals,
            uvs,
            tangents: world_tangents,
            face_indices,
            alpha_mask: alpha_mask.clone(),
            shadow_alpha_mask: shadow_alpha_mask.clone(),
        }
    }
}

impl<'a> Triangle<'a> {
    pub fn new(
        object_to_world: &'a Transform,
        world_to_object: &'a Transform,
        reverse_orientation: bool,
        mesh: &'a TriangleMesh,
        triangle_index: usize,
    ) -> Self {
        let transform_swaps_handedness = object_to_world.swaps_handedness();

        Self {
            object_to_world,
            world_to_object,
            reverse_orientation,
            transform_swaps_handedness,
            mesh: mesh.clone(),
            vertex_index: mesh.vertex_indices[3 * triangle_index] as usize,
            face_index: mesh.face_indices.len(),
        }
    }

    fn get_uvs(&self, uvs: &mut [Point2; 3]) {
        todo!()
    }
}

impl<'a> Shape for Triangle<'a> {
    fn object_bound(&self) -> Bounds3 {
        let p0 = self
            .world_to_object
            .transform_point(&self.mesh.positions[self.vertex_index]);
        let p1 = self
            .world_to_object
            .transform_point(&self.mesh.positions[self.vertex_index + 1]);
        let p2 = self
            .world_to_object
            .transform_point(&self.mesh.positions[self.vertex_index + 2]);

        Bounds3::new(&p0, &p1).union_point(&p2)
    }

    fn world_bound(&self) -> Bounds3 {
        let p0 = &self.mesh.positions[self.vertex_index];
        let p1 = &self.mesh.positions[self.vertex_index + 1];
        let p2 = &self.mesh.positions[self.vertex_index + 2];

        Bounds3::new(&p0, &p1).union_point(&p2)
    }

    fn intersect(
        &self,
        ray: &Ray,
        t_hit: &mut Float,
        interaction: &mut SurfaceInteraction,
        include_alpha: bool,
    ) -> bool {
        // Get triangle vertices.
        let p0 = &self.mesh.positions[self.vertex_index];
        let p1 = &self.mesh.positions[self.vertex_index + 1];
        let p2 = &self.mesh.positions[self.vertex_index + 2];

        // Translate vertices based on ray origin.
        let mut p0t = p0 - &Vec3::from(ray.origin);
        let mut p1t = p1 - &Vec3::from(ray.origin);
        let mut p2t = p2 - &Vec3::from(ray.origin);

        // Permutate components of triangle vertices and ray direction.
        let kz = ray.direction.abs().max_dimension() as u32;
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
        let mut uvs = [Point2::default(); 3];
        self.get_uvs(&mut uvs);

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
            Vec3::coordinate_system(&ng.normalize(), &mut dpdu, &mut dpdv);
        }

        // Compute error bounds for triangle intersection.
        let x_abs_sum = (b0 * p0.x).abs() + (b1 * p1.x).abs() + (b2 * p2.x).abs();
        let y_abs_sum = (b0 * p0.y).abs() + (b1 * p1.y).abs() + (b2 * p2.y).abs();
        let z_abs_sum = (b0 * p0.z).abs() + (b1 * p1.z).abs() + (b2 * p2.z).abs();
        let p_error = gamma(7.0) * Vec3::new(x_abs_sum, y_abs_sum, z_abs_sum);

        // Interpolate UV parametric coordinates and hit point.
        let p_hit = b0 * p0 + b1 * p1 + b2 * p2;
        let uv_hit = b0 * uvs[0] + b1 * uvs[1] + b2 * uvs[2];

        // Test intersection against alpha texture, if present.
        if include_alpha && self.mesh.alpha_mask.is_some() {
            let mut local_interaction = SurfaceInteraction::new(
                p_hit,
                Vec3::default(),
                uv_hit,
                -ray.direction,
                dpdu,
                dpdv,
                Normal::default(),
                Normal::default(),
                ray.time,
                self.face_index,
                self.reverse_orientation,
                self.transform_swaps_handedness,
            );
            let alpha_mask = self.mesh.alpha_mask.as_ref().unwrap();
            if alpha_mask.evaluate(&mut local_interaction) == 0.0 {
                return false;
            }
        }

        // Fill in interaction from triangle hit.
        *interaction = SurfaceInteraction::new(
            p_hit,
            p_error,
            uv_hit,
            -ray.direction,
            dpdu,
            dpdv,
            Normal::default(),
            Normal::default(),
            ray.time,
            self.face_index,
            self.reverse_orientation,
            self.transform_swaps_handedness,
        );

        // Override surface normals in interaction for triangle.
        let new_normal = Normal::from(dp02.cross(&dp12).normalize());
        interaction.normal = new_normal;
        interaction.shading.normal = new_normal;
        if self.reverse_orientation ^ self.transform_swaps_handedness {
            let new_normal = -interaction.normal;
            interaction.normal = new_normal;
            interaction.shading.normal = new_normal;
        }

        if self.mesh.normals.len() > 0 || self.mesh.tangents.len() > 0 {
            // Compute shading normal for triangle.
            let shading_normal = if self.mesh.normals.len() > 0 {
                let new_normal = b0 * self.mesh.normals[self.vertex_index]
                    + b1 * self.mesh.normals[self.vertex_index + 1]
                    + b2 * self.mesh.normals[self.vertex_index + 2];
                if new_normal.length_squared() > 0.0 {
                    new_normal.normalize()
                } else {
                    interaction.normal
                }
            } else {
                interaction.normal
            };

            // Compute shading tangent for triangle.
            let mut shading_tangent = if self.mesh.tangents.len() > 0 {
                let new_tangent = b0 * self.mesh.tangents[self.vertex_index]
                    + b1 * self.mesh.tangents[self.vertex_index + 1]
                    + b2 * self.mesh.tangents[self.vertex_index + 2];
                if new_tangent.length_squared() > 0.0 {
                    new_tangent.normalize()
                } else {
                    interaction.dpdu.normalize()
                }
            } else {
                interaction.dpdu.normalize()
            };

            // Compute shading bitangent for triangle and adjust shading tangent.
            let mut shading_bitangent = shading_tangent.cross(&Vec3::from(shading_normal));
            if shading_bitangent.length_squared() > 0.0 {
                shading_bitangent = shading_bitangent.normalize();
                shading_tangent = shading_bitangent.cross(&Vec3::from(shading_normal));
            } else {
                Vec3::coordinate_system(
                    &Vec3::from(shading_normal),
                    &mut shading_tangent,
                    &mut shading_bitangent,
                );
            }

            // Compute normal partial derivatives for triangle shading geometry.
            let mut dndu = Normal::default();
            let mut dndv = Normal::default();
            if self.mesh.normals.len() > 0 {
                // Compute deltas for triangle partial derivatives of normal.
                let duv02 = uvs[0] - uvs[2];
                let duv12 = uvs[1] - uvs[2];
                let dn1 =
                    self.mesh.normals[self.vertex_index] - self.mesh.normals[self.vertex_index + 2];
                let dn2 = self.mesh.normals[self.vertex_index + 1]
                    - self.mesh.normals[self.vertex_index + 2];
                let determinant = duv02[0] * duv12[1] - duv02[1] * duv12[0];
                let degenerate_uv = determinant.abs() < 1e-8;
                if degenerate_uv {
                    // We can still compute dndu and dndv, with respect to the
                    // same arbitrary coordinate system we use to compute dpdu
                    // and dpdv when this happens. It's important to do this
                    // so that ray differentials for rays reflected from triangles
                    // with degenerate parameterizations are still reasonable.
                    let dn = Vec3::from(
                        self.mesh.normals[self.vertex_index + 2]
                            - self.mesh.normals[self.vertex_index],
                    )
                    .cross(&Vec3::from(
                        self.mesh.normals[self.vertex_index + 1]
                            - self.mesh.normals[self.vertex_index],
                    ));
                    if dn.length_squared() != 0.0 {
                        let mut dnu = Vec3::default();
                        let mut dnv = Vec3::default();
                        Vec3::coordinate_system(&dn, &mut dnu, &mut dnv);
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
            interaction.set_shading_geometry(
                &shading_tangent,
                &shading_bitangent,
                &dndu,
                &dndv,
                true,
            );
        }

        *t_hit = t;
        true
    }

    fn intersect_test(&self, ray: &Ray, include_alpha: bool) -> bool {
        // Get triangle vertices.
        let p0 = &self.mesh.positions[self.vertex_index];
        let p1 = &self.mesh.positions[self.vertex_index + 1];
        let p2 = &self.mesh.positions[self.vertex_index + 2];

        // Translate vertices based on ray origin.
        let mut p0t = p0 - &Vec3::from(ray.origin);
        let mut p1t = p1 - &Vec3::from(ray.origin);
        let mut p2t = p2 - &Vec3::from(ray.origin);

        // Permutate components of triangle vertices and ray direction.
        let kz = ray.direction.abs().max_dimension() as u32;
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
        let mut uvs = [Point2::default(); 3];
        self.get_uvs(&mut uvs);

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
            Vec3::coordinate_system(&ng.normalize(), &mut dpdu, &mut dpdv);
        }

        // Compute error bounds for triangle intersection.
        let x_abs_sum = (b0 * p0.x).abs() + (b1 * p1.x).abs() + (b2 * p2.x).abs();
        let y_abs_sum = (b0 * p0.y).abs() + (b1 * p1.y).abs() + (b2 * p2.y).abs();
        let z_abs_sum = (b0 * p0.z).abs() + (b1 * p1.z).abs() + (b2 * p2.z).abs();
        let p_error = gamma(7.0) * Vec3::new(x_abs_sum, y_abs_sum, z_abs_sum);

        // Interpolate UV parametric coordinates and hit point.
        let p_hit = b0 * p0 + b1 * p1 + b2 * p2;
        let uv_hit = b0 * uvs[0] + b1 * uvs[1] + b2 * uvs[2];

        // Test intersection against alpha texture, if present.
        if include_alpha && self.mesh.alpha_mask.is_some() {
            let mut local_interaction = SurfaceInteraction::new(
                p_hit,
                Vec3::default(),
                uv_hit,
                -ray.direction,
                dpdu,
                dpdv,
                Normal::default(),
                Normal::default(),
                ray.time,
                self.face_index,
                self.reverse_orientation,
                self.transform_swaps_handedness,
            );
            let alpha_mask = self.mesh.alpha_mask.as_ref().unwrap();
            if alpha_mask.evaluate(&mut local_interaction) == 0.0 {
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
        let p0 = self.mesh.positions[self.vertex_index];
        let p1 = self.mesh.positions[self.vertex_index + 1];
        let p2 = self.mesh.positions[self.vertex_index + 2];
        0.5 * (p1 - p0).cross(&(p2 - p0)).length()
    }

    fn solid_angle(&self, p: &Point3, n_samples: u32) -> Float {
        todo!()
    }
}
