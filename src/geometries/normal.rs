use std::ops;

use crate::geometries::vec3::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Normal {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Normal {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        debug_assert!(!x.is_nan() && !y.is_nan() && !z.is_nan());
        Self { x, y, z }
    }

    pub fn face_forward(n: &Self, v: &Vec3) -> Self {
        if Self::dot(n, &Self::from(v)) < 0.0 {
            -n.clone()
        } else {
            n.clone()
        }
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn dot(v: &Self, w: &Self) -> f32 {
        debug_assert!(!v.is_nan() && !w.is_nan());
        v.x * w.x + v.y * w.y + v.z * w.z
    }

    pub fn abs_dot(v: &Self, w: &Self) -> f32 {
        debug_assert!(!v.is_nan() && !w.is_nan());
        Self::dot(v, w).abs()
    }

    pub fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
    }
}

impl From<Vec3> for Normal {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<&Vec3> for Normal {
    fn from(v: &Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

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

impl ops::Index<u32> for Normal {
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
