use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use crate::{
    base::{constants::Float, transform::Transform},
    geometries::{normal::Normal, vec3::Vec3},
    utils::math::{gamma, next_down, next_up},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

impl Point3 {
    pub fn new(x: Float, y: Float, z: Float) -> Self {
        debug_assert!(!x.is_nan() && !y.is_nan() && !z.is_nan());
        Self { x, y, z }
    }

    pub fn lerp(t: Float, a: &Self, b: &Self) -> Self {
        (1.0 - t) * a + t * b
    }

    pub fn offset_ray_origin(&self, p_error: &Vec3, n: &Normal, w: &Vec3) -> Self {
        let d = Vec3::from(n.abs()).dot(p_error);
        let mut offset = d * Vec3::from(*n);
        if w.dot(&Vec3::from(*n)) < 0.0 {
            offset = -offset;
        }
        let mut p = self + &offset;
        for i in 0..3 {
            if offset[i] > 0.0 {
                p[i] = next_up(p[i]);
            } else if offset[i] < 0.0 {
                p[i] = next_down(p[i]);
            }
        }
        p
    }

    pub fn transform(&self, t: &Transform) -> Self {
        // Compute transformed coordinates from point.
        let x = t.m.m[0][0] * self.x + t.m.m[0][1] * self.y + t.m.m[0][2] * self.z + t.m.m[0][3];
        let y = t.m.m[1][0] * self.x + t.m.m[1][1] * self.y + t.m.m[1][2] * self.z + t.m.m[1][3];
        let z = t.m.m[2][0] * self.x + t.m.m[2][1] * self.y + t.m.m[2][2] * self.z + t.m.m[2][3];
        let w = t.m.m[3][0] * self.x + t.m.m[3][1] * self.y + t.m.m[3][2] * self.z + t.m.m[3][3];

        // Perform nonhomogeneous conversion.
        debug_assert_ne!(w, 0.0);
        if w == 1.0 {
            Point3::new(x, y, z)
        } else {
            Point3::new(x, y, z) / w
        }
    }

    pub fn transform_with_error(&self, t: &Transform, error: &mut Vec3) -> Self {
        // Compute transformed coordinates from point.
        let x = t.m.m[0][0] * self.x + t.m.m[0][1] * self.y + t.m.m[0][2] * self.z + t.m.m[0][3];
        let y = t.m.m[1][0] * self.x + t.m.m[1][1] * self.y + t.m.m[1][2] * self.z + t.m.m[1][3];
        let z = t.m.m[2][0] * self.x + t.m.m[2][1] * self.y + t.m.m[2][2] * self.z + t.m.m[2][3];
        let w = t.m.m[3][0] * self.x + t.m.m[3][1] * self.y + t.m.m[3][2] * self.z + t.m.m[3][3];

        // Compute absolute error for transformed point.
        let x_abs_sum = ((t.m.m[0][0] * self.x).abs()
            + (t.m.m[0][1] * self.y).abs()
            + (t.m.m[0][2] * self.z).abs()
            + (t.m.m[0][3]))
            .abs();
        let y_abs_sum = ((t.m.m[1][0] * self.x).abs()
            + (t.m.m[1][1] * self.y).abs()
            + (t.m.m[1][2] * self.z).abs()
            + (t.m.m[1][3]))
            .abs();
        let z_abs_sum = ((t.m.m[2][0] * self.x).abs()
            + (t.m.m[2][1] * self.y).abs()
            + (t.m.m[2][2] * self.z).abs()
            + (t.m.m[2][3]))
            .abs();
        *error = gamma(3.0) * Vec3::new(x_abs_sum, y_abs_sum, z_abs_sum);

        // Perform nonhomogeneous conversion.
        debug_assert_ne!(w, 0.0);
        if w == 1.0 {
            Point3::new(x, y, z)
        } else {
            Point3::new(x, y, z) / w
        }
    }

    pub fn transform_with_point_error(
        &self,
        t: &Transform,
        p_error: &Vec3,
        abs_error: &mut Vec3,
    ) -> Point3 {
        // Compute transformed coordinates from point.
        let x = t.m.m[0][0] * self.x + t.m.m[0][1] * self.y + t.m.m[0][2] * self.z + t.m.m[0][3];
        let y = t.m.m[1][0] * self.x + t.m.m[1][1] * self.y + t.m.m[1][2] * self.z + t.m.m[1][3];
        let z = t.m.m[2][0] * self.x + t.m.m[2][1] * self.y + t.m.m[2][2] * self.z + t.m.m[2][3];
        let w = t.m.m[3][0] * self.x + t.m.m[3][1] * self.y + t.m.m[3][2] * self.z + t.m.m[3][3];

        abs_error.x = (gamma(3.0) + 1.0)
            * (t.m.m[0][0].abs() * p_error.x
                + t.m.m[0][1].abs() * p_error.y
                + t.m.m[0][2].abs() * p_error.z)
            + gamma(3.0)
                * ((t.m.m[0][0] * self.x).abs()
                    + (t.m.m[0][1] * self.y).abs()
                    + (t.m.m[0][2] * self.z).abs()
                    + (t.m.m[0][3]).abs());
        abs_error.y = (gamma(3.0) + 1.0)
            * ((t.m.m[1][0]).abs() * p_error.x
                + (t.m.m[1][1]).abs() * p_error.y
                + (t.m.m[1][2]).abs() * p_error.z)
            + gamma(3.0)
                * ((t.m.m[1][0] * self.x).abs()
                    + (t.m.m[1][1] * self.y).abs()
                    + (t.m.m[1][2] * self.z).abs()
                    + (t.m.m[1][3]).abs());
        abs_error.z = (gamma(3.0) + 1.0)
            * ((t.m.m[2][0]).abs() * p_error.x
                + (t.m.m[2][1]).abs() * p_error.y
                + (t.m.m[2][2]).abs() * p_error.z)
            + gamma(3.0)
                * ((t.m.m[2][0] * self.x).abs()
                    + (t.m.m[2][1] * self.y).abs()
                    + (t.m.m[2][2] * self.z).abs()
                    + (t.m.m[2][3]).abs());

        // Perform nonhomogeneous conversion.
        debug_assert_ne!(w, 0.0);
        if w == 1.0 {
            Point3::new(x, y, z)
        } else {
            Point3::new(x, y, z) / w
        }
    }

    pub fn distance_squared(&self, p: &Self) -> Float {
        (self - p).length_squared()
    }

    pub fn distance(&self, p: &Self) -> Float {
        (self - p).length()
    }

    pub fn length_squared(&self) -> Float {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> Float {
        self.length_squared().sqrt()
    }

    pub fn floor(&self) -> Self {
        Self::new(self.x.floor(), self.y.floor(), self.z.floor())
    }

    pub fn ceil(&self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil(), self.z.ceil())
    }

    pub fn min(&self, p: &Self) -> Self {
        Self::new(self.x.min(p.x), self.y.min(p.y), self.z.min(p.z))
    }

    pub fn max(&self, p: &Self) -> Self {
        Self::new(self.x.max(p.x), self.y.max(p.y), self.z.max(p.z))
    }

    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    pub fn permute(&self, i: usize, j: usize, k: usize) -> Self {
        Self::new(self[i], self[j], self[k])
    }

    pub fn is_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
    }
}

impl Default for Point3 {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl From<Vec3> for Point3 {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl Add for Point3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add for &Point3 {
    type Output = Point3;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<Vec3> for Point3 {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<&Vec3> for &Point3 {
    type Output = Point3;

    fn add(self, rhs: &Vec3) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Point3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl AddAssign<Vec3> for Point3 {
    fn add_assign(&mut self, rhs: Vec3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Point3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub for &Point3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<Vec3> for Point3 {
    type Output = Self;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<&Vec3> for &Point3 {
    type Output = Point3;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl SubAssign for Point3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl SubAssign<Vec3> for Point3 {
    fn sub_assign(&mut self, rhs: Vec3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<Float> for Point3 {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Float> for &Point3 {
    type Output = Point3;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Point3> for Float {
    type Output = Point3;

    fn mul(self, rhs: Point3) -> Self::Output {
        rhs * self
    }
}

impl Mul<&Point3> for Float {
    type Output = Point3;

    fn mul(self, rhs: &Point3) -> Self::Output {
        rhs * self
    }
}

impl MulAssign<Float> for Point3 {
    fn mul_assign(&mut self, rhs: Float) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<Float> for Point3 {
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

impl Div<Float> for &Point3 {
    type Output = Point3;

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

impl DivAssign<Float> for Point3 {
    fn div_assign(&mut self, rhs: Float) {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        self.x *= inverse;
        self.y *= inverse;
        self.z *= inverse;
    }
}

impl Neg for Point3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Neg for &Point3 {
    type Output = Point3;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Index<usize> for Point3 {
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

impl IndexMut<usize> for Point3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < 3);
        if index == 0 {
            &mut self.x
        } else if index == 1 {
            &mut self.y
        } else {
            &mut self.z
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{constants::Float, transform::Transform},
        geometries::{point3::Point3, vec3::Vec3},
    };

    #[test]
    fn new() {
        let p = Point3::new(1.0, 3.0, 4.0);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 3.0);
        assert_eq!(p.z, 4.0);
    }

    #[test]
    fn lerp() {
        let a = Point3::new(1.0, 2.0, 5.0);
        let b = Point3::new(2.0, 4.0, 2.0);
        let c = Point3::new(1.25, 2.5, 4.25);
        let t = 0.25;
        assert_eq!(Point3::lerp(t, &a, &b), c);
    }

    #[test]
    fn transform() {
        let t = Transform::translate(&Vec3::new(2.0, 3.0, 6.0));
        let a = Point3::new(4.0, 2.0, 1.0);
        let b = Point3::new(6.0, 5.0, 7.0);
        assert_eq!(a.transform(&t), b);
    }

    #[test]
    fn distance_squared() {
        let a = Point3::new(1.0, 2.0, 3.0);
        let b = Point3::new(2.0, 4.0, 1.0);
        let x = 9.0;
        assert_eq!(a.distance_squared(&b), x);
    }

    #[test]
    fn distance() {
        let a = Point3::new(1.0, 2.0, 3.0);
        let b = Point3::new(2.0, 4.0, 1.0);
        let x = (9.0 as Float).sqrt();
        assert_eq!(a.distance(&b), x);
    }

    #[test]
    fn length_squared() {
        let a = Point3::new(1.0, 2.0, 3.0);
        let x = 14.0;
        assert_eq!(a.length_squared(), x);
    }

    #[test]
    fn length() {
        let a = Point3::new(1.0, 2.0, 3.0);
        let x = (14.0 as Float).sqrt();
        assert_eq!(a.length(), x);
    }

    #[test]
    fn floor() {
        let a = Point3::new(3.5, 2.0, 1.8);
        let b = Point3::new(3.0, 2.0, 1.0);
        assert_eq!(a.floor(), b);
    }

    #[test]
    fn ceil() {
        let a = Point3::new(3.5, 2.0, 1.1);
        let b = Point3::new(4.0, 2.0, 2.0);
        assert_eq!(a.ceil(), b);
    }

    #[test]
    fn min() {
        let a = Point3::new(3.5, 2.0, 6.8);
        let b = Point3::new(4.0, 1.0, 6.6);
        let c = Point3::new(3.5, 1.0, 6.6);
        assert_eq!(a.min(&b), c);
    }

    #[test]
    fn max() {
        let a = Point3::new(3.5, 2.0, 3.0);
        let b = Point3::new(4.0, 1.0, 1.0);
        let c = Point3::new(4.0, 2.0, 3.0);
        assert_eq!(a.max(&b), c);
    }

    #[test]
    fn abs() {
        let a = Point3::new(-4.0, 2.0, -3.0);
        let b = Point3::new(4.0, 2.0, 3.0);
        assert_eq!(a.abs(), b);
    }

    #[test]
    fn permute() {
        let a = Point3::new(4.0, 2.0, 3.0);
        let b = Point3::new(2.0, 3.0, 4.0);
        assert_eq!(a.permute(1, 2, 0), b);
    }

    #[test]
    fn is_nan() {
        let mut a = Point3::new(4.0, 2.0, 3.0);
        assert_ne!(a.is_nan(), true);
        a.y = Float::NAN;
        assert_eq!(a.is_nan(), true);
    }

    #[test]
    fn default() {
        let p = Point3::default();
        assert_eq!(p.x, 0.0);
        assert_eq!(p.y, 0.0);
        assert_eq!(p.z, 0.0);
    }

    #[test]
    fn from() {
        let v = Vec3::new(3.0, 4.0, 2.0);
        let p = Point3::new(3.0, 4.0, 2.0);
        assert_eq!(Point3::from(v), p);
    }

    #[test]
    fn add() {
        let a = Point3::new(3.0, 4.0, 2.0);
        let b = Point3::new(2.0, 1.0, 3.0);
        let v = Vec3::new(2.0, 1.0, 3.0);
        let c = Point3::new(5.0, 5.0, 5.0);
        assert_eq!(a + b, c);
        assert_eq!(&a + &b, c);
        assert_eq!(a + v, c);
        assert_eq!(&a + &v, c);
    }

    #[test]
    fn add_assign() {
        let mut p = Point3::new(3.0, 4.0, 2.0);
        p += Point3::new(2.0, 1.0, 3.0);
        p += Vec3::new(2.0, 3.0, 1.0);
        assert_eq!(p, Point3::new(7.0, 8.0, 6.0));
    }

    #[test]
    fn sub() {
        let a = Point3::new(3.0, 4.0, 1.0);
        let b = Point3::new(2.0, 1.0, -2.0);
        let c = Vec3::new(2.0, 1.0, -2.0);
        let v = Vec3::new(1.0, 3.0, 3.0);
        let p = Point3::new(1.0, 3.0, 3.0);
        assert_eq!(a - b, v);
        assert_eq!(&a - &b, v);
        assert_eq!(a - c, p);
        assert_eq!(&a - &c, p);
    }

    #[test]
    fn sub_assign() {
        let mut p = Point3::new(3.0, 4.0, 1.0);
        p -= Point3::new(2.0, 1.0, -2.0);
        p -= Vec3::new(2.0, 3.0, 4.0);
        assert_eq!(p, Point3::new(-1.0, 0.0, -1.0));
    }

    #[test]
    fn mul() {
        let a = Point3::new(3.0, 4.0, 1.0);
        let x = 4.0;
        let b = Point3::new(12.0, 16.0, 4.0);
        assert_eq!(a * x, b);
        assert_eq!(&a * x, b);
        assert_eq!(x * a, b);
        assert_eq!(x * &a, b);
    }

    #[test]
    fn mul_assign() {
        let mut a = Point3::new(3.0, 4.0, 1.0);
        a *= 4.0;
        assert_eq!(a, Point3::new(12.0, 16.0, 4.0));
    }

    #[test]
    fn div() {
        let a = Point3::new(3.0, 4.0, 2.0);
        let x = 4.0;
        let b = Point3::new(0.75, 1.0, 0.5);
        assert_eq!(a / x, b);
        assert_eq!(&a / x, b);
    }

    #[test]
    fn div_assign() {
        let mut a = Point3::new(3.0, 4.0, 1.0);
        a /= 4.0;
        assert_eq!(a, Point3::new(0.75, 1.0, 0.25));
    }

    #[test]
    fn neg() {
        let a = Point3::new(3.0, 4.0, -2.0);
        let b = Point3::new(-3.0, -4.0, 2.0);
        assert_eq!(-a, b);
        assert_eq!(-(&a), b);
    }

    #[test]
    fn index() {
        let a = Point3::new(3.0, 4.0, 1.0);
        assert_eq!(a[0], 3.0);
        assert_eq!(a[1], 4.0);
        assert_eq!(a[2], 1.0);
    }

    #[test]
    fn index_mut() {
        let mut a = Point3::new(3.0, 4.0, 1.0);
        assert_eq!(a[0], 3.0);
        a[0] += 1.0;
        assert_eq!(a[0], 4.0);
    }
}
