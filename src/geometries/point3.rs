use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use crate::{
    base::transform::Transform,
    geometries::{normal::Normal, vec3::Vec3},
    utils::math::{next_down, next_up, Float},
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
        let xp = t.m.m[0][0] * self.x + t.m.m[0][1] * self.y + t.m.m[0][2] * self.z + t.m.m[0][3];
        let yp = t.m.m[1][0] * self.x + t.m.m[1][1] * self.y + t.m.m[1][2] * self.z + t.m.m[1][3];
        let zp = t.m.m[2][0] * self.x + t.m.m[2][1] * self.y + t.m.m[2][2] * self.z + t.m.m[2][3];
        let wp = t.m.m[3][0] * self.x + t.m.m[3][1] * self.y + t.m.m[3][2] * self.z + t.m.m[3][3];

        if wp == 1.0 {
            Point3::new(xp, yp, zp)
        } else {
            Point3::new(xp, yp, zp) / wp
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
        Self::new(self.x.ceil(), self.y.ceil(), self.z.floor())
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

    pub fn permute(&self, x: usize, y: usize, z: usize) -> Self {
        Self::new(self[x], self[y], self[z])
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
