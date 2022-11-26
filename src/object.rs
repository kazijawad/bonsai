use std::rc::Rc;

use crate::{
    material::{LambertianMaterial, Material},
    math::{aabb::AABB, vec3::Vec3},
    ray::Ray,
};

pub struct HitRecord {
    pub position: Vec3,
    pub normal: Vec3,
    pub material: Rc<dyn Material>,
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub is_front_face: bool,
}

impl HitRecord {
    pub fn new() -> Self {
        Self {
            position: Vec3::zeros(),
            normal: Vec3::zeros(),
            material: Rc::new(LambertianMaterial::new(&Vec3::zeros())),
            t: 0.0,
            u: 0.0,
            v: 0.0,
            is_front_face: true,
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

    pub fn copy_from(&mut self, new_record: &Self) {
        self.position = new_record.position;
        self.normal = new_record.normal;
        self.material = Rc::clone(&new_record.material);
        self.t = new_record.t;
        self.u = new_record.u;
        self.v = new_record.v;
        self.is_front_face = new_record.is_front_face;
    }
}

pub trait Object {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
    fn bounding_box(&self, start_time: f32, end_time: f32) -> Option<AABB>;

    fn pdf_value(&self, _origin: &Vec3, _direction: &Vec3) -> f32 {
        0.0
    }

    fn random(&self, _origin: &Vec3) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
}
