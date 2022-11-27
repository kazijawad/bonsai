use std::ops;

use num::traits::{cast, Signed};

use crate::geometries::point3::Point3;
use crate::geometries::vec2::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2<T> {
    pub x: T,
    pub y: T,
}

pub type Point2F = Point2<f32>;
pub type Point2I = Point2<i32>;

impl<T: Copy + PartialOrd + Signed + cast::AsPrimitive<f32>> Point2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Default for Point2F {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Default for Point2I {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl<T> From<Vec2<T>> for Point2<T> {
    fn from(v: Vec2<T>) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl<T> From<Point3<T>> for Point2<T> {
    fn from(point: Point3<T>) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl<T: ops::Add<Output = T>> ops::Add for Point2<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: ops::AddAssign> ops::AddAssign for Point2<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for Point2<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: ops::SubAssign> ops::SubAssign for Point2<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: Copy + ops::Mul<Output = T>> ops::Mul<T> for Point2<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T: Copy + ops::MulAssign> ops::MulAssign<T> for Point2<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<T: Copy + ops::Div<Output = T>> ops::Div<T> for Point2<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T: Copy + ops::DivAssign> ops::DivAssign<T> for Point2<T> {
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl<T: ops::Neg<Output = T>> ops::Neg for Point2<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> ops::Index<u32> for Point2<T> {
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        assert!(index <= 1);
        if index == 0 {
            &self.x
        } else {
            &self.y
        }
    }
}
