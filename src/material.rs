use std::rc::Rc;

use crate::{
    math::{
        pdf::{CosinePDF, PDF},
        vec3::Vec3,
    },
    object::HitRecord,
    ray::Ray,
    texture::{ColorTexture, Texture},
};

pub struct ScatterRecord {
    pub specular: Ray,
    pub attenuation: Vec3,
    pub distribution: Rc<dyn PDF>,
    pub is_specular: bool,
}

impl ScatterRecord {
    pub fn new() -> Self {
        Self {
            specular: Ray::new(&Vec3::zeros(), &Vec3::zeros(), 0.0),
            attenuation: Vec3::zeros(),
            distribution: Rc::new(CosinePDF::new(&Vec3::zeros())),
            is_specular: true,
        }
    }
}

pub trait Material: Send + Sync {
    fn emitted(&self, _ray: &Ray, _hit_record: &HitRecord) -> Vec3 {
        Vec3::zeros()
    }

    fn scatter(&self, _ray: &Ray, _hit_record: &HitRecord) -> Option<ScatterRecord> {
        None
    }

    fn scattering_pdf(&self, _ray: &Ray, _scattered_ray: &Ray, _hit_record: &HitRecord) -> f32 {
        0.0
    }

    fn clone_dyn(&self) -> Box<dyn Material + Send + Sync>;
}

pub struct LambertianMaterial {
    map: Box<dyn Texture>,
}

impl Clone for Box<dyn Material + Send + Sync> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

impl LambertianMaterial {
    pub fn new(color: &Vec3) -> Self {
        let map = ColorTexture::new(color);
        Self { map: Box::new(map) }
    }
}

impl Material for LambertianMaterial {
    fn scatter(&self, _ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            specular: Ray::new(&Vec3::zeros(), &Vec3::zeros(), 0.0),
            attenuation: (*self.map).value(hit_record.u, hit_record.v, &hit_record.position),
            distribution: Rc::new(CosinePDF::new(&hit_record.normal)),
            is_specular: false,
        })
    }

    fn scattering_pdf(&self, _ray: &Ray, scattered_ray: &Ray, hit_record: &HitRecord) -> f32 {
        let cosine = Vec3::dot(
            &hit_record.normal,
            &Vec3::normalize(&scattered_ray.direction),
        );
        if cosine < 0.0 {
            0.0
        } else {
            cosine / std::f32::consts::PI
        }
    }

    fn clone_dyn(&self) -> Box<dyn Material + Send + Sync> {
        Box::new(self.clone())
    }
}

impl Clone for LambertianMaterial {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
        }
    }
}
