use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use crate::{
    base::{
        constants::Float,
        math::{gamma, next_float_down, next_float_up},
        transform::{AnimatedTransform, Transform},
    },
    geometries::{normal::Normal, vec3::Vec3},
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

    pub fn offset_ray_origin(&self, point_error: &Vec3, normal: &Normal, direction: &Vec3) -> Self {
        let normal = Vec3::from(*normal);

        let mut offset = normal.abs().dot(point_error) * normal;
        if direction.dot(&normal) < 0.0 {
            offset = -offset;
        }

        let mut point = self + &offset;
        for i in 0..3 {
            if offset[i] > 0.0 {
                point[i] = next_float_up(point[i]);
            } else if offset[i] < 0.0 {
                point[i] = next_float_down(point[i]);
            }
        }

        point
    }

    pub fn transform(&self, t: &Transform) -> Self {
        let x = self.x;
        let y = self.y;
        let z = self.z;

        // Compute transformed coordinates from point.
        let xp = t.m.m[0][0] * x + t.m.m[0][1] * y + t.m.m[0][2] * z + t.m.m[0][3];
        let yp = t.m.m[1][0] * x + t.m.m[1][1] * y + t.m.m[1][2] * z + t.m.m[1][3];
        let zp = t.m.m[2][0] * x + t.m.m[2][1] * y + t.m.m[2][2] * z + t.m.m[2][3];
        let wp = t.m.m[3][0] * x + t.m.m[3][1] * y + t.m.m[3][2] * z + t.m.m[3][3];

        // Perform nonhomogeneous conversion.
        debug_assert_ne!(wp, 0.0);
        if wp == 1.0 {
            Self::new(xp, yp, zp)
        } else {
            Self::new(xp, yp, zp) / wp
        }
    }

    pub fn transform_with_error(&self, t: &Transform, error: &mut Vec3) -> Self {
        let x = self.x;
        let y = self.y;
        let z = self.z;

        // Compute transformed coordinates from point.
        let xp = (t.m.m[0][0] * x + t.m.m[0][1] * y) + (t.m.m[0][2] * z + t.m.m[0][3]);
        let yp = (t.m.m[1][0] * x + t.m.m[1][1] * y) + (t.m.m[1][2] * z + t.m.m[1][3]);
        let zp = (t.m.m[2][0] * x + t.m.m[2][1] * y) + (t.m.m[2][2] * z + t.m.m[2][3]);
        let wp = (t.m.m[3][0] * x + t.m.m[3][1] * y) + (t.m.m[3][2] * z + t.m.m[3][3]);

        // Compute absolute error for transformed point.
        let x_abs_sum = (t.m.m[0][0] * x).abs()
            + (t.m.m[0][1] * y).abs()
            + (t.m.m[0][2] * z).abs()
            + (t.m.m[0][3]).abs();
        let y_abs_sum = (t.m.m[1][0] * x).abs()
            + (t.m.m[1][1] * y).abs()
            + (t.m.m[1][2] * z).abs()
            + (t.m.m[1][3]).abs();
        let z_abs_sum = (t.m.m[2][0] * x).abs()
            + (t.m.m[2][1] * y).abs()
            + (t.m.m[2][2] * z).abs()
            + (t.m.m[2][3]).abs();
        *error = gamma(3.0) * Vec3::new(x_abs_sum, y_abs_sum, z_abs_sum);

        // Perform nonhomogeneous conversion.
        debug_assert_ne!(wp, 0.0);
        if wp == 1.0 {
            Self::new(xp, yp, zp)
        } else {
            Self::new(xp, yp, zp) / wp
        }
    }

    pub fn transform_with_point_error(
        &self,
        t: &Transform,
        point_error: &Vec3,
        abs_error: &mut Vec3,
    ) -> Self {
        let x = self.x;
        let y = self.y;
        let z = self.z;

        // Compute transformed coordinates from point.
        let xp = (t.m.m[0][0] * x + t.m.m[0][1] * y) + (t.m.m[0][2] * z + t.m.m[0][3]);
        let yp = (t.m.m[1][0] * x + t.m.m[1][1] * y) + (t.m.m[1][2] * z + t.m.m[1][3]);
        let zp = (t.m.m[2][0] * x + t.m.m[2][1] * y) + (t.m.m[2][2] * z + t.m.m[2][3]);
        let wp = (t.m.m[3][0] * x + t.m.m[3][1] * y) + (t.m.m[3][2] * z + t.m.m[3][3]);

        abs_error.x = (gamma(3.0) + 1.0)
            * (t.m.m[0][0].abs() * point_error.x
                + t.m.m[0][1].abs() * point_error.y
                + t.m.m[0][2].abs() * point_error.z)
            + gamma(3.0)
                * ((t.m.m[0][0] * x).abs()
                    + (t.m.m[0][1] * y).abs()
                    + (t.m.m[0][2] * z).abs()
                    + (t.m.m[0][3]).abs());
        abs_error.y = (gamma(3.0) + 1.0)
            * ((t.m.m[1][0]).abs() * point_error.x
                + (t.m.m[1][1]).abs() * point_error.y
                + (t.m.m[1][2]).abs() * point_error.z)
            + gamma(3.0)
                * ((t.m.m[1][0] * x).abs()
                    + (t.m.m[1][1] * y).abs()
                    + (t.m.m[1][2] * z).abs()
                    + (t.m.m[1][3]).abs());
        abs_error.z = (gamma(3.0) + 1.0)
            * ((t.m.m[2][0]).abs() * point_error.x
                + (t.m.m[2][1]).abs() * point_error.y
                + (t.m.m[2][2]).abs() * point_error.z)
            + gamma(3.0)
                * ((t.m.m[2][0] * x).abs()
                    + (t.m.m[2][1] * y).abs()
                    + (t.m.m[2][2] * z).abs()
                    + (t.m.m[2][3]).abs());

        // Perform nonhomogeneous conversion.
        debug_assert_ne!(wp, 0.0);
        if wp == 1.0 {
            Self::new(xp, yp, zp)
        } else {
            Self::new(xp, yp, zp) / wp
        }
    }

    pub fn animated_transform(&self, at: &AnimatedTransform, time: Float) -> Self {
        if !at.is_animated || time <= at.start_time {
            self.transform(&at.start_transform)
        } else if time >= at.end_time {
            self.transform(&at.end_transform)
        } else {
            let mut t = Transform::default();
            at.interpolate(time, &mut t);
            self.transform(&t)
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
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl Mul<&Point3> for Float {
    type Output = Point3;

    fn mul(self, rhs: &Point3) -> Self::Output {
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
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
