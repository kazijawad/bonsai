use std::ops;

use crate::geometries::{point2::Point2, point3::Point3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        debug_assert!(!x.is_nan() && !y.is_nan());
        Self { x, y }
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn dot(&self, v: &Self) -> f32 {
        debug_assert!(!self.is_nan() && !v.is_nan());
        self.x * v.x + self.y * v.y
    }

    pub fn abs_dot(&self, v: &Self) -> f32 {
        debug_assert!(!self.is_nan() && !v.is_nan());
        self.dot(v).abs()
    }

    pub fn normalize(&self) -> Self {
        self / self.length()
    }

    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    pub fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

// TYPE CONVERSION

impl From<Point2> for Vec2 {
    fn from(point: Point2) -> Self {
        debug_assert!(!point.x.is_nan() && !point.y.is_nan());
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<Point3> for Vec2 {
    fn from(point: Point3) -> Self {
        debug_assert!(!point.x.is_nan() && !point.y.is_nan());
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

// ADDITION

impl ops::Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Add for &Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

// SUBTRACTION

impl ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::Sub for &Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

// MULTIPLICATION

impl ops::Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Mul<f32> for &Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        rhs * self
    }
}

impl ops::Mul<&Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: &Vec2) -> Self::Output {
        rhs * self
    }
}

impl ops::MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

// DIVISION

impl ops::Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            x: self.x * inverse,
            y: self.y * inverse,
        }
    }
}

impl ops::Div<f32> for &Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f32) -> Self::Output {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            x: self.x * inverse,
            y: self.y * inverse,
        }
    }
}

impl ops::DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        self.x *= inverse;
        self.y *= inverse;
    }
}

// NEGATION

impl ops::Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl ops::Neg for &Vec2 {
    type Output = Vec2;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
        }
    }
}

// INDEXING

impl ops::Index<u32> for Vec2 {
    type Output = f32;

    fn index(&self, index: u32) -> &Self::Output {
        debug_assert!(index <= 1);
        if index == 0 {
            &self.x
        } else {
            &self.y
        }
    }
}