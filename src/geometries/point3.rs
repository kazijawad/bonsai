use std::ops;

use num::traits::{cast, Signed};

use crate::geometries::vec3::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub type Point3F = Point3<f32>;
pub type Point3I = Point3<i32>;

impl<T: Copy + PartialOrd + Signed + cast::AsPrimitive<f32> + cast::AsPrimitive<f64>> Point3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub fn permute(v: &Self, x: u32, y: u32, z: u32) -> Self {
        Self {
            x: v[x],
            y: v[y],
            z: v[z],
        }
    }

    pub fn distance_squared(a: &Self, b: &Self) -> T {
        (*a - *b).length_squared()
    }

    pub fn distance(a: &Self, b: &Self) -> f32 {
        (*a - *b).length()
    }

    pub fn abs(v: &Self) -> Self {
        Self {
            x: v.x.abs(),
            y: v.y.abs(),
            z: v.z.abs(),
        }
    }

    pub fn min(v: &Self, w: &Self) -> Self {
        let x = if v.x < w.x { v.x } else { w.x };
        let y = if v.y < w.y { v.y } else { w.y };
        let z = if v.z < w.z { v.z } else { w.z };
        Self { x, y, z }
    }

    pub fn max(v: &Self, w: &Self) -> Self {
        let x = if v.x > w.x { v.x } else { w.x };
        let y = if v.y > w.y { v.y } else { w.y };
        let z = if v.z > w.z { v.z } else { w.z };
        Self { x, y, z }
    }
}

impl Default for Point3F {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Default for Point3I {
    fn default() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}

impl<T> From<Vec3<T>> for Point3<T> {
    fn from(v: Vec3<T>) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl<T: ops::Add<Output = T>> ops::Add for Point3<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T: ops::Add<Output = T>> ops::Add<Vec3<T>> for Point3<T> {
    type Output = Self;

    fn add(self, rhs: Vec3<T>) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T: ops::AddAssign> ops::AddAssign for Point3<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<T: ops::AddAssign> ops::AddAssign<Vec3<T>> for Point3<T> {
    fn add_assign(&mut self, rhs: Vec3<T>) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for Point3<T> {
    type Output = Vec3<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub<Vec3<T>> for Point3<T> {
    type Output = Self;

    fn sub(self, rhs: Vec3<T>) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T: ops::SubAssign> ops::SubAssign for Point3<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<T: ops::SubAssign> ops::SubAssign<Vec3<T>> for Point3<T> {
    fn sub_assign(&mut self, rhs: Vec3<T>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<T: Copy + ops::Mul<Output = T>> ops::Mul<T> for Point3<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<T: Copy + ops::MulAssign> ops::MulAssign<T> for Point3<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl<T: Copy + ops::Div<Output = T>> ops::Div<T> for Point3<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl<T: Copy + ops::DivAssign> ops::DivAssign<T> for Point3<T> {
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl<T: ops::Neg<Output = T>> ops::Neg for Point3<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<T> ops::Index<u32> for Point3<T> {
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        assert!(index <= 2);
        if index == 0 {
            return &self.x;
        } else if index == 1 {
            return &self.y;
        }
        &self.z
    }
}
