use std::ops;

use crate::{
    base::{
        constants::{Float, PI},
        transform::{AnimatedTransform, Transform},
    },
    geometries::{normal::Normal, point3::Point3},
    utils::math::gamma,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

impl Vec3 {
    pub fn new(x: Float, y: Float, z: Float) -> Self {
        debug_assert!(!x.is_nan() && !y.is_nan() && !z.is_nan());
        Self { x, y, z }
    }

    pub fn coordinate_system(v1: &Self) -> (Self, Self) {
        let v2 = if v1.x.abs() > v1.y.abs() {
            Self::new(-v1.z, 0.0, v1.x) / (v1.x * v1.x + v1.z * v1.z).sqrt()
        } else {
            Self::new(0.0, v1.z, -v1.y) / (v1.y * v1.y + v1.z * v1.z).sqrt()
        };
        let v3 = v1.cross(&v2);
        (v2, v3)
    }

    pub fn transform(&self, t: &Transform, include_error: bool) -> (Self, Option<Self>) {
        let x = self.x;
        let y = self.y;
        let z = self.z;
        let transformed = Vec3::new(
            t.m.m[0][0] * x + t.m.m[0][1] * y + t.m.m[0][2] * z,
            t.m.m[1][0] * x + t.m.m[1][1] * y + t.m.m[1][2] * z,
            t.m.m[2][0] * x + t.m.m[2][1] * y + t.m.m[2][2] * z,
        );
        if include_error {
            let error = Vec3::new(
                gamma(3.0)
                    * ((t.m.m[0][0] * x).abs() + (t.m.m[0][1] * y).abs() + (t.m.m[0][2] * z).abs()),
                gamma(3.0)
                    * ((t.m.m[1][0] * x).abs() + (t.m.m[1][1] * y).abs() + (t.m.m[1][2] * z).abs()),
                gamma(3.0)
                    * ((t.m.m[2][0] * x).abs() + (t.m.m[2][1] * y).abs() + (t.m.m[2][2] * z).abs()),
            );
            (transformed, Some(error))
        } else {
            (transformed, None)
        }
    }

    pub fn animated_transform(self, at: &AnimatedTransform, time: Float) -> Self {
        if !at.is_animated || time <= at.start_time {
            self.transform(&at.start_transform, false).0
        } else if time >= at.end_time {
            self.transform(&at.end_transform, false).0
        } else {
            let mut t = Transform::default();
            at.interpolate(time, &mut t);
            self.transform(&t, false).0
        }
    }

    pub fn spherical_theta(&self) -> Float {
        self.z.clamp(-1.0, 1.0).acos()
    }

    pub fn spherical_phi(&self) -> Float {
        let p = self.y.atan2(self.x);
        if p < 0.0 {
            p + 2.0 * PI
        } else {
            p
        }
    }

    pub fn face_forward(&self, v: &Self) -> Self {
        if self.dot(v) < 0.0 {
            -self.clone()
        } else {
            self.clone()
        }
    }

    pub fn length_squared(&self) -> Float {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> Float {
        self.length_squared().sqrt()
    }

    pub fn cross(&self, v: &Self) -> Self {
        let x = self.x as f64;
        let y = self.y as f64;
        let z = self.z as f64;

        let vx = v.x as f64;
        let vy = v.y as f64;
        let vz = v.z as f64;

        Self::new(
            ((y * vz) - (z * vy)) as Float,
            ((z * vx) - (x * vz)) as Float,
            ((x * vy) - (y * vx)) as Float,
        )
    }

    pub fn dot(&self, v: &Self) -> Float {
        debug_assert!(!self.is_nan() && !v.is_nan());
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn dot_normal(&self, v: &Normal) -> Float {
        debug_assert!(!self.is_nan() && !v.is_nan());
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn abs_dot(&self, v: &Self) -> Float {
        self.dot(v).abs()
    }

    pub fn abs_dot_normal(&self, v: &Normal) -> Float {
        self.dot_normal(v).abs()
    }

    pub fn normalize(&self) -> Self {
        self / self.length()
    }

    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    pub fn min_component(&self) -> Float {
        self.x.min(self.y.min(self.z))
    }

    pub fn max_component(&self) -> Float {
        self.x.max(self.y.max(self.z))
    }

    pub fn min(&self, v: &Self) -> Self {
        Self::new(self.x.min(v.x), self.y.min(v.y), self.z.min(v.z))
    }

    pub fn max(&self, v: &Self) -> Self {
        Self::new(self.x.max(v.x), self.y.max(v.y), self.z.max(v.z))
    }

    pub fn max_dimension(&self) -> usize {
        if self.x > self.y {
            if self.x > self.z {
                0
            } else {
                2
            }
        } else if self.y > self.z {
            1
        } else {
            2
        }
    }

    pub fn permute(&self, x: usize, y: usize, z: usize) -> Self {
        Self::new(self[x], self[y], self[z])
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

impl From<[Float; 3]> for Vec3 {
    fn from(v: [Float; 3]) -> Self {
        Self {
            x: v[0],
            y: v[1],
            z: v[2],
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

impl ops::Add for &Vec3 {
    type Output = Vec3;

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

impl ops::Sub for &Vec3 {
    type Output = Vec3;

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

impl ops::Mul<Float> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Mul<Float> for &Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Mul<Vec3> for Float {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl ops::Mul<&Vec3> for Float {
    type Output = Vec3;

    fn mul(self, rhs: &Vec3) -> Self::Output {
        rhs * self
    }
}

impl ops::MulAssign<Float> for Vec3 {
    fn mul_assign(&mut self, rhs: Float) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl ops::Div<Float> for Vec3 {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            x: self.x * inverse,
            y: self.y * inverse,
            z: self.z * inverse,
        }
    }
}

impl ops::Div<Float> for &Vec3 {
    type Output = Vec3;

    fn div(self, rhs: Float) -> Self::Output {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            x: self.x * inverse,
            y: self.y * inverse,
            z: self.z * inverse,
        }
    }
}

impl ops::DivAssign<Float> for Vec3 {
    fn div_assign(&mut self, rhs: Float) {
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

impl ops::Neg for &Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::Index<usize> for Vec3 {
    type Output = Float;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < 3);
        if index == 0 {
            &self.x
        } else if index == 1 {
            &self.y
        } else {
            &self.z
        }
    }
}
