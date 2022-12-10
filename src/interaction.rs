use std::sync::Arc;

use crate::{
    bssrdf::BSSRDF,
    geometries::{normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    math::Float,
    medium::{Medium, MediumInterface},
    primitive::Primitive,
    reflection::BSDF,
    shape::Shape,
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
    fn spawn_ray_to_interaction(&self, it: Arc<dyn Interaction>) -> Ray;

    fn get_medium(&self) -> Medium;
    fn get_medium_with_vec(&self, w: &Vec3) -> Medium;
}

#[derive(Debug)]
pub struct InteractionProperties {
    pub point: Point3,
    pub point_error: Vec3,
    pub normal: Normal,
    pub negative_direction: Vec3,
    pub time: Float,
    pub medium_interface: Option<MediumInterface>,
}

#[derive(Debug)]
pub struct Shading {
    pub normal: Normal,
    pub dpdu: Vec3,
    pub dpdv: Vec3,
    pub dndu: Normal,
    pub dndv: Normal,
}

pub struct SurfaceInteraction {
    pub interaction: InteractionProperties,
    pub uv: Point2,
    pub dpdu: Vec3,
    pub dpdv: Vec3,
    pub dndu: Normal,
    pub dndv: Normal,
    pub shading: Shading,
    pub shape: Option<Arc<dyn Shape + Send + Sync>>,
    pub primitive: Option<Arc<dyn Primitive>>,
    pub bsdf: Option<BSDF>,
    pub bssrdf: Option<BSSRDF>,
    pub dpdx: Option<Vec3>,
    pub dpdy: Option<Vec3>,
    pub dudx: Option<Float>,
    pub dvdx: Option<Float>,
    pub dudy: Option<Float>,
    pub dvdy: Option<Float>,
    pub face_index: i32,
}

impl SurfaceInteraction {
    pub fn new(
        p: &Point3,
        p_error: &Vec3,
        uv: &Point2,
        wo: &Vec3,
        dpdu: &Vec3,
        dpdv: &Vec3,
        dndu: &Normal,
        dndv: &Normal,
        time: Float,
        shape: &dyn Shape,
        face_index: i32,
    ) -> Self {
        let point = p.clone();
        let point_error = p_error.clone();
        let mut normal = Normal::from(dpdu.cross(dpdv).normalize());
        let negative_direction = wo.clone();

        // Adjust normal based on orientation and handedness.
        if shape.reverse_orientation() ^ shape.transform_swaps_handedness() {
            normal *= -1.0;
        }

        Self {
            interaction: InteractionProperties {
                point,
                point_error,
                normal,
                negative_direction,
                time,
                medium_interface: None,
            },
            uv: uv.clone(),
            dpdu: dpdu.clone(),
            dpdv: dpdv.clone(),
            dndu: dndu.clone(),
            dndv: dndv.clone(),
            // Initialize shading geometry from true geometry.
            shading: Shading {
                normal,
                dpdu: dpdu.clone(),
                dpdv: dpdv.clone(),
                dndu: dndu.clone(),
                dndv: dndv.clone(),
            },
            shape: None,
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
}
