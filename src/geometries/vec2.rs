use std::ops;

use num::traits::{cast, Signed};

use crate::geometries::point2::Point2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

pub type Vec2F = Vec2<f32>;
pub type Vec2I = Vec2<i32>;

impl<T: Copy + PartialOrd + Signed + cast::AsPrimitive<f32>> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn permute(v: &Self, x: u32, y: u32) -> Self {
        Self { x: v[x], y: v[y] }
    }

    pub fn abs(v: &Self) -> Self {
        Self {
            x: v.x.abs(),
            y: v.y.abs(),
        }
    }

    pub fn dot(v: &Self, w: &Self) -> T {
        v.x * w.x + v.y * w.y
    }

    pub fn normalize(v: &Self) -> Vec2F {
        let vx: f32 = v.x.as_();
        let vy: f32 = v.y.as_();
        let v_length = v.length();
        Vec2F {
            x: vx / v_length,
            y: vy / v_length,
        }
    }

    pub fn abs_dot(v: &Self, w: &Self) -> T {
        Self::dot(v, w).abs()
    }

    pub fn min(v: &Self, w: &Self) -> Self {
        let x = if v.x < w.x { v.x } else { w.x };
        let y = if v.y < w.y { v.y } else { w.y };
        Self { x, y }
    }

    pub fn max(v: &Self, w: &Self) -> Self {
        let x = if v.x > w.x { v.x } else { w.x };
        let y = if v.y > w.y { v.y } else { w.y };
        Self { x, y }
    }

    pub fn min_component(v: &Self) -> T {
        if v.x < v.y {
            v.x
        } else {
            v.y
        }
    }

    pub fn max_component(v: &Self) -> T {
        if v.x > v.y {
            v.x
        } else {
            v.y
        }
    }

    pub fn max_dimension(v: &Self) -> usize {
        if v.x > v.y {
            0
        } else {
            1
        }
    }

    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> f32 {
        let squared_length: f32 = self.length_squared().as_();
        squared_length.sqrt()
    }
}

impl Default for Vec2F {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Default for Vec2I {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl<T> From<Point2<T>> for Vec2<T> {
    fn from(point: Point2<T>) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl<T: ops::Add<Output = T>> ops::Add for Vec2<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: ops::AddAssign> ops::AddAssign for Vec2<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for Vec2<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: ops::SubAssign> ops::SubAssign for Vec2<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: Copy + ops::Mul<Output = T>> ops::Mul<T> for Vec2<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T: Copy + ops::MulAssign> ops::MulAssign<T> for Vec2<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<T: Copy + ops::Div<Output = T>> ops::Div<T> for Vec2<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T: Copy + ops::DivAssign> ops::DivAssign<T> for Vec2<T> {
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl<T: ops::Neg<Output = T>> ops::Neg for Vec2<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> ops::Index<u32> for Vec2<T> {
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
