use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use crate::{base::constants::Float, geometries::vec2::Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2<T> {
    pub x: T,
    pub y: T,
}

pub type Point2I = Point2<i32>;
pub type Point2F = Point2<Float>;

impl<T> Point2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Point2I {
    pub fn lerp(t: i32, a: &Self, b: &Self) -> Self {
        (1 - t) * a + t * b
    }
}

impl Point2F {
    pub fn lerp(t: Float, a: &Self, b: &Self) -> Self {
        (1.0 - t) * a + t * b
    }

    pub fn distance_squared(&self, p: &Self) -> Float {
        (self - p).length_squared()
    }

    pub fn distance(&self, p: &Self) -> Float {
        (self - p).length()
    }

    pub fn length_squared(&self) -> Float {
        self.x * self.x + self.y * self.y
    }

    pub fn length(&self) -> Float {
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

impl Default for Point2I {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Default for Point2F {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl<T> From<Vec2<T>> for Point2<T> {
    fn from(v: Vec2<T>) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Point2I> for Point2F {
    fn from(p: Point2I) -> Self {
        Self {
            x: p.x as Float,
            y: p.y as Float,
        }
    }
}

impl<T: Add<Output = T>> Add for Point2<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Copy + Add<Output = T>> Add for &Point2<T> {
    type Output = Point2<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Add<Output = T>> Add<Vec2<T>> for Point2<T> {
    type Output = Self;

    fn add(self, rhs: Vec2<T>) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Copy + Add<Output = T>> Add<&Vec2<T>> for &Point2<T> {
    type Output = Point2<T>;

    fn add(self, rhs: &Vec2<T>) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: AddAssign> AddAssign for Point2<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: AddAssign> AddAssign<Vec2<T>> for Point2<T> {
    fn add_assign(&mut self, rhs: Vec2<T>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: Sub<Output = T>> Sub for Point2<T> {
    type Output = Vec2<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Copy + Sub<Output = T>> Sub for &Point2<T> {
    type Output = Vec2<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Sub<Output = T>> Sub<Vec2<T>> for Point2<T> {
    type Output = Self;

    fn sub(self, rhs: Vec2<T>) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Copy + Sub<Output = T>> Sub<&Vec2<T>> for &Point2<T> {
    type Output = Point2<T>;

    fn sub(self, rhs: &Vec2<T>) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: SubAssign> SubAssign for Point2<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: SubAssign> SubAssign<Vec2<T>> for Point2<T> {
    fn sub_assign(&mut self, rhs: Vec2<T>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<i32> for Point2I {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<i32> for &Point2I {
    type Output = Point2I;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Float> for Point2F {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Float> for &Point2F {
    type Output = Point2F;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Point2I> for i32 {
    type Output = Point2I;

    fn mul(self, rhs: Point2I) -> Self::Output {
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Mul<&Point2I> for i32 {
    type Output = Point2I;

    fn mul(self, rhs: &Point2I) -> Self::Output {
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Mul<Point2F> for Float {
    type Output = Point2F;

    fn mul(self, rhs: Point2F) -> Self::Output {
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Mul<&Point2F> for Float {
    type Output = Point2F;

    fn mul(self, rhs: &Point2F) -> Self::Output {
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl MulAssign<Float> for Point2F {
    fn mul_assign(&mut self, rhs: Float) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl MulAssign<i32> for Point2I {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<T: Copy + Div<Output = T>> Div<T> for Point2<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T: Copy + Div<Output = T>> Div<T> for &Point2<T> {
    type Output = Point2<T>;

    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T: Copy + DivAssign> DivAssign<T> for Point2<T> {
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl<T: Neg<Output = T>> Neg for Point2<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T: Copy + Neg<Output = T>> Neg for &Point2<T> {
    type Output = Point2<T>;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> Index<usize> for Point2<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < 2);
        if index == 0 {
            &self.x
        } else {
            &self.y
        }
    }
}

impl<T> IndexMut<usize> for Point2<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < 2);
        if index == 0 {
            &mut self.x
        } else {
            &mut self.y
        }
    }
}
