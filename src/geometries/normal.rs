use std::ops;

use crate::{geometries::vec3::Vec3, math::Float};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Normal {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

impl Normal {
    pub fn new(x: Float, y: Float, z: Float) -> Self {
        debug_assert!(!x.is_nan() && !y.is_nan() && !z.is_nan());
        Self { x, y, z }
    }

    pub fn face_forward(&self, n: &Self) -> Self {
        if self.dot(n) < 0.0 {
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

    pub fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
    }
}

// TYPE CONVERSION

impl From<Vec3> for Normal {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

// ADDITION

impl ops::Add for Normal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::Add for &Normal {
    type Output = Normal;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::AddAssign for Normal {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

// SUBTRACTION

impl ops::Sub for Normal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Sub for &Normal {
    type Output = Normal;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::SubAssign for Normal {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

// MULTIPLICATION

impl ops::Mul<Float> for Normal {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Mul<Float> for &Normal {
    type Output = Normal;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Mul<Normal> for Float {
    type Output = Normal;

    fn mul(self, rhs: Normal) -> Self::Output {
        rhs * self
    }
}

impl ops::Mul<&Normal> for Float {
    type Output = Normal;

    fn mul(self, rhs: &Normal) -> Self::Output {
        rhs * self
    }
}

impl ops::MulAssign<Float> for Normal {
    fn mul_assign(&mut self, rhs: Float) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

// DIVISION

impl ops::Div<Float> for Normal {
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

impl ops::Div<Float> for &Normal {
    type Output = Normal;

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

impl ops::DivAssign<Float> for Normal {
    fn div_assign(&mut self, rhs: Float) {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        self.x *= inverse;
        self.y *= inverse;
        self.z *= inverse;
    }
}

// NEGATION

impl ops::Neg for Normal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::Neg for &Normal {
    type Output = Normal;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// INDEXING

impl ops::Index<u32> for Normal {
    type Output = Float;

    fn index(&self, index: u32) -> &Self::Output {
        debug_assert!(index <= 2);
        if index == 0 {
            return &self.x;
        } else if index == 1 {
            return &self.y;
        }
        &self.z
    }
}
