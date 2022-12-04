use std::ops;

use crate::{
    geometries::{mat4::Mat4, transform::Transform, vec3::Vec3},
    math,
    math::Float,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub v: Vec3,
    pub w: Float,
}

impl Quaternion {
    pub fn slerp(t: Float, a: &Self, b: &Self) -> Self {
        let cos_theta = a.dot(&b);
        if cos_theta > 0.9995 {
            ((1.0 - t) * a + t * b).normalize()
        } else {
            let theta = math::clamp(cos_theta, -1.0, 1.0).cos();
            let theta_p = theta * t;
            let q_perp = (b - &(a * cos_theta)).normalize();
            a * theta_p.cos() + q_perp * theta_p.sin()
        }
    }

    pub fn dot(&self, q: &Self) -> Float {
        self.v.dot(&q.v) + self.w * q.w
    }

    pub fn normalize(&self) -> Self {
        self / self.dot(&self.clone()).sqrt()
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        Self {
            v: Vec3::default(),
            w: 1.0,
        }
    }
}

// TYPE CONVERSION

impl From<Transform> for Quaternion {
    fn from(t: Transform) -> Self {
        let mut quat = Quaternion::default();

        let m = t.m;
        let trace = m.m[0][0] + m.m[1][1] + m.m[2][2];

        if trace > 0.0 {
            // Compute w from matrix trace, then xyz.
            let mut s = (trace + 1.0).sqrt();
            quat.w = s / 2.0;

            s = 0.5 / s;
            quat.v.x = (m.m[2][1] - m.m[1][2]) * s;
            quat.v.y = (m.m[0][2] - m.m[2][0]) * s;
            quat.v.z = (m.m[1][0] - m.m[0][1]) * s;
        } else {
            // Compute largest of x, y, z, then remaining components.
            let next = [1, 2, 0];
            let mut q = [0.0; 3];

            let mut i = 0;
            if m.m[1][1] > m.m[0][0] {
                i = 1;
            }
            if m.m[2][2] > m.m[i][i] {
                i = 2;
            }

            let j = next[i];
            let k = next[j];
            let mut s = ((m.m[i][i] - (m.m[j][j] + m.m[k][k])) + 1.0).sqrt();
            q[i] = s * 0.5;

            if s != 0.0 {
                s = 0.5 / s;
            }

            quat.w = (m.m[k][j] - m.m[j][k]) * s;

            q[j] = (m.m[j][i] + m.m[i][j]) * s;
            q[k] = (m.m[k][i] + m.m[i][k]) * s;

            quat.v.x = q[0];
            quat.v.y = q[1];
            quat.v.z = q[2];
        }

        quat
    }
}

impl From<Mat4> for Quaternion {
    fn from(m: Mat4) -> Self {
        let mut quat = Quaternion::default();
        let trace = m.m[0][0] + m.m[1][1] + m.m[2][2];

        if trace > 0.0 {
            // Compute w from matrix trace, then xyz.
            let mut s = (trace + 1.0).sqrt();
            quat.w = s / 2.0;

            s = 0.5 / s;
            quat.v.x = (m.m[2][1] - m.m[1][2]) * s;
            quat.v.y = (m.m[0][2] - m.m[2][0]) * s;
            quat.v.z = (m.m[1][0] - m.m[0][1]) * s;
        } else {
            // Compute largest of x, y, z, then remaining components.
            let next = [1, 2, 0];
            let mut q = [0.0; 3];

            let mut i = 0;
            if m.m[1][1] > m.m[0][0] {
                i = 1;
            }
            if m.m[2][2] > m.m[i][i] {
                i = 2;
            }

            let j = next[i];
            let k = next[j];
            let mut s = ((m.m[i][i] - (m.m[j][j] + m.m[k][k])) + 1.0).sqrt();
            q[i] = s * 0.5;

            if s != 0.0 {
                s = 0.5 / s;
            }

            quat.w = (m.m[k][j] - m.m[j][k]) * s;

            q[j] = (m.m[j][i] + m.m[i][j]) * s;
            q[k] = (m.m[k][i] + m.m[i][k]) * s;

            quat.v.x = q[0];
            quat.v.y = q[1];
            quat.v.z = q[2];
        }

        quat
    }
}

// ADDITION

impl ops::Add for Quaternion {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            v: self.v + rhs.v,
            w: self.w + rhs.w,
        }
    }
}

impl ops::Add for &Quaternion {
    type Output = Quaternion;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            v: self.v + rhs.v,
            w: self.w + rhs.w,
        }
    }
}

impl ops::AddAssign for Quaternion {
    fn add_assign(&mut self, rhs: Self) {
        self.v += rhs.v;
        self.w += rhs.w;
    }
}

// SUBTRACTION

impl ops::Sub for Quaternion {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            v: self.v - rhs.v,
            w: self.w - rhs.w,
        }
    }
}

impl ops::Sub for &Quaternion {
    type Output = Quaternion;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            v: self.v - rhs.v,
            w: self.w - rhs.w,
        }
    }
}

impl ops::SubAssign for Quaternion {
    fn sub_assign(&mut self, rhs: Self) {
        self.v -= rhs.v;
        self.w -= rhs.w;
    }
}

// MULTIPLICATION

impl ops::Mul<Float> for Quaternion {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            v: self.v * rhs,
            w: self.w * rhs,
        }
    }
}

impl ops::Mul<Float> for &Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Float) -> Self::Output {
        Self::Output {
            v: self.v * rhs,
            w: self.w * rhs,
        }
    }
}

impl ops::Mul<Quaternion> for Float {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Self::Output {
        rhs * self
    }
}

impl ops::Mul<&Quaternion> for Float {
    type Output = Quaternion;

    fn mul(self, rhs: &Quaternion) -> Self::Output {
        rhs * self
    }
}

impl ops::MulAssign<Float> for Quaternion {
    fn mul_assign(&mut self, rhs: Float) {
        self.v *= rhs;
        self.w *= rhs;
    }
}

// DIVISION

impl ops::Div<Float> for Quaternion {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            v: self.v * inverse,
            w: self.w * inverse,
        }
    }
}

impl ops::Div<Float> for &Quaternion {
    type Output = Quaternion;

    fn div(self, rhs: Float) -> Self::Output {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            v: self.v * inverse,
            w: self.w * inverse,
        }
    }
}

impl ops::DivAssign<Float> for Quaternion {
    fn div_assign(&mut self, rhs: Float) {
        debug_assert!(rhs != 0.0);
        let inverse = 1.0 / rhs;
        self.v *= inverse;
        self.w *= inverse;
    }
}

// NEGATION

impl ops::Neg for Quaternion {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            v: -self.v,
            w: -self.w,
        }
    }
}

impl ops::Neg for &Quaternion {
    type Output = Quaternion;

    fn neg(self) -> Self::Output {
        Self::Output {
            v: -self.v,
            w: -self.w,
        }
    }
}
