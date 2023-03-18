use std::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::{
    base::constants::Float,
    geometries::{point3::Point3, vec3::Vec3},
};

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

impl Default for Normal {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
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

impl From<Point3> for Normal {
    fn from(p: Point3) -> Self {
        Self {
            x: p.x,
            y: p.y,
            z: p.z,
        }
    }
}

impl Add for Normal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add for &Normal {
    type Output = Normal;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Normal {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Normal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub for &Normal {
    type Output = Normal;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl SubAssign for Normal {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<Float> for Normal {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Float> for &Normal {
    type Output = Normal;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Normal> for Float {
    type Output = Normal;

    fn mul(self, rhs: Normal) -> Self::Output {
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl Mul<&Normal> for Float {
    type Output = Normal;

    fn mul(self, rhs: &Normal) -> Self::Output {
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl MulAssign<Float> for Normal {
    fn mul_assign(&mut self, rhs: Float) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<Float> for Normal {
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

impl Div<Float> for &Normal {
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

impl DivAssign<Float> for Normal {
    fn div_assign(&mut self, rhs: Float) {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        self.x *= inverse;
        self.y *= inverse;
        self.z *= inverse;
    }
}

impl Neg for Normal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Neg for &Normal {
    type Output = Normal;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Index<usize> for Normal {
    type Output = Float;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < 3);
        if index == 0 {
            return &self.x;
        } else if index == 1 {
            return &self.y;
        }
        &self.z
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        geometries::normal::{Normal, Point3, Vec3},
        utils::math::Float,
    };

    #[test]
    fn new() {
        let n = Normal::new(3.0, 4.0, 7.0);
        assert_eq!(n.x, 3.0);
        assert_eq!(n.y, 4.0);
        assert_eq!(n.z, 7.0);
    }

    #[test]
    fn face_forward() {
        let a = Normal::new(3.0, 4.0, 7.0);
        let b = Normal::new(-3.0, -4.0, -5.0);
        let c = Normal::new(-3.0, -4.0, -7.0);
        assert_eq!(a.face_forward(&b), c);
    }

    #[test]
    fn length_squared() {
        let n = Normal::new(3.0, 4.0, 7.0);
        let x = 74.0;
        assert_eq!(n.length_squared(), x);
    }

    #[test]
    fn length() {
        let n = Normal::new(3.0, 4.0, 7.0);
        let x = 74.0 as Float;
        assert_eq!(n.length(), x.sqrt());
    }

    #[test]
    fn dot() {
        let a = Normal::new(3.0, 4.0, 7.0);
        let b = Normal::new(2.0, 6.0, 1.0);
        let x = 37.0;
        assert_eq!(a.dot(&b), x);
    }

    #[test]
    fn abs_dot() {
        let a = Normal::new(-3.0, -4.0, 7.0);
        let b = Normal::new(-2.0, 6.0, -1.0);
        let x = 25.0;
        assert_eq!(a.dot(&b), -x);
        assert_eq!(a.abs_dot(&b), x);
    }

    #[test]
    fn normalize() {
        let n = Normal::new(3.0, 4.0, 7.0);
        let magnitude = n.length();
        assert_eq!(n.normalize(), n / magnitude);
    }

    #[test]
    fn abs() {
        let a = Normal::new(-3.0, 4.0, 0.0);
        let b = Normal::new(3.0, -4.0, -0.0);
        let c = Normal::new(3.0, 4.0, 0.0);
        assert_eq!(a.abs(), c);
        assert_eq!(b.abs(), c);
    }

    #[test]
    fn is_nan() {
        let mut a = Normal::new(3.0, 2.0, 4.0);
        assert!(!a.is_nan());
        a.x = Float::NAN;
        assert!(a.is_nan());
    }

    #[test]
    fn default() {
        let n = Normal::new(0.0, 0.0, 0.0);
        assert_eq!(n, Normal::default());
    }

    #[test]
    fn from() {
        let n = Normal::new(3.0, 4.0, 7.0);
        let v = Vec3::new(3.0, 4.0, 7.0);
        let p = Point3::new(3.0, 4.0, 7.0);
        assert_eq!(n, Normal::from(v));
        assert_eq!(n, Normal::from(p));
    }

    #[test]
    fn add() {
        let a = Normal::new(3.0, 4.0, 7.0);
        let b = Normal::new(2.0, 1.0, 7.0);
        let c = Normal::new(5.0, 5.0, 14.0);
        assert_eq!(a + b, c);
        assert_eq!(&a + &b, c);
    }

    #[test]
    fn add_assign() {
        let mut a = Normal::new(3.0, 4.0, 7.0);
        let b = Normal::new(2.0, 1.0, 7.0);
        let c = Normal::new(5.0, 5.0, 14.0);
        a += b;
        assert_eq!(a, c);
    }

    #[test]
    fn sub() {
        let a = Normal::new(3.0, 4.0, 7.0);
        let b = Normal::new(2.0, 1.0, 7.0);
        let c = Normal::new(5.0, 5.0, 14.0);
        assert_eq!(c - b, a);
        assert_eq!(&c - &b, a);
    }

    #[test]
    fn sub_assign() {
        let a = Normal::new(3.0, 4.0, 7.0);
        let b = Normal::new(2.0, 1.0, 7.0);
        let mut c = Normal::new(5.0, 5.0, 14.0);
        c -= b;
        assert_eq!(c, a);
    }

    #[test]
    fn mul() {
        let a = Normal::new(3.0, 4.0, 7.0);
        let b = Normal::new(9.0, 12.0, 21.0);
        let x = 3.0;
        assert_eq!(a * x, b);
        assert_eq!(&a * x, b);
        assert_eq!(x * a, b);
        assert_eq!(x * &a, b);
    }

    #[test]
    fn mul_assign() {
        let mut a = Normal::new(3.0, 4.0, 7.0);
        let b = Normal::new(9.0, 12.0, 21.0);
        a *= 3.0;
        assert_eq!(a, b);
    }

    #[test]
    fn div() {
        let a = Normal::new(3.0, 4.0, 7.0);
        let b = Normal::new(9.0, 12.0, 21.0);
        let x = 3.0;
        assert_eq!(b / x, a);
        assert_eq!(&b / x, a);
    }

    #[test]
    fn div_assign() {
        let a = Normal::new(3.0, 4.0, 7.0);
        let mut b = Normal::new(9.0, 12.0, 21.0);
        b /= 3.0;
        assert_eq!(b, a);
    }

    #[test]
    fn neg() {
        let a = Normal::new(3.0, 4.0, 7.0);
        let b = Normal::new(-3.0, -4.0, -7.0);
        assert_eq!(-a, b);
        assert_eq!(-&a, b);
    }

    #[test]
    fn index() {
        let a = Normal::new(3.0, 4.0, 7.0);
        assert_eq!(a[0], 3.0);
        assert_eq!(a[1], 4.0);
        assert_eq!(a[2], 7.0);
    }
}
