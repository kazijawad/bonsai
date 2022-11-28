use std::ops;

use crate::geometries::{point3::Point3, vec3::Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds3 {
    min: Point3,
    max: Point3,
}

impl Bounds3 {
    pub fn new(a: &Point3, b: &Point3) -> Self {
        Self {
            min: Point3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            max: Point3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
        }
    }

    pub fn union_point(b: &Self, p: &Point3) -> Self {
        Self {
            min: Point3::new(b.min.x.min(p.x), b.min.y.min(p.y), b.min.z.min(p.z)),
            max: Point3::new(b.max.x.max(p.x), b.max.y.max(p.y), b.max.z.max(p.z)),
        }
    }

    pub fn union_bound(a: &Self, b: &Self) -> Self {
        Self {
            min: Point3::new(
                a.min.x.min(b.min.x),
                a.min.y.min(b.min.y),
                a.min.z.min(b.min.z),
            ),
            max: Point3::new(
                a.max.x.max(b.max.x),
                a.max.y.max(b.max.y),
                a.max.z.max(b.max.z),
            ),
        }
    }

    pub fn intersect(a: &Self, b: &Self) -> Self {
        Self {
            min: Point3::new(
                a.min.x.max(b.min.x),
                a.min.y.max(b.min.y),
                a.min.z.max(b.min.z),
            ),
            max: Point3::new(
                a.max.x.min(b.max.x),
                a.max.y.min(b.max.y),
                a.max.z.min(b.max.z),
            ),
        }
    }

    pub fn overlap(a: &Self, b: &Self) -> bool {
        let x = a.max.x >= b.min.x && a.min.x <= b.max.x;
        let y = a.max.y >= b.min.y && a.min.y <= b.max.y;
        let z = a.max.z >= b.min.z && a.min.z <= b.max.z;
        x && y && z
    }

    pub fn inside(p: &Point3, b: &Self) -> bool {
        p.x >= b.min.x
            && p.x <= b.max.x
            && p.y >= b.min.y
            && p.y <= b.max.y
            && p.z >= b.min.z
            && p.z <= b.max.z
    }

    pub fn inside_exclusive(p: &Point3, b: &Self) -> bool {
        p.x >= b.min.x
            && p.x < b.max.x
            && p.y >= b.min.y
            && p.y < b.max.y
            && p.z >= b.min.z
            && p.z < b.max.z
    }

    pub fn expand(b: &Self, delta: f32) -> Self {
        Self {
            min: b.min - Vec3::new(delta, delta, delta),
            max: b.max + Vec3::new(delta, delta, delta),
        }
    }

    pub fn diagonal(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn surface_area(&self) -> f32 {
        let diag = self.diagonal();
        2.0 * (diag.x * diag.y + diag.x * diag.z + diag.y * diag.z)
    }

    pub fn volume(&self) -> f32 {
        let diag = self.diagonal();
        diag.x * diag.y * diag.z
    }

    pub fn area(&self) -> f32 {
        let distance = self.max - self.min;
        distance.x * distance.y
    }

    pub fn maximum_extent(&self) -> u32 {
        let diag = self.diagonal();
        if diag.x > diag.y && diag.x > diag.z {
            0
        } else if diag.y > diag.z {
            1
        } else {
            2
        }
    }

    pub fn offset(&self, p: &Point3) -> Vec3 {
        let mut offset = *p - self.min;
        if self.max.x > self.min.x {
            offset.x /= self.max.x - self.min.x;
        }
        if self.max.y > self.min.y {
            offset.y /= self.max.y - self.min.y;
        }
        if self.max.z > self.min.z {
            offset.z /= self.max.z - self.min.z;
        }
        offset
    }

    pub fn bounding_sphere(&self, center: &mut Point3, radius: &mut f32) {
        *center = (self.min + self.max) / 2.0;
        *radius = if Self::inside(&center, &self) {
            Point3::distance(&center, &self.max)
        } else {
            0.0
        };
    }
}

impl Default for Bounds3 {
    fn default() -> Self {
        Self {
            min: Point3::new(f32::MAX, f32::MAX, f32::MAX),
            max: Point3::new(f32::MIN, f32::MIN, f32::MIN),
        }
    }
}

impl ops::Index<u32> for Bounds3 {
    type Output = Point3;

    fn index(&self, index: u32) -> &Self::Output {
        debug_assert!(index == 0 || index == 1);
        if index == 0 {
            &self.min
        } else {
            &self.max
        }
    }
}
