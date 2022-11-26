use crate::{
    material::Material,
    math::{aabb::AABB, vec3::Vec3},
    ray::Ray,
};

pub trait Object: Send + Sync {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
    fn bounding_box(&self, start_time: f32, end_time: f32) -> Option<AABB>;

    fn pdf_value(&self, _origin: &Vec3, _direction: &Vec3) -> f32 {
        0.0
    }

    fn random(&self, _origin: &Vec3) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
}

pub struct HitRecord {
    pub position: Vec3,
    pub normal: Vec3,
    pub material: Box<dyn Material + Send + Sync>,
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub is_front_face: bool,
}

impl HitRecord {
    pub fn new(
        position: Vec3,
        normal: Vec3,
        material: Box<dyn Material + Send + Sync>,
        t: f32,
        u: f32,
        v: f32,
        is_front_face: bool,
    ) -> Self {
        Self {
            position,
            normal,
            material,
            t,
            u,
            v,
            is_front_face,
        }
    }

    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: &Vec3) {
        self.is_front_face = Vec3::dot(&ray.direction, outward_normal) < 0.0;
        self.normal = if self.is_front_face {
            outward_normal.clone()
        } else {
            -outward_normal.clone()
        };
    }
}

impl Clone for HitRecord {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            normal: self.normal,
            material: self.material.clone(),
            t: self.t,
            u: self.u,
            v: self.v,
            is_front_face: self.is_front_face,
        }
    }
}
