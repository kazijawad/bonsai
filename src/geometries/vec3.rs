use std::ops;

use num::traits::{cast, Signed};

use crate::geometries::point3::Point3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub type Vec3D = Vec3<f64>;
pub type Vec3F = Vec3<f32>;
pub type Vec3I = Vec3<i32>;

impl<T: Copy + PartialOrd + Signed + cast::AsPrimitive<f32> + cast::AsPrimitive<f64>> Vec3<T> {
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

    pub fn coordinate_system(v1: &Vec3D) -> (Vec3D, Vec3D) {
        let v2 = if v1.x.abs() > v1.y.abs() {
            Vec3::new(-v1.z, 0.0, v1.x) / (v1.x * v1.x + v1.z * v1.z).sqrt()
        } else {
            Vec3::new(0.0, v1.z, -v1.y) / (v1.y * v1.y + v1.z * v1.z).sqrt()
        };
        let v3 = Vec3::cross(v1, &v2);
        (v2, v3)
    }

    pub fn dot(v: &Self, w: &Self) -> T {
        v.x * w.x + v.y * w.y + v.z * w.z
    }

    pub fn cross(v: &Self, w: &Self) -> Vec3D {
        let vx: f64 = v.x.as_();
        let vy: f64 = v.y.as_();
        let vz: f64 = v.z.as_();

        let wx: f64 = w.x.as_();
        let wy: f64 = w.y.as_();
        let wz: f64 = w.z.as_();

        Vec3D {
            x: vy * wz - vz * wy,
            y: vz * wx - vx * wz,
            z: vx * wy - vy * wx,
        }
    }

    pub fn normalize(v: &Self) -> Vec3F {
        let vx: f32 = v.x.as_();
        let vy: f32 = v.y.as_();
        let vz: f32 = v.z.as_();
        let v_length = v.length();
        Vec3F {
            x: vx / v_length,
            y: vy / v_length,
            z: vz / v_length,
        }
    }

    pub fn abs(v: &Self) -> Self {
        Self {
            x: v.x.abs(),
            y: v.y.abs(),
            z: v.z.abs(),
        }
    }

    pub fn abs_dot(v: &Self, w: &Self) -> T {
        Self::dot(v, w).abs()
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

    pub fn min_component(v: &Self) -> T {
        if v.x < v.y {
            if v.x < v.z {
                v.x
            } else {
                v.z
            }
        } else if v.y < v.z {
            v.y
        } else {
            v.z
        }
    }

    pub fn max_component(v: &Self) -> T {
        if v.x > v.y {
            if v.x > v.z {
                v.x
            } else {
                v.z
            }
        } else if v.y > v.z {
            v.y
        } else {
            v.z
        }
    }

    pub fn max_dimension(v: &Self) -> usize {
        if v.x > v.y {
            if v.x > v.z {
                0
            } else {
                2
            }
        } else if v.y > v.z {
            1
        } else {
            2
        }
    }

    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        let squared_length: f32 = self.length_squared().as_();
        squared_length.sqrt()
    }
}

impl Default for Vec3D {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Default for Vec3F {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Default for Vec3I {
    fn default() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}

impl<T> From<Point3<T>> for Vec3<T> {
    fn from(point: Point3<T>) -> Self {
        Self {
            x: point.x,
            y: point.y,
            z: point.z,
        }
    }
}

impl<T: ops::Add<Output = T>> ops::Add for Vec3<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T: ops::AddAssign> ops::AddAssign for Vec3<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<T: ops::Sub<Output = T>> ops::Sub for Vec3<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T: ops::SubAssign> ops::SubAssign for Vec3<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<T: Copy + ops::Mul<Output = T>> ops::Mul<T> for Vec3<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<T: Copy + ops::MulAssign> ops::MulAssign<T> for Vec3<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl<T: Copy + ops::Div<Output = T>> ops::Div<T> for Vec3<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl<T: Copy + ops::DivAssign> ops::DivAssign<T> for Vec3<T> {
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl<T: ops::Neg<Output = T>> ops::Neg for Vec3<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<T> ops::Index<u32> for Vec3<T> {
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
