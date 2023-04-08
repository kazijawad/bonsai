use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use crate::{base::constants::Float, geometries::vec2::Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: Float,
    pub y: Float,
}

impl Point2 {
    pub fn new(x: Float, y: Float) -> Self {
        debug_assert!(!x.is_nan() && !y.is_nan());
        Self { x, y }
    }

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

impl Default for Point2 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl From<Vec2> for Point2 {
    fn from(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl Add for Point2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add for &Point2 {
    type Output = Point2;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<Vec2> for Point2 {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<&Vec2> for &Point2 {
    type Output = Point2;

    fn add(self, rhs: &Vec2) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Point2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<Vec2> for Point2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Point2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub for &Point2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<Vec2> for Point2 {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<&Vec2> for &Point2 {
    type Output = Point2;

    fn sub(self, rhs: &Vec2) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Point2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl SubAssign<Vec2> for Point2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<Float> for Point2 {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Float> for &Point2 {
    type Output = Point2;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Point2> for Float {
    type Output = Point2;

    fn mul(self, rhs: Point2) -> Self::Output {
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Mul<&Point2> for Float {
    type Output = Point2;

    fn mul(self, rhs: &Point2) -> Self::Output {
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl MulAssign<Float> for Point2 {
    fn mul_assign(&mut self, rhs: Float) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<Float> for Point2 {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            x: self.x * inverse,
            y: self.y * inverse,
        }
    }
}

impl Div<Float> for &Point2 {
    type Output = Point2;

    fn div(self, rhs: Float) -> Self::Output {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            x: self.x * inverse,
            y: self.y * inverse,
        }
    }
}

impl DivAssign<Float> for Point2 {
    fn div_assign(&mut self, rhs: Float) {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        self.x *= inverse;
        self.y *= inverse;
    }
}

impl Neg for Point2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Neg for &Point2 {
    type Output = Point2;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Index<usize> for Point2 {
    type Output = Float;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < 2);
        if index == 0 {
            &self.x
        } else {
            &self.y
        }
    }
}

impl IndexMut<usize> for Point2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < 2);
        if index == 0 {
            &mut self.x
        } else {
            &mut self.y
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::constants::Float,
        geometries::{point2::Point2, vec2::Vec2},
    };

    #[test]
    fn new() {
        let p = Point2::new(1.0, 3.0);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 3.0);
    }

    #[test]
    fn lerp() {
        let a = Point2::new(1.0, 2.0);
        let b = Point2::new(2.0, 4.0);
        let c = Point2::new(1.25, 2.5);
        let t = 0.25;
        assert_eq!(Point2::lerp(t, &a, &b), c);
    }

    #[test]
    fn distance_squared() {
        let a = Point2::new(1.0, 2.0);
        let b = Point2::new(2.0, 4.0);
        let x = 5.0;
        assert_eq!(a.distance_squared(&b), x);
    }

    #[test]
    fn distance() {
        let a = Point2::new(1.0, 2.0);
        let b = Point2::new(2.0, 4.0);
        let x = (5.0 as Float).sqrt();
        assert_eq!(a.distance(&b), x);
    }

    #[test]
    fn length_squared() {
        let a = Point2::new(1.0, 2.0);
        let x = 5.0;
        assert_eq!(a.length_squared(), x);
    }

    #[test]
    fn length() {
        let a = Point2::new(1.0, 2.0);
        let x = (5.0 as Float).sqrt();
        assert_eq!(a.length(), x);
    }

    #[test]
    fn floor() {
        let a = Point2::new(3.5, 2.0);
        let b = Point2::new(3.0, 2.0);
        assert_eq!(a.floor(), b);
    }

    #[test]
    fn ceil() {
        let a = Point2::new(3.5, 2.0);
        let b = Point2::new(4.0, 2.0);
        assert_eq!(a.ceil(), b);
    }

    #[test]
    fn min() {
        let a = Point2::new(3.5, 2.0);
        let b = Point2::new(4.0, 1.0);
        let c = Point2::new(3.5, 1.0);
        assert_eq!(a.min(&b), c);
    }

    #[test]
    fn max() {
        let a = Point2::new(3.5, 2.0);
        let b = Point2::new(4.0, 1.0);
        let c = Point2::new(4.0, 2.0);
        assert_eq!(a.max(&b), c);
    }

    #[test]
    fn default() {
        let p = Point2::default();
        assert_eq!(p.x, 0.0);
        assert_eq!(p.y, 0.0);
    }

    #[test]
    fn from() {
        let v = Vec2::new(3.0, 4.0);
        let p = Point2::new(3.0, 4.0);
        assert_eq!(Point2::from(v), p);
    }

    #[test]
    fn add() {
        let a = Point2::new(3.0, 4.0);
        let b = Point2::new(2.0, 1.0);
        let v = Vec2::new(2.0, 1.0);
        let c = Point2::new(5.0, 5.0);
        assert_eq!(a + b, c);
        assert_eq!(&a + &b, c);
        assert_eq!(a + v, c);
        assert_eq!(&a + &v, c);
    }

    #[test]
    fn add_assign() {
        let mut p = Point2::new(3.0, 4.0);
        p += Point2::new(2.0, 1.0);
        p += Vec2::new(2.0, 3.0);
        assert_eq!(p, Point2::new(7.0, 8.0));
    }

    #[test]
    fn sub() {
        let a = Point2::new(3.0, 4.0);
        let b = Point2::new(2.0, 1.0);
        let c = Vec2::new(2.0, 1.0);
        let v = Vec2::new(1.0, 3.0);
        let p = Point2::new(1.0, 3.0);
        assert_eq!(a - b, v);
        assert_eq!(&a - &b, v);
        assert_eq!(a - c, p);
        assert_eq!(&a - &c, p);
    }

    #[test]
    fn sub_assign() {
        let mut p = Point2::new(3.0, 4.0);
        p -= Point2::new(2.0, 1.0);
        p -= Vec2::new(2.0, 3.0);
        assert_eq!(p, Point2::new(-1.0, 0.0));
    }

    #[test]
    fn mul() {
        let a = Point2::new(3.0, 4.0);
        let x = 4.0;
        let b = Point2::new(12.0, 16.0);
        assert_eq!(a * x, b);
        assert_eq!(&a * x, b);
        assert_eq!(x * a, b);
        assert_eq!(x * &a, b);
    }

    #[test]
    fn mul_assign() {
        let mut a = Point2::new(3.0, 4.0);
        a *= 4.0;
        assert_eq!(a, Point2::new(12.0, 16.0));
    }

    #[test]
    fn div() {
        let a = Point2::new(3.0, 4.0);
        let x = 4.0;
        let b = Point2::new(0.75, 1.0);
        assert_eq!(a / x, b);
        assert_eq!(&a / x, b);
    }

    #[test]
    fn div_assign() {
        let mut a = Point2::new(3.0, 4.0);
        a /= 4.0;
        assert_eq!(a, Point2::new(0.75, 1.0));
    }

    #[test]
    fn neg() {
        let a = Point2::new(3.0, 4.0);
        let b = Point2::new(-3.0, -4.0);
        assert_eq!(-a, b);
        assert_eq!(-(&a), b);
    }

    #[test]
    fn index() {
        let a = Point2::new(3.0, 4.0);
        assert_eq!(a[0], 3.0);
        assert_eq!(a[1], 4.0);
    }
}
