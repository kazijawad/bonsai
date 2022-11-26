use std::mem;

use crate::{math::vec3::Vec3, ray::Ray};

pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: &Vec3, max: &Vec3) -> Self {
        Self {
            min: min.clone(),
            max: max.clone(),
        }
    }

    pub fn surrounding_box(box_a: &Self, box_b: &Self) -> Self {
        let min = Vec3::new(
            box_a.min.x.min(box_b.min.x),
            box_a.min.y.min(box_b.min.y),
            box_a.min.z.min(box_b.min.z),
        );
        let max = Vec3::new(
            box_a.max.x.max(box_b.max.x),
            box_a.max.y.max(box_b.max.y),
            box_a.max.z.max(box_b.max.z),
        );
        Self { min, max }
    }

    pub fn intersect(&self, ray: &Ray, mut t_min: f32, mut t_max: f32) -> bool {
        for a in 0..3 {
            let inverse_direction = 1.0 / ray.direction[a];
            let mut t0 = (self.min[a] - ray.origin[a]) * inverse_direction;
            let mut t1 = (self.min[a] - ray.origin[a]) * inverse_direction;
            if inverse_direction < 0.0 {
                mem::swap(&mut t0, &mut t1);
            }
            t_min = if t0 > t_min { t0 } else { t_min };
            t_max = if t1 < t_max { t1 } else { t_max };
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    pub fn copy_from(&mut self, new_box: &Self) {
        self.min = new_box.min.clone();
        self.max = new_box.max.clone();
    }
}
