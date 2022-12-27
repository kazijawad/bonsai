use std::sync::Arc;

use crate::{
    base::shape::Shape,
    bssrdf::BSSRDF,
    geometric::GeometricPrimitive,
    geometries::{normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    medium::Medium,
    reflection::BSDF,
    utils::math::Float,
    MediumInterface,
};

pub trait Interaction: Send + Sync {
    fn is_surface_interaction(&self) -> bool {
        false
    }

    fn is_medium_interaction(&self) -> bool {
        false
    }

    fn spawn_ray(&self, d: &Vec3) -> Ray;
    fn spawn_ray_to_point(&self, p: Point3) -> Ray;
    fn spawn_ray_to_interaction(&self, it: Box<dyn Interaction>) -> Ray;

    fn get_medium(&self) -> Medium;
    fn get_medium_with_vec(&self, w: &Vec3) -> Medium;
}

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
    pub shape: Option<Arc<dyn Shape<'a>>>,
    pub shading: Shading,
    pub primitive: Option<Arc<GeometricPrimitive<'a>>>,
    pub bsdf: Option<BSDF>,
    pub bssrdf: Option<BSSRDF>,
    pub dpdx: Option<Vec3>,
    pub dpdy: Option<Vec3>,
    pub dudx: Option<Float>,
    pub dvdx: Option<Float>,
    pub dudy: Option<Float>,
    pub dvdy: Option<Float>,
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
        // Adjust normal based on orientation and handedness.
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
            shape: None,
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
            dpdx: None,
            dpdy: None,
            dudx: None,
            dvdx: None,
            dudy: None,
            dvdy: None,
            face_index,
        }
    }

    pub fn set_shading_geometry(
        &self,
        dpdu: &Vec3,
        dpdv: &Vec3,
        dndu: &Normal,
        dndv: &Normal,
        orientation_is_authoritative: bool,
    ) {
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
            shape: None,
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
            dpdx: None,
            dpdy: None,
            dudx: None,
            dvdx: None,
            dudy: None,
            dvdy: None,
            face_index: 0,
        }
    }
}
