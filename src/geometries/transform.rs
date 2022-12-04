use std::{cmp::Ordering, ops};

use crate::{
    float,
    geometries::{
        bounds3::Bounds3,
        mat4::Mat4,
        normal::Normal,
        point3::Point3,
        quaternion::Quaternion,
        ray::{Ray, RayDifferential},
        vec3::Vec3,
    },
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub m: Mat4,
    pub m_inverse: Mat4,
}

impl Transform {
    pub fn new(m: Mat4, m_inverse: Mat4) -> Self {
        Self { m, m_inverse }
    }

    pub fn transform_point(&self, p: &Point3) -> Point3 {
        let x = p.x;
        let y = p.y;
        let z = p.z;

        let xp = self.m.m[0][0] * x + self.m.m[0][1] * y + self.m.m[0][2] * z + self.m.m[0][3];
        let yp = self.m.m[1][0] * x + self.m.m[1][1] * y + self.m.m[1][2] * z + self.m.m[1][3];
        let zp = self.m.m[2][0] * x + self.m.m[2][1] * y + self.m.m[2][2] * z + self.m.m[2][3];
        let wp = self.m.m[3][0] * x + self.m.m[3][1] * y + self.m.m[3][2] * z + self.m.m[3][3];

        if wp == 1.0 {
            Point3::new(xp, yp, zp)
        } else {
            Point3::new(xp, yp, zp) / wp
        }
    }

    pub fn transform_point_with_error(&self, p: &Point3, error: &mut Vec3) -> Point3 {
        let x = p.x;
        let y = p.y;
        let z = p.z;

        // Compute transformed coordinates from point.
        let xp = (self.m.m[0][0] * x + self.m.m[0][1] * y) + (self.m.m[0][2] * z + self.m.m[0][3]);
        let yp = (self.m.m[1][0] * x + self.m.m[1][1] * y) + (self.m.m[1][2] * z + self.m.m[1][3]);
        let zp = (self.m.m[2][0] * x + self.m.m[2][1] * y) + (self.m.m[2][2] * z + self.m.m[2][3]);
        let wp = (self.m.m[3][0] * x + self.m.m[3][1] * y) + (self.m.m[3][2] * z + self.m.m[3][3]);

        // Compute absolute error for transformed point.
        let x_abs_sum = ((self.m.m[0][0] * x).abs()
            + (self.m.m[0][1] * y).abs()
            + (self.m.m[0][2] * z).abs()
            + (self.m.m[0][3]))
            .abs();
        let y_abs_sum = ((self.m.m[1][0] * x).abs()
            + (self.m.m[1][1] * y).abs()
            + (self.m.m[1][2] * z).abs()
            + (self.m.m[1][3]))
            .abs();
        let z_abs_sum = ((self.m.m[2][0] * x).abs()
            + (self.m.m[2][1] * y).abs()
            + (self.m.m[2][2] * z).abs()
            + (self.m.m[2][3]))
            .abs();

        *error = float::gamma(3.0) * Vec3::new(x_abs_sum, y_abs_sum, z_abs_sum);

        debug_assert!(wp != 0.0);
        if wp == 1.0 {
            Point3::new(xp, yp, zp)
        } else {
            Point3::new(xp, yp, zp) / wp
        }
    }

    pub fn transform_vec(&self, v: &Vec3) -> Vec3 {
        let x = v.x;
        let y = v.y;
        let z = v.z;
        Vec3::new(
            self.m.m[0][0] * x + self.m.m[0][1] * y + self.m.m[0][2] * z,
            self.m.m[1][0] * x + self.m.m[1][1] * y + self.m.m[1][2] * z,
            self.m.m[2][0] * x + self.m.m[2][1] * y + self.m.m[2][2] * z,
        )
    }

    pub fn transform_normal(&self, n: &Normal) -> Normal {
        let x = n.x;
        let y = n.y;
        let z = n.z;
        Normal::new(
            self.m_inverse.m[0][0] * x + self.m_inverse.m[1][0] * y + self.m_inverse.m[2][0] * z,
            self.m_inverse.m[0][1] * x + self.m_inverse.m[1][1] * y + self.m_inverse.m[2][1] * z,
            self.m_inverse.m[0][2] * x + self.m_inverse.m[1][2] * y + self.m_inverse.m[2][2] * z,
        )
    }

    pub fn transform_ray(&self, r: &Ray) -> Ray {
        let mut origin_error = Vec3::default();
        let mut origin = self.transform_point_with_error(&r.origin, &mut origin_error);

        let direction = self.transform_vec(&r.direction);
        // Offset ray origin to edge of error bounds and compute max.
        let length_squared = direction.length_squared();
        let mut t_max = r.t_max;

        if length_squared > 0.0 {
            let dt = direction.abs().dot(&origin_error) / length_squared;
            origin += direction * dt;
            t_max -= dt;
        }

        Ray::new(&origin, &direction, t_max, r.time, r.medium)
    }

    pub fn transform_ray_differential(&self, r: &RayDifferential) -> RayDifferential {
        let tr = self.transform_ray(&Ray::from(r.clone()));
        let mut ret = RayDifferential::new(&tr.origin, &tr.direction, tr.t_max, tr.time, tr.medium);
        ret.has_differentials = r.has_differentials;
        ret.rx_origin = self.transform_point(&r.rx_origin);
        ret.ry_origin = self.transform_point(&r.ry_origin);
        ret.rx_direction = self.transform_vec(&r.rx_direction);
        ret.ry_direction = self.transform_vec(&r.ry_direction);
        ret
    }

    pub fn transform_bounds(&self, b: &Bounds3) -> Bounds3 {
        let mut ret = Bounds3::from(self.transform_point(&Point3::new(b.min.x, b.min.y, b.min.z)));
        ret = ret.union_point(&self.transform_point(&Point3::new(b.max.x, b.min.y, b.min.z)));
        ret = ret.union_point(&self.transform_point(&Point3::new(b.min.x, b.max.y, b.min.z)));
        ret = ret.union_point(&self.transform_point(&Point3::new(b.min.x, b.min.y, b.max.z)));
        ret = ret.union_point(&self.transform_point(&Point3::new(b.min.x, b.max.y, b.max.z)));
        ret = ret.union_point(&self.transform_point(&Point3::new(b.max.x, b.max.y, b.min.z)));
        ret = ret.union_point(&self.transform_point(&Point3::new(b.max.x, b.min.y, b.max.z)));
        ret = ret.union_point(&self.transform_point(&Point3::new(b.max.x, b.max.y, b.max.z)));
        ret
    }

    pub fn translate(delta: &Vec3) -> Self {
        let m = Mat4::new(
            1.0, 0.0, 0.0, delta.x, 0.0, 1.0, 0.0, delta.y, 0.0, 0.0, 1.0, delta.z, 0.0, 0.0, 0.0,
            1.0,
        );
        let m_inverse = Mat4::new(
            1.0, 0.0, 0.0, -delta.x, 0.0, 1.0, 0.0, -delta.y, 0.0, 0.0, 1.0, -delta.z, 0.0, 0.0,
            0.0, 1.0,
        );
        Self::new(m, m_inverse)
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        debug_assert!(x != 0.0 && y != 0.0 && z != 0.0);
        let m = Mat4::new(
            x, 0.0, 0.0, 0.0, 0.0, y, 0.0, 0.0, 0.0, 0.0, z, 0.0, 0.0, 0.0, 0.0, 1.0,
        );
        let m_inverse = Mat4::new(
            1.0 / x,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0 / y,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0 / z,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );
        Self::new(m, m_inverse)
    }

    pub fn rotate_x(theta: f32) -> Self {
        let sin_theta = theta.to_radians().sin();
        let cos_theta = theta.to_radians().cos();
        let m = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, cos_theta, -sin_theta, 0.0, 0.0, sin_theta, cos_theta, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        Self::new(m, m.transpose())
    }

    pub fn rotate_y(theta: f32) -> Self {
        let sin_theta = theta.to_radians().sin();
        let cos_theta = theta.to_radians().cos();
        let m = Mat4::new(
            cos_theta, 0.0, sin_theta, 0.0, 0.0, 1.0, 0.0, 0.0, -sin_theta, 0.0, cos_theta, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        Self::new(m, m.transpose())
    }

    pub fn rotate_z(theta: f32) -> Self {
        let sin_theta = theta.to_radians().sin();
        let cos_theta = theta.to_radians().cos();
        let m = Mat4::new(
            cos_theta, -sin_theta, 0.0, 0.0, sin_theta, cos_theta, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        Self::new(m, m.transpose())
    }

    pub fn rotate(theta: f32, axis: &Vec3) -> Self {
        let a = Vec3::normalize(axis);
        let sin_theta = theta.to_radians().sin();
        let cos_theta = theta.to_radians().cos();
        let mut m = Mat4::default();

        m.m[0][0] = a.x * a.x + (1.0 - a.x * a.x) * cos_theta;
        m.m[0][1] = a.x * a.y * (1.0 - cos_theta) - a.z * sin_theta;
        m.m[0][2] = a.x * a.z * (1.0 - cos_theta) + a.y * sin_theta;
        m.m[0][3] = 0.0;

        m.m[1][0] = a.x * a.y * (1.0 - cos_theta) + a.z * sin_theta;
        m.m[1][1] = a.y * a.y + (1.0 - a.y * a.y) * cos_theta;
        m.m[1][2] = a.y * a.z * (1.0 - cos_theta) - a.x * sin_theta;
        m.m[1][3] = 0.0;

        m.m[2][0] = a.x * a.z * (1.0 - cos_theta) - a.y * sin_theta;
        m.m[2][1] = a.y * a.z * (1.0 - cos_theta) + a.x * sin_theta;
        m.m[2][2] = a.z * a.z + (1.0 - a.z * a.z) * cos_theta;
        m.m[2][3] = 0.0;

        Self::new(m, m.transpose())
    }

    pub fn look_at(position: &Point3, look: &Point3, up: &Vec3) -> Self {
        let mut camera_to_world = Mat4::default();

        camera_to_world.m[0][3] = position.x;
        camera_to_world.m[1][3] = position.y;
        camera_to_world.m[2][3] = position.z;
        camera_to_world.m[3][3] = 1.0;

        let direction = look - position;
        let right = Vec3::normalize(&Vec3::cross(&Vec3::normalize(up), &direction));
        let new_up = Vec3::cross(&direction, &right);

        camera_to_world.m[0][0] = right.x;
        camera_to_world.m[1][0] = right.y;
        camera_to_world.m[2][0] = right.z;
        camera_to_world.m[3][0] = 0.0;

        camera_to_world.m[0][1] = new_up.x;
        camera_to_world.m[1][1] = new_up.y;
        camera_to_world.m[2][1] = new_up.z;
        camera_to_world.m[3][1] = 0.0;

        camera_to_world.m[0][2] = direction.x;
        camera_to_world.m[1][2] = direction.y;
        camera_to_world.m[2][2] = direction.z;
        camera_to_world.m[3][2] = 0.0;

        Self::new(camera_to_world.inverse(), camera_to_world)
    }

    pub fn inverse(&self) -> Self {
        Self::new(self.m_inverse, self.m)
    }

    pub fn transpose(&self) -> Self {
        Self::new(self.m.transpose(), self.m_inverse.transpose())
    }

    pub fn is_identity(&self) -> bool {
        self.m.m[0][0] == 1.0
            && self.m.m[0][1] == 0.0
            && self.m.m[0][2] == 0.0
            && self.m.m[0][3] == 0.0
            && self.m.m[1][0] == 0.0
            && self.m.m[1][1] == 1.0
            && self.m.m[1][2] == 0.0
            && self.m.m[1][3] == 0.0
            && self.m.m[2][0] == 0.0
            && self.m.m[2][1] == 0.0
            && self.m.m[2][2] == 1.0
            && self.m.m[2][3] == 0.0
            && self.m.m[3][0] == 0.0
            && self.m.m[3][1] == 0.0
            && self.m.m[3][2] == 0.0
            && self.m.m[3][3] == 1.0
    }

    pub fn swaps_handedness(&self) -> bool {
        let det = self.m.m[0][0]
            * (self.m.m[1][1] * self.m.m[2][2] - self.m.m[1][2] * self.m.m[2][1])
            - self.m.m[0][1] * (self.m.m[1][0] * self.m.m[2][2] - self.m.m[1][2] * self.m.m[2][0])
            + self.m.m[0][2] * (self.m.m[1][0] * self.m.m[2][1] - self.m.m[1][1] * self.m.m[2][0]);
        det < 0.0
    }

    pub fn has_scale(&self) -> bool {
        let la2 = self
            .transform_vec(&Vec3::new(1.0, 0.0, 0.0))
            .length_squared();
        let lb2 = self
            .transform_vec(&Vec3::new(0.0, 1.0, 0.0))
            .length_squared();
        let lc2 = self
            .transform_vec(&Vec3::new(0.0, 0.0, 1.0))
            .length_squared();
        float::not_one(la2) || float::not_one(lb2) || float::not_one(lc2)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            m: Mat4::default(),
            m_inverse: Mat4::default(),
        }
    }
}

// TYPE CONVERSION

impl From<Quaternion> for Transform {
    fn from(q: Quaternion) -> Self {
        let xx = q.v.x * q.v.x;
        let yy = q.v.y * q.v.y;
        let zz = q.v.z * q.v.z;

        let xy = q.v.x * q.v.y;
        let xz = q.v.x * q.v.z;
        let yz = q.v.y * q.v.z;

        let wx = q.v.x * q.w;
        let wy = q.v.y * q.w;
        let wz = q.v.z * q.w;

        let mut m = Mat4::default();
        m.m[0][0] = 1.0 - 2.0 * (yy + zz);
        m.m[0][1] = 2.0 * (xy + wz);
        m.m[0][2] = 2.0 * (xz - wy);
        m.m[1][0] = 2.0 * (xy - wz);
        m.m[1][1] = 1.0 - 2.0 * (xx + zz);
        m.m[1][2] = 2.0 * (yz + wx);
        m.m[2][0] = 2.0 * (xz + wy);
        m.m[2][1] = 2.0 * (yz - wx);
        m.m[2][2] = 1.0 - 2.0 * (xx + yy);

        // Transpose for left-handed.
        Transform::new(m.transpose(), m)
    }
}

// MULTIPLICATION

impl ops::Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Transform::new(
            Mat4::mul(&self.m, &rhs.m),
            Mat4::mul(&rhs.m_inverse, &self.m_inverse),
        )
    }
}

impl ops::Mul for &Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        Transform::new(
            Mat4::mul(&self.m, &rhs.m),
            Mat4::mul(&rhs.m_inverse, &self.m_inverse),
        )
    }
}

// ORDERING

impl PartialOrd for Transform {
    fn lt(&self, other: &Self) -> bool {
        for i in 0..4 {
            for j in 0..4 {
                if self.m.m[i][j] < other.m.m[i][j] {
                    return true;
                }
                if self.m.m[i][j] > other.m.m[i][j] {
                    return false;
                }
            }
        }
        false
    }

    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        None
    }
}
