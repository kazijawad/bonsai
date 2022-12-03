use std::ops;

use crate::geometries::{point3::Point3, vec2::Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Self {
        debug_assert!(!x.is_nan() && !y.is_nan());
        Self { x, y }
    }

    pub fn lerp(t: f32, a: &Self, b: &Self) -> Self {
        (1.0 - t) * a + t * b
    }

    pub fn distance_squared(&self, p: &Self) -> f32 {
        (self - p).length_squared()
    }

    pub fn distance(&self, p: &Self) -> f32 {
        (self - p).length()
    }

    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn floor(&self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    pub fn ceil(&self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil())
    }

    pub fn min(&self, p: &Self) -> Self {
        Self::new(self.x.min(p.x), self.y.min(p.y))
    }

    pub fn max(&self, p: &Self) -> Self {
        Self::new(self.x.max(p.x), self.y.max(p.y))
    }
}

impl Default for Point2 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

// TYPE CONVERSION

impl From<Vec2> for Point2 {
    fn from(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Point3> for Point2 {
    fn from(p: Point3) -> Self {
        Self { x: p.x, y: p.y }
    }
}

// ADDITION

impl ops::Add for Point2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Add for &Point2 {
    type Output = Point2;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Add<Vec2> for Point2 {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Add<&Vec2> for &Point2 {
    type Output = Point2;

    fn add(self, rhs: &Vec2) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::AddAssign for Point2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::AddAssign<Vec2> for Point2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

// SUBTRACTION

impl ops::Sub for Point2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::Sub for &Point2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::Sub<Vec2> for Point2 {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::Sub<&Vec2> for &Point2 {
    type Output = Point2;

    fn sub(self, rhs: &Vec2) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::SubAssign for Point2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl ops::SubAssign<Vec2> for Point2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

// MULTIPLICATION

impl ops::Mul<f32> for Point2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Mul<f32> for &Point2 {
    type Output = Point2;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Mul<Point2> for f32 {
    type Output = Point2;

    fn mul(self, rhs: Point2) -> Self::Output {
        rhs * self
    }
}

impl ops::Mul<&Point2> for f32 {
    type Output = Point2;

    fn mul(self, rhs: &Point2) -> Self::Output {
        rhs * self
    }
}

impl ops::MulAssign<f32> for Point2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

// DIVISION

impl ops::Div<f32> for Point2 {
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

impl ops::Div<f32> for &Point2 {
    type Output = Point2;

    fn div(self, rhs: f32) -> Self::Output {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            x: self.x * inverse,
            y: self.y * inverse,
        }
    }
}

impl ops::DivAssign<f32> for Point2 {
    fn div_assign(&mut self, rhs: f32) {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        self.x *= inverse;
        self.y *= inverse;
    }
}

// Negation

impl ops::Neg for Point2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl ops::Neg for &Point2 {
    type Output = Point2;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
        }
    }
}

// INDEXING

impl ops::Index<u32> for Point2 {
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
