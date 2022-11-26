use std::rc::Rc;

use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    math::aabb::AABB,
    object::{HitRecord, Object},
    ray::Ray,
    Vec3,
};

pub struct Scene {
    objects: Vec<Rc<dyn Object>>,
}

impl Scene {
    pub fn new() -> Self {
        Self { objects: vec![] }
    }

    pub fn add(&mut self, object: Rc<dyn Object>) {
        self.objects.push(Rc::clone(&object));
    }

    pub fn remove(&mut self, object: Rc<dyn Object>) {
        let index = self
            .objects
            .iter()
            .position(|element| Rc::ptr_eq(element, &object))
            .unwrap();
        self.objects.remove(index);
    }
}

impl Object for Scene {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut hit_record = HitRecord::new();
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        for object in self.objects.iter() {
            if let Some(object_hit_record) = object.hit(ray, t_min, closest_so_far) {
                hit_record.copy_from(&object_hit_record);
                closest_so_far = hit_record.t;
                hit_anything = true;
            }
        }

        if hit_anything {
            Some(hit_record)
        } else {
            None
        }
    }

    fn bounding_box(&self, start_time: f32, end_time: f32) -> Option<AABB> {
        if self.objects.is_empty() {
            return None;
        }

        let mut bbox = AABB::new(&Vec3::zeros(), &Vec3::zeros());
        let mut first_box = true;

        for object in self.objects.iter() {
            match object.bounding_box(start_time, end_time) {
                Some(object_bbox) => {
                    if first_box {
                        bbox.copy_from(&object_bbox);
                    } else {
                        bbox.copy_from(&AABB::surrounding_box(&bbox, &object_bbox));
                    }
                    first_box = false;
                }
                None => return None,
            }
        }

        return Some(bbox);
    }

    fn pdf_value(&self, origin: &Vec3, direction: &Vec3) -> f32 {
        let weight = 1.0 / (self.objects.len() as f32);
        let mut sum = 0.0;
        for object in self.objects.iter() {
            sum += weight * object.pdf_value(origin, direction);
        }
        sum
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        let mut rng = StdRng::from_entropy();
        let int_size = self.objects.len() as u32;
        self.objects[rng.gen_range(0..int_size) as usize].random(origin)
    }
}
