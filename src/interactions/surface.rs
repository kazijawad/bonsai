use crate::{
    base::{material::TransportMode, primitive::Primitive},
    bssrdf::BSSRDF,
    geometries::{
        normal::Normal, point2::Point2, point3::Point3, ray::RayDifferential, vec3::Vec3,
    },
    medium::MediumInterface,
    reflection::BSDF,
    utils::math::Float,
};

#[derive(Clone)]
pub struct Shading {
    pub normal: Normal,
    pub dpdu: Vec3,
    pub dpdv: Vec3,
    pub dndu: Normal,
    pub dndv: Normal,
}

#[derive(Clone)]
pub struct SurfaceInteraction<'a> {
    pub point: Point3,
    pub point_error: Vec3,
    pub normal: Normal,
    pub negative_direction: Vec3,
    pub time: Float,
    pub medium_interface: MediumInterface,
    pub uv: Point2,
    pub dpdu: Vec3,
    pub dpdv: Vec3,
    pub dndu: Normal,
    pub dndv: Normal,
    pub shading: Shading,
    pub primitive: Option<&'a dyn Primitive<'a>>,
    pub bsdf: Option<BSDF>,
    pub bssrdf: Option<BSSRDF>,
    pub dpdx: Vec3,
    pub dpdy: Vec3,
    pub dudx: Float,
    pub dvdx: Float,
    pub dudy: Float,
    pub dvdy: Float,
    pub face_index: usize,
}

impl<'a> SurfaceInteraction<'a> {
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
        let mut normal = Normal::from(dpdu.cross(&dpdv).normalize());
        if reverse_orientation ^ transform_swaps_handedness {
            normal *= -1.0;
        }

        Self {
            point,
            point_error,
            normal,
            negative_direction,
            time,
            medium_interface: MediumInterface,
            uv,
            dpdu,
            dpdv,
            dndu,
            dndv,
            // Initialize shading geometry from true geometry.
            shading: Shading {
                normal,
                dpdu,
                dpdv,
                dndu,
                dndv,
            },
            primitive: None,
            bsdf: None,
            bssrdf: None,
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
        self.shading.normal = Normal::from(dpdus.cross(dpdvs)).normalize();
        if orientation_is_authoritative {
            self.normal = self.normal.face_forward(&self.shading.normal);
        } else {
            self.shading.normal = self.shading.normal.face_forward(&self.normal);
        }

        // Initialize shading partial derivatives.
        self.shading.dpdu.clone_from(dpdus);
        self.shading.dpdv.clone_from(dpdvs);
        self.shading.dndu.clone_from(dndus);
        self.shading.dndv.clone_from(dndvs);
    }

    pub fn compute_scattering_functions(
        &self,
        ray: &RayDifferential,
        allow_multiple_lobes: bool,
        mode: TransportMode,
    ) {
        self.compute_differentials(ray);
        todo!()
    }

    pub fn compute_differentials(&self, ray: &RayDifferential) {
        todo!()
    }

    pub fn le(&self) {
        todo!()
    }
}

impl<'a> Default for SurfaceInteraction<'a> {
    fn default() -> Self {
        Self {
            point: Point3::default(),
            point_error: Vec3::default(),
            normal: Normal::default(),
            negative_direction: Vec3::default(),
            time: 0.0,
            medium_interface: MediumInterface,
            uv: Point2::default(),
            dpdu: Vec3::default(),
            dpdv: Vec3::default(),
            dndu: Normal::default(),
            dndv: Normal::default(),
            shading: Shading {
                normal: Normal::default(),
                dpdu: Vec3::default(),
                dpdv: Vec3::default(),
                dndu: Normal::default(),
                dndv: Normal::default(),
            },
            primitive: None,
            bsdf: None,
            bssrdf: None,
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
