use std::ops;

use crate::{
    base::constants::{Float, PI},
    geometries::{normal::Normal, point3::Point3},
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

    pub fn coordinate_system(v1: &Self, v2: &mut Self, v3: &mut Self) {
        let new_v2 = if v1.x.abs() > v1.y.abs() {
            Self::new(-v1.z, 0.0, v1.x) / (v1.x * v1.x + v1.z * v1.z).sqrt()
        } else {
            Self::new(0.0, v1.z, -v1.y) / (v1.y * v1.y + v1.z * v1.z).sqrt()
        };
        v2.clone_from(&new_v2);
        v3.clone_from(&v1.cross(&v2));
    }

    pub fn spherical_direction(sin_theta: Float, cos_theta: Float, phi: Float) -> Self {
        Self::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta)
    }

    pub fn spherical_direction_from_basis(
        sin_theta: Float,
        cos_theta: Float,
        phi: Float,
        x: &Self,
        y: &Self,
        z: &Self,
    ) -> Self {
        sin_theta * phi.cos() * x + sin_theta * phi.sin() * y + cos_theta * z
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
        let x: f64 = self.x.into();
        let y: f64 = self.y.into();
        let z: f64 = self.z.into();

        let vx: f64 = v.x.into();
        let vy: f64 = v.y.into();
        let vz: f64 = v.z.into();

        Self::new(
            (y * vz - z * vy) as Float,
            (z * vx - x * vz) as Float,
            (x * vy - y * vx) as Float,
        )
    }

    pub fn dot(&self, v: &Self) -> Float {
        debug_assert!(!self.is_nan() && !v.is_nan());
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn abs_dot(&self, v: &Self) -> Float {
        debug_assert!(!self.is_nan() && !v.is_nan());
        self.dot(v).abs()
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
