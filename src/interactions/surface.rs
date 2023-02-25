use crate::{
    base::{bsdf::BSDF, material::TransportMode, primitive::Primitive},
    geometries::{
        normal::Normal, point2::Point2, point3::Point3, ray::RayDifferential, vec3::Vec3,
    },
    utils::math::Float,
};

pub struct Shading {
    pub n: Normal,
    pub dpdu: Vec3,
    pub dpdv: Vec3,
    pub dndu: Normal,
    pub dndv: Normal,
}

pub struct SurfaceInteraction {
    pub p: Point3,
    pub p_error: Vec3,
    pub n: Normal,
    pub wo: Vec3,
    pub time: Float,
    pub uv: Point2,
    pub dpdu: Vec3,
    pub dpdv: Vec3,
    pub dndu: Normal,
    pub dndv: Normal,
    pub shading: Shading,
    pub bsdf: Option<BSDF>,
    pub dpdx: Vec3,
    pub dpdy: Vec3,
    pub dudx: Float,
    pub dvdx: Float,
    pub dudy: Float,
    pub dvdy: Float,
    pub face_index: usize,
}

impl SurfaceInteraction {
    pub fn new(
        point: Point3,
        point_error: Vec3,
        uv: Point2,
        negative_direction: Vec3,
        dpdu: Vec3,
        dpdv: Vec3,
        dndu: Normal,
        dndv: Normal,
        time: Float,
        face_index: usize,
        reverse_orientation: bool,
        transform_swaps_handedness: bool,
    ) -> Self {
        let mut n = Normal::from(dpdu.cross(&dpdv).normalize());
        if reverse_orientation ^ transform_swaps_handedness {
            n *= -1.0;
        }

        Self {
            p: point,
            p_error: point_error,
            n,
            wo: negative_direction,
            time,
            uv,
            dpdu,
            dpdv,
            dndu,
            dndv,
            // Initialize shading geometry from true geometry.
            shading: Shading {
                n,
                dpdu,
                dpdv,
                dndu,
                dndv,
            },
            bsdf: None,
            dpdx: Vec3::default(),
            dpdy: Vec3::default(),
            dudx: 0.0,
            dvdx: 0.0,
            dudy: 0.0,
            dvdy: 0.0,
            face_index,
        }
    }

    pub fn set_shading_geometry(
        &mut self,
        dpdus: &Vec3,
        dpdvs: &Vec3,
        dndus: &Normal,
        dndvs: &Normal,
        orientation_is_authoritative: bool,
    ) {
        // Compute shading normal.
        self.shading.n = Normal::from(dpdus.cross(dpdvs)).normalize();
        if orientation_is_authoritative {
            self.n = self.n.face_forward(&self.shading.n);
        } else {
            self.shading.n = self.shading.n.face_forward(&self.n);
        }

        // Initialize shading partial derivatives.
        self.shading.dpdu.clone_from(dpdus);
        self.shading.dpdv.clone_from(dpdvs);
        self.shading.dndu.clone_from(dndus);
        self.shading.dndv.clone_from(dndvs);
    }

    pub fn compute_scattering_functions(
        &mut self,
        primitive: &dyn Primitive,
        ray: &RayDifferential,
        mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        self.compute_differentials(ray);
        primitive.compute_scattering_functions(self, mode, allow_multiple_lobes);
    }

    pub fn compute_differentials(&self, ray: &RayDifferential) {
        todo!()
    }

    pub fn le(&self) {
        todo!()
    }
}

impl Default for SurfaceInteraction {
    fn default() -> Self {
        Self {
            p: Point3::default(),
            p_error: Vec3::default(),
            n: Normal::default(),
            wo: Vec3::default(),
            time: 0.0,
            uv: Point2::default(),
            dpdu: Vec3::default(),
            dpdv: Vec3::default(),
            dndu: Normal::default(),
            dndv: Normal::default(),
            shading: Shading {
                n: Normal::default(),
                dpdu: Vec3::default(),
                dpdv: Vec3::default(),
                dndu: Normal::default(),
                dndv: Normal::default(),
            },
            bsdf: None,
            dpdx: Vec3::default(),
            dpdy: Vec3::default(),
            dudx: 0.0,
            dvdx: 0.0,
            dudy: 0.0,
            dvdy: 0.0,
            face_index: 0,
        }
    }
}
