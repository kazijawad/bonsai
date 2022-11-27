use std::ops;

use crate::geometries::{normal::Normal, point3::Point3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        debug_assert!(!x.is_nan() && !y.is_nan() && !z.is_nan());
        Self { x, y, z }
    }

    pub fn permute(v: &Self, x: u32, y: u32, z: u32) -> Self {
        Self {
            x: v[x],
            y: v[y],
            z: v[z],
        }
    }

    pub fn coordinate_system(v1: &Self) -> (Self, Self) {
        let v2 = if v1.x.abs() > v1.y.abs() {
            Vec3::new(-v1.z, 0.0, v1.x) / (v1.x * v1.x + v1.z * v1.z).sqrt()
        } else {
            Vec3::new(0.0, v1.z, -v1.y) / (v1.y * v1.y + v1.z * v1.z).sqrt()
        };
        let v3 = Vec3::cross(v1, &v2);
        (v2, v3)
    }

    pub fn dot(v: &Self, w: &Self) -> f32 {
        debug_assert!(!v.is_nan() && !w.is_nan());
        v.x * w.x + v.y * w.y + v.z * w.z
    }

    pub fn abs_dot(v: &Self, w: &Self) -> f32 {
        debug_assert!(!v.is_nan() && !w.is_nan());
        Self::dot(v, w).abs()
    }

    pub fn cross(v: &Self, w: &Self) -> Self {
        let vx: f64 = v.x.into();
        let vy: f64 = v.y.into();
        let vz: f64 = v.z.into();

        let wx: f64 = w.x.into();
        let wy: f64 = w.y.into();
        let wz: f64 = w.z.into();

        Self {
            x: (vy * wz - vz * wy) as f32,
            y: (vz * wx - vx * wz) as f32,
            z: (vx * wy - vy * wx) as f32,
        }
    }

    pub fn normalize(v: &Self) -> Self {
        v / v.length()
    }

    pub fn abs(v: &Self) -> Self {
        Self {
            x: v.x.abs(),
            y: v.y.abs(),
            z: v.z.abs(),
        }
    }

    pub fn min(v: &Self, w: &Self) -> Self {
        Self {
            x: v.x.min(w.x),
            y: v.y.min(w.y),
            z: v.z.min(w.z),
        }
    }

    pub fn max(v: &Self, w: &Self) -> Self {
        Self {
            x: v.x.max(w.x),
            y: v.y.max(w.y),
            z: v.z.max(w.z),
        }
    }

    pub fn min_component(v: &Self) -> f32 {
        v.x.min(v.y.min(v.z))
    }

    pub fn max_component(v: &Self) -> f32 {
        v.x.max(v.y.max(v.z))
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

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl From<Point3> for Vec3 {
    fn from(point: Point3) -> Self {
        Self {
            x: point.x,
            y: point.y,
            z: point.z,
        }
    }
}

impl From<Normal> for Vec3 {
    fn from(normal: Normal) -> Self {
        Self {
            x: normal.x,
            y: normal.y,
            z: normal.z,
        }
    }
}

impl ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl ops::Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            x: self.x * inverse,
            y: self.y * inverse,
            z: self.z * inverse,
        }
    }
}

impl ops::Div<f32> for &Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            x: self.x * inverse,
            y: self.y * inverse,
            z: self.z * inverse,
        }
    }
}

impl ops::DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        self.x *= inverse;
        self.y *= inverse;
        self.z *= inverse;
    }
}

impl ops::Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::Index<u32> for Vec3 {
    type Output = f32;

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
