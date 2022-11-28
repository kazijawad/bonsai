use std::ops;

use crate::geometries::{point2::Point2, vec2::Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds2 {
    min: Point2,
    max: Point2,
}

impl Bounds2 {
    pub fn new(a: &Point2, b: &Point2) -> Self {
        Self {
            min: Point2::new(a.x.min(b.x), a.y.min(b.y)),
            max: Point2::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    pub fn diagonal(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn area(&self) -> f32 {
        let distance = self.max - self.min;
        distance.x * distance.y
    }

    pub fn maximum_extent(&self) -> u32 {
        let diag = self.diagonal();
        if diag.x > diag.y {
            0
        } else {
            1
        }
    }

    pub fn offset(&self, p: &Point2) -> Vec2 {
        let mut offset = *p - self.min;
        if self.max.x > self.min.x {
            offset.x /= self.max.x - self.min.x;
        }
        if self.max.y > self.min.y {
            offset.y /= self.max.y - self.min.y;
        }
        offset
    }
}

impl Default for Bounds2 {
    fn default() -> Self {
        Self {
            min: Point2::new(f32::MAX, f32::MAX),
            max: Point2::new(f32::MIN, f32::MIN),
        }
    }
}

impl ops::Index<u32> for Bounds2 {
    type Output = Point2;

    fn index(&self, index: u32) -> &Self::Output {
        debug_assert!(index == 0 || index == 1);
        if index == 0 {
            &self.min
        } else {
            &self.max
        }
    }
}
