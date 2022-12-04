use std::ops;

use crate::{
    geometries::{point3::Point3, vec3::Vec3},
    math,
    math::Float,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds3 {
    pub min: Point3,
    pub max: Point3,
}

impl Bounds3 {
    pub fn new(a: &Point3, b: &Point3) -> Self {
        Self {
            min: Point3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            max: Point3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
        }
    }

    pub fn union(&self, b: &Self) -> Self {
        Self {
            min: Point3::new(
                self.min.x.min(b.min.x),
                self.min.y.min(b.min.y),
                self.min.z.min(b.min.z),
            ),
            max: Point3::new(
                self.max.x.max(b.max.x),
                self.max.y.max(b.max.y),
                self.max.z.max(b.max.z),
            ),
        }
    }

    pub fn union_point(&self, p: &Point3) -> Self {
        Self {
            min: Point3::new(
                self.min.x.min(p.x),
                self.min.y.min(p.y),
                self.min.z.min(p.z),
            ),
            max: Point3::new(
                self.max.x.max(p.x),
                self.max.y.max(p.y),
                self.max.z.max(p.z),
            ),
        }
    }

    pub fn intersect(&self, b: &Self) -> Self {
        Self {
            min: Point3::new(
                self.min.x.max(b.min.x),
                self.min.y.max(b.min.y),
                self.min.z.max(b.min.z),
            ),
            max: Point3::new(
                self.max.x.min(b.max.x),
                self.max.y.min(b.max.y),
                self.max.z.min(b.max.z),
            ),
        }
    }

    pub fn overlaps(&self, b: &Self) -> bool {
        let x = self.max.x >= b.min.x && self.min.x <= b.max.x;
        let y = self.max.y >= b.min.y && self.min.y <= b.max.y;
        let z = self.max.z >= b.min.z && self.min.z <= b.max.z;
        x && y && z
    }

    pub fn inside(&self, p: &Point3) -> bool {
        p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }

    pub fn inside_exclusive(&self, p: &Point3) -> bool {
        p.x >= self.min.x
            && p.x < self.max.x
            && p.y >= self.min.y
            && p.y < self.max.y
            && p.z >= self.min.z
            && p.z < self.max.z
    }

    pub fn expand(&self, delta: Float) -> Self {
        Self::new(
            &(self.min - Vec3::new(delta, delta, delta)),
            &(self.max + Vec3::new(delta, delta, delta)),
        )
    }

    pub fn distance_squared(&self, p: &Point3) -> Float {
        let dx = (0.0 as Float).max(self.min.x - p.x).max(p.x - self.max.x);
        let dy = (0.0 as Float).max(self.min.y - p.y).max(p.y - self.max.y);
        let dz = (0.0 as Float).max(self.min.z - p.z).max(p.z - self.max.z);
        dx * dx + dy * dy + dz * dz
    }

    pub fn distance(&self, p: &Point3) -> Float {
        self.distance_squared(p).sqrt()
    }

    pub fn corner(&self, index: u32) -> Point3 {
        debug_assert!(index < 8);

        let mut ret = Point3::default();
        ret.x = self[index & 1].x;

        let y_index = if (index & 2) != 0 { 1 } else { 0 };
        ret.y = self[y_index].y;

        let z_index = if (index & 4) != 0 { 1 } else { 0 };
        ret.z = self[z_index].z;

        ret
    }

    pub fn diagonal(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn surface_area(&self) -> Float {
        let diag = self.diagonal();
        2.0 * (diag.x * diag.y + diag.x * diag.z + diag.y * diag.z)
    }

    pub fn volume(&self) -> Float {
        let diag = self.diagonal();
        diag.x * diag.y * diag.z
    }

    pub fn area(&self) -> Float {
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

    pub fn lerp(&self, t: &Point3) -> Point3 {
        Point3::new(
            math::lerp(t.x, self.min.x, self.max.x),
            math::lerp(t.y, self.min.y, self.max.y),
            math::lerp(t.z, self.min.z, self.max.z),
        )
    }

    pub fn offset(&self, p: &Point3) -> Vec3 {
        let mut offset = p - &self.min;
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

    pub fn bounding_sphere(&self) -> (Point3, Float) {
        let center = (self.min + self.max) / 2.0;
        let radius = if self.inside(&center) {
            center.distance(&self.max)
        } else {
            0.0
        };
        (center, radius)
    }
}

impl Default for Bounds3 {
    fn default() -> Self {
        Self {
            min: Point3::new(Float::MAX, Float::MAX, Float::MAX),
            max: Point3::new(Float::MIN, Float::MIN, Float::MIN),
        }
    }
}

// TYPE CONVERSION

impl From<Point3> for Bounds3 {
    fn from(p: Point3) -> Self {
        Self { min: p, max: p }
    }
}

// INDEXING

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
