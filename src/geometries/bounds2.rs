use std::ops::Index;

use crate::{
    base::constants::Float,
    geometries::{
        point2::{Point2, Point2F},
        vec2::{Vec2, Vec2F},
    },
    utils::math,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds2<T> {
    pub min: Point2<T>,
    pub max: Point2<T>,
}

pub type Bounds2I = Bounds2<i32>;
pub type Bounds2F = Bounds2<Float>;

impl Bounds2F {
    pub fn new(a: &Point2F, b: &Point2F) -> Self {
        Self {
            min: Point2::new(a.x.min(b.x), a.y.min(b.y)),
            max: Point2::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    pub fn union(&self, b: &Self) -> Self {
        Self {
            min: self.min.min(&b.min),
            max: self.max.max(&b.max),
        }
    }

    pub fn union_point(&self, p: &Point2F) -> Self {
        Self {
            min: self.min.min(p),
            max: self.max.max(p),
        }
    }

    pub fn intersect(&self, b: &Self) -> Self {
        // Important: Assign min/max without new
        // because new applies min/max on each
        // parameter.
        Self {
            min: self.min.max(&b.min),
            max: self.max.min(&b.max),
        }
    }

    pub fn overlaps(&self, b: &Self) -> bool {
        let x = (self.max.x >= b.min.x) && (self.min.x <= b.max.x);
        let y = (self.max.y >= b.min.y) && (self.min.y <= b.max.y);
        x && y
    }

    pub fn inside(&self, p: &Point2F) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    pub fn inside_exclusive(&self, p: &Point2F) -> bool {
        p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y
    }

    pub fn expand(&self, delta: Float) -> Self {
        Self::new(
            &(self.min - Vec2::new(delta, delta)),
            &(self.max + Vec2::new(delta, delta)),
        )
    }

    pub fn diagonal(&self) -> Vec2F {
        self.max - self.min
    }

    pub fn area(&self) -> Float {
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

    pub fn lerp(&self, t: &Point2F) -> Point2F {
        Point2::new(
            math::lerp(t.x, self.min.x, self.max.x),
            math::lerp(t.y, self.min.y, self.max.y),
        )
    }

    pub fn offset(&self, p: &Point2F) -> Vec2F {
        let mut offset = p - &self.min;
        if self.max.x > self.min.x {
            offset.x /= self.max.x - self.min.x;
        }
        if self.max.y > self.min.y {
            offset.y /= self.max.y - self.min.y;
        }
        offset
    }

    pub fn bounding_sphere(&self) -> (Point2F, Float) {
        let center = (self.min + self.max) / 2.0;
        let radius = if self.inside(&center) {
            center.distance(&self.max)
        } else {
            0.0
        };
        (center, radius)
    }

    pub fn traverse<F>(&self, mut f: F)
    where
        F: FnMut(Point2F),
    {
        let mut point = self.min;
        loop {
            f(point);
            point.x += 1.0;
            if point.x > self.max.x {
                point.x = self.min.x;
                point.y += 1.0;
            }
            if point.y > self.max.y {
                break;
            }
        }
    }
}

impl Default for Bounds2F {
    fn default() -> Self {
        Self {
            min: Point2::new(Float::MAX, Float::MAX),
            max: Point2::new(Float::MIN, Float::MIN),
        }
    }
}

impl<T: Copy> From<Point2<T>> for Bounds2<T> {
    fn from(p: Point2<T>) -> Self {
        Self { min: p, max: p }
    }
}

impl<T> Index<usize> for Bounds2<T> {
    type Output = Point2<T>;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < 2);
        if index == 0 {
            &self.min
        } else {
            &self.max
        }
    }
}
