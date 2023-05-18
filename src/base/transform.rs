use std::{cmp::Ordering, ops::Mul};

use crate::{
    base::{constants::Float, math::lerp},
    geometries::{
        bounds3::Bounds3, interval::Interval, mat4::Mat4, point3::Point3, quaternion::Quaternion,
        vec3::Vec3,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub m: Mat4,
    pub m_inverse: Mat4,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimatedTransform {
    pub start_transform: Transform,
    pub end_transform: Transform,
    pub start_time: Float,
    pub end_time: Float,
    pub is_animated: bool,
    pub has_rotation: bool,
    translation: Option<Vec<Vec3>>,
    rotation: Option<Vec<Quaternion>>,
    scale: Option<Vec<Mat4>>,
    c1: Option<Vec<DerivativeTerm>>,
    c2: Option<Vec<DerivativeTerm>>,
    c3: Option<Vec<DerivativeTerm>>,
    c4: Option<Vec<DerivativeTerm>>,
    c5: Option<Vec<DerivativeTerm>>,
}

#[derive(Debug, Clone, PartialEq)]
struct DerivativeTerm {
    kc: Float,
    kx: Float,
    ky: Float,
    kz: Float,
}

impl Transform {
    pub fn new(m: Mat4, m_inverse: Mat4) -> Self {
        Self { m, m_inverse }
    }

    pub fn perspective(fov: Float, near: Float, far: Float) -> Self {
        // Perform projective divide for perspective projection.
        let mat = Mat4::new(
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            far / (far - near),
            -far * near / (far - near),
            0.0,
            0.0,
            1.0,
            0.0,
        );
        let inverse_mat = mat.inverse();

        // Scale canonical perspective view to specified field of view.
        let inverse_tan_angle = 1.0 / (fov.to_radians() / 2.0).tan();
        Self::scale(inverse_tan_angle, inverse_tan_angle, 1.0) * Self::new(mat, inverse_mat)
    }

    pub fn transform_bounds(&self, b: &Bounds3) -> Bounds3 {
        let mut ret = Bounds3::from(Point3::new(b.min.x, b.min.y, b.min.z).transform(self));
        ret = ret.union_point(&Point3::new(b.max.x, b.min.y, b.min.z).transform(self));
        ret = ret.union_point(&Point3::new(b.min.x, b.max.y, b.min.z).transform(self));
        ret = ret.union_point(&Point3::new(b.min.x, b.min.y, b.max.z).transform(self));
        ret = ret.union_point(&Point3::new(b.min.x, b.max.y, b.max.z).transform(self));
        ret = ret.union_point(&Point3::new(b.max.x, b.max.y, b.min.z).transform(self));
        ret = ret.union_point(&Point3::new(b.max.x, b.min.y, b.max.z).transform(self));
        ret = ret.union_point(&Point3::new(b.max.x, b.max.y, b.max.z).transform(self));
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

    pub fn scale(x: Float, y: Float, z: Float) -> Self {
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

    pub fn rotate_x(theta: Float) -> Self {
        let sin_theta = theta.to_radians().sin();
        let cos_theta = theta.to_radians().cos();
        let m = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, cos_theta, -sin_theta, 0.0, 0.0, sin_theta, cos_theta, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        let m_transpose = m.transpose();
        Self::new(m, m_transpose)
    }

    pub fn rotate_y(theta: Float) -> Self {
        let sin_theta = theta.to_radians().sin();
        let cos_theta = theta.to_radians().cos();
        let m = Mat4::new(
            cos_theta, 0.0, sin_theta, 0.0, 0.0, 1.0, 0.0, 0.0, -sin_theta, 0.0, cos_theta, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        let m_transpose = m.transpose();
        Self::new(m, m_transpose)
    }

    pub fn rotate_z(theta: Float) -> Self {
        let sin_theta = theta.to_radians().sin();
        let cos_theta = theta.to_radians().cos();
        let m = Mat4::new(
            cos_theta, -sin_theta, 0.0, 0.0, sin_theta, cos_theta, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        );
        let m_transpose = m.transpose();
        Self::new(m, m_transpose)
    }

    pub fn rotate(theta: Float, axis: &Vec3) -> Self {
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

        let m_transpose = m.transpose();
        Self::new(m, m_transpose)
    }

    pub fn look_at(position: &Point3, look: &Point3, up: &Vec3) -> Self {
        let mut camera_to_world = Mat4::default();

        // Initialize fourth column of view matrix.
        camera_to_world.m[0][3] = position.x;
        camera_to_world.m[1][3] = position.y;
        camera_to_world.m[2][3] = position.z;
        camera_to_world.m[3][3] = 1.0;

        // Initialize first three columns of view matrix.
        let direction = (look - position).normalize();
        if up.normalize().cross(&direction).length() == 0.0 {
            eprintln!("Transform::look_at have up and view direction in the same direction. Using the identity transformation.");
            return Self::default();
        }

        let right = up.normalize().cross(&direction).normalize();
        let new_up = direction.cross(&right);

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

        let inv_camera_to_world = camera_to_world.inverse();

        Self::new(camera_to_world, inv_camera_to_world)
    }

    pub fn inverse(&self) -> Self {
        Self::new(self.m_inverse.clone(), self.m.clone())
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
        let la2 = Vec3::new(1.0, 0.0, 0.0).transform(self).length_squared();
        let lb2 = Vec3::new(0.0, 1.0, 0.0).transform(self).length_squared();
        let lc2 = Vec3::new(0.0, 0.0, 1.0).transform(self).length_squared();
        let not_one = |x: Float| -> bool { x < 0.999 || x > 1.001 };
        not_one(la2) || not_one(lb2) || not_one(lc2)
    }
}

impl AnimatedTransform {
    pub fn new(
        start_transform: Transform,
        start_time: Float,
        end_transform: Transform,
        end_time: Float,
    ) -> Self {
        if start_transform == end_transform {
            return Self {
                start_transform,
                start_time,
                end_transform,
                end_time,
                is_animated: false,
                has_rotation: false,
                translation: None,
                rotation: None,
                scale: None,
                c1: None,
                c2: None,
                c3: None,
                c4: None,
                c5: None,
            };
        }

        let mut translation = Vec::with_capacity(2);
        let mut rotation = Vec::with_capacity(2);
        let mut scale = Vec::with_capacity(2);

        let mut c1 = Vec::with_capacity(3);
        let mut c2 = Vec::with_capacity(3);
        let mut c3 = Vec::with_capacity(3);
        let mut c4 = Vec::with_capacity(3);
        let mut c5 = Vec::with_capacity(3);

        let (t, r, s) = Self::decompose(&start_transform.m);
        translation[0] = t;
        rotation[0] = r;
        scale[0] = s;

        let (t, r, s) = Self::decompose(&end_transform.m);
        translation[1] = t;
        rotation[1] = r;
        scale[1] = s;

        // Flip rotation if needed to select shortest path.
        if rotation[0].dot(&rotation[1]) < 0.0 {
            rotation[1] = -rotation[1];
        }
        let has_rotation = rotation[0].dot(&rotation[1]) < 0.9995;

        // Compute terms of motion derivative function.
        if has_rotation {
            let cos_theta = rotation[0].dot(&rotation[1]);
            let theta = cos_theta.clamp(-1.0, 1.0).acos();
            let qperp = (rotation[1] - rotation[0] * cos_theta).normalize();

            let t0x = translation[0].x;
            let t0y = translation[0].y;
            let t0z = translation[0].z;
            let t1x = translation[1].x;
            let t1y = translation[1].y;
            let t1z = translation[1].z;
            let q0x = rotation[0].v.x;
            let q0y = rotation[0].v.y;
            let q0z = rotation[0].v.z;
            let q0w = rotation[0].w;
            let qperpx = qperp.v.x;
            let qperpy = qperp.v.y;
            let qperpz = qperp.v.z;
            let qperpw = qperp.w;
            let s000 = scale[0].m[0][0];
            let s001 = scale[0].m[0][1];
            let s002 = scale[0].m[0][2];
            let s010 = scale[0].m[1][0];
            let s011 = scale[0].m[1][1];
            let s012 = scale[0].m[1][2];
            let s020 = scale[0].m[2][0];
            let s021 = scale[0].m[2][1];
            let s022 = scale[0].m[2][2];
            let s100 = scale[1].m[0][0];
            let s101 = scale[1].m[0][1];
            let s102 = scale[1].m[0][2];
            let s110 = scale[1].m[1][0];
            let s111 = scale[1].m[1][1];
            let s112 = scale[1].m[1][2];
            let s120 = scale[1].m[2][0];
            let s121 = scale[1].m[2][1];
            let s122 = scale[1].m[2][2];

            c1[0] = DerivativeTerm::new(
                -t0x + t1x,
                (-1.0 + q0y * q0y + q0z * q0z + qperpy * qperpy + qperpz * qperpz) * s000
                    + q0w * q0z * s010
                    - qperpx * qperpy * s010
                    + qperpw * qperpz * s010
                    - q0w * q0y * s020
                    - qperpw * qperpy * s020
                    - qperpx * qperpz * s020
                    + s100
                    - q0y * q0y * s100
                    - q0z * q0z * s100
                    - qperpy * qperpy * s100
                    - qperpz * qperpz * s100
                    - q0w * q0z * s110
                    + qperpx * qperpy * s110
                    - qperpw * qperpz * s110
                    + q0w * q0y * s120
                    + qperpw * qperpy * s120
                    + qperpx * qperpz * s120
                    + q0x * (-(q0y * s010) - q0z * s020 + q0y * s110 + q0z * s120),
                (-1.0 + q0y * q0y + q0z * q0z + qperpy * qperpy + qperpz * qperpz) * s001
                    + q0w * q0z * s011
                    - qperpx * qperpy * s011
                    + qperpw * qperpz * s011
                    - q0w * q0y * s021
                    - qperpw * qperpy * s021
                    - qperpx * qperpz * s021
                    + s101
                    - q0y * q0y * s101
                    - q0z * q0z * s101
                    - qperpy * qperpy * s101
                    - qperpz * qperpz * s101
                    - q0w * q0z * s111
                    + qperpx * qperpy * s111
                    - qperpw * qperpz * s111
                    + q0w * q0y * s121
                    + qperpw * qperpy * s121
                    + qperpx * qperpz * s121
                    + q0x * (-(q0y * s011) - q0z * s021 + q0y * s111 + q0z * s121),
                (-1.0 + q0y * q0y + q0z * q0z + qperpy * qperpy + qperpz * qperpz) * s002
                    + q0w * q0z * s012
                    - qperpx * qperpy * s012
                    + qperpw * qperpz * s012
                    - q0w * q0y * s022
                    - qperpw * qperpy * s022
                    - qperpx * qperpz * s022
                    + s102
                    - q0y * q0y * s102
                    - q0z * q0z * s102
                    - qperpy * qperpy * s102
                    - qperpz * qperpz * s102
                    - q0w * q0z * s112
                    + qperpx * qperpy * s112
                    - qperpw * qperpz * s112
                    + q0w * q0y * s122
                    + qperpw * qperpy * s122
                    + qperpx * qperpz * s122
                    + q0x * (-(q0y * s012) - q0z * s022 + q0y * s112 + q0z * s122),
            );

            c2[0] = DerivativeTerm::new(
                0.0,
                -(qperpy * qperpy * s000) - qperpz * qperpz * s000 + qperpx * qperpy * s010
                    - qperpw * qperpz * s010
                    + qperpw * qperpy * s020
                    + qperpx * qperpz * s020
                    + q0y * q0y * (s000 - s100)
                    + q0z * q0z * (s000 - s100)
                    + qperpy * qperpy * s100
                    + qperpz * qperpz * s100
                    - qperpx * qperpy * s110
                    + qperpw * qperpz * s110
                    - qperpw * qperpy * s120
                    - qperpx * qperpz * s120
                    + 2.0 * q0x * qperpy * s010 * theta
                    - 2.0 * q0w * qperpz * s010 * theta
                    + 2.0 * q0w * qperpy * s020 * theta
                    + 2.0 * q0x * qperpz * s020 * theta
                    + q0y
                        * (q0x * (-s010 + s110)
                            + q0w * (-s020 + s120)
                            + 2.0 * (-2.0 * qperpy * s000 + qperpx * s010 + qperpw * s020) * theta)
                    + q0z
                        * (q0w * (s010 - s110) + q0x * (-s020 + s120)
                            - 2.0 * (2.0 * qperpz * s000 + qperpw * s010 - qperpx * s020) * theta),
                -(qperpy * qperpy * s001) - qperpz * qperpz * s001 + qperpx * qperpy * s011
                    - qperpw * qperpz * s011
                    + qperpw * qperpy * s021
                    + qperpx * qperpz * s021
                    + q0y * q0y * (s001 - s101)
                    + q0z * q0z * (s001 - s101)
                    + qperpy * qperpy * s101
                    + qperpz * qperpz * s101
                    - qperpx * qperpy * s111
                    + qperpw * qperpz * s111
                    - qperpw * qperpy * s121
                    - qperpx * qperpz * s121
                    + 2.0 * q0x * qperpy * s011 * theta
                    - 2.0 * q0w * qperpz * s011 * theta
                    + 2.0 * q0w * qperpy * s021 * theta
                    + 2.0 * q0x * qperpz * s021 * theta
                    + q0y
                        * (q0x * (-s011 + s111)
                            + q0w * (-s021 + s121)
                            + 2.0 * (-2.0 * qperpy * s001 + qperpx * s011 + qperpw * s021) * theta)
                    + q0z
                        * (q0w * (s011 - s111) + q0x * (-s021 + s121)
                            - 2.0 * (2.0 * qperpz * s001 + qperpw * s011 - qperpx * s021) * theta),
                -(qperpy * qperpy * s002) - qperpz * qperpz * s002 + qperpx * qperpy * s012
                    - qperpw * qperpz * s012
                    + qperpw * qperpy * s022
                    + qperpx * qperpz * s022
                    + q0y * q0y * (s002 - s102)
                    + q0z * q0z * (s002 - s102)
                    + qperpy * qperpy * s102
                    + qperpz * qperpz * s102
                    - qperpx * qperpy * s112
                    + qperpw * qperpz * s112
                    - qperpw * qperpy * s122
                    - qperpx * qperpz * s122
                    + 2.0 * q0x * qperpy * s012 * theta
                    - 2.0 * q0w * qperpz * s012 * theta
                    + 2.0 * q0w * qperpy * s022 * theta
                    + 2.0 * q0x * qperpz * s022 * theta
                    + q0y
                        * (q0x * (-s012 + s112)
                            + q0w * (-s022 + s122)
                            + 2.0 * (-2.0 * qperpy * s002 + qperpx * s012 + qperpw * s022) * theta)
                    + q0z
                        * (q0w * (s012 - s112) + q0x * (-s022 + s122)
                            - 2.0 * (2.0 * qperpz * s002 + qperpw * s012 - qperpx * s022) * theta),
            );

            c3[0] = DerivativeTerm::new(
                0.0,
                -2.0 * (q0x * qperpy * s010 - q0w * qperpz * s010
                    + q0w * qperpy * s020
                    + q0x * qperpz * s020
                    - q0x * qperpy * s110
                    + q0w * qperpz * s110
                    - q0w * qperpy * s120
                    - q0x * qperpz * s120
                    + q0y
                        * (-2.0 * qperpy * s000
                            + qperpx * s010
                            + qperpw * s020
                            + 2.0 * qperpy * s100
                            - qperpx * s110
                            - qperpw * s120)
                    + q0z
                        * (-2.0 * qperpz * s000 - qperpw * s010
                            + qperpx * s020
                            + 2.0 * qperpz * s100
                            + qperpw * s110
                            - qperpx * s120))
                    * theta,
                -2.0 * (q0x * qperpy * s011 - q0w * qperpz * s011
                    + q0w * qperpy * s021
                    + q0x * qperpz * s021
                    - q0x * qperpy * s111
                    + q0w * qperpz * s111
                    - q0w * qperpy * s121
                    - q0x * qperpz * s121
                    + q0y
                        * (-2.0 * qperpy * s001
                            + qperpx * s011
                            + qperpw * s021
                            + 2.0 * qperpy * s101
                            - qperpx * s111
                            - qperpw * s121)
                    + q0z
                        * (-2.0 * qperpz * s001 - qperpw * s011
                            + qperpx * s021
                            + 2.0 * qperpz * s101
                            + qperpw * s111
                            - qperpx * s121))
                    * theta,
                -2.0 * (q0x * qperpy * s012 - q0w * qperpz * s012
                    + q0w * qperpy * s022
                    + q0x * qperpz * s022
                    - q0x * qperpy * s112
                    + q0w * qperpz * s112
                    - q0w * qperpy * s122
                    - q0x * qperpz * s122
                    + q0y
                        * (-2.0 * qperpy * s002
                            + qperpx * s012
                            + qperpw * s022
                            + 2.0 * qperpy * s102
                            - qperpx * s112
                            - qperpw * s122)
                    + q0z
                        * (-2.0 * qperpz * s002 - qperpw * s012
                            + qperpx * s022
                            + 2.0 * qperpz * s102
                            + qperpw * s112
                            - qperpx * s122))
                    * theta,
            );

            c4[0] = DerivativeTerm::new(
                0.0,
                -(q0x * qperpy * s010) + q0w * qperpz * s010
                    - q0w * qperpy * s020
                    - q0x * qperpz * s020
                    + q0x * qperpy * s110
                    - q0w * qperpz * s110
                    + q0w * qperpy * s120
                    + q0x * qperpz * s120
                    + 2.0 * q0y * q0y * s000 * theta
                    + 2.0 * q0z * q0z * s000 * theta
                    - 2.0 * qperpy * qperpy * s000 * theta
                    - 2.0 * qperpz * qperpz * s000 * theta
                    + 2.0 * qperpx * qperpy * s010 * theta
                    - 2.0 * qperpw * qperpz * s010 * theta
                    + 2.0 * qperpw * qperpy * s020 * theta
                    + 2.0 * qperpx * qperpz * s020 * theta
                    + q0y
                        * (-(qperpx * s010) - qperpw * s020
                            + 2.0 * qperpy * (s000 - s100)
                            + qperpx * s110
                            + qperpw * s120
                            - 2.0 * q0x * s010 * theta
                            - 2.0 * q0w * s020 * theta)
                    + q0z
                        * (2.0 * qperpz * s000 + qperpw * s010
                            - qperpx * s020
                            - 2.0 * qperpz * s100
                            - qperpw * s110
                            + qperpx * s120
                            + 2.0 * q0w * s010 * theta
                            - 2.0 * q0x * s020 * theta),
                -(q0x * qperpy * s011) + q0w * qperpz * s011
                    - q0w * qperpy * s021
                    - q0x * qperpz * s021
                    + q0x * qperpy * s111
                    - q0w * qperpz * s111
                    + q0w * qperpy * s121
                    + q0x * qperpz * s121
                    + 2.0 * q0y * q0y * s001 * theta
                    + 2.0 * q0z * q0z * s001 * theta
                    - 2.0 * qperpy * qperpy * s001 * theta
                    - 2.0 * qperpz * qperpz * s001 * theta
                    + 2.0 * qperpx * qperpy * s011 * theta
                    - 2.0 * qperpw * qperpz * s011 * theta
                    + 2.0 * qperpw * qperpy * s021 * theta
                    + 2.0 * qperpx * qperpz * s021 * theta
                    + q0y
                        * (-(qperpx * s011) - qperpw * s021
                            + 2.0 * qperpy * (s001 - s101)
                            + qperpx * s111
                            + qperpw * s121
                            - 2.0 * q0x * s011 * theta
                            - 2.0 * q0w * s021 * theta)
                    + q0z
                        * (2.0 * qperpz * s001 + qperpw * s011
                            - qperpx * s021
                            - 2.0 * qperpz * s101
                            - qperpw * s111
                            + qperpx * s121
                            + 2.0 * q0w * s011 * theta
                            - 2.0 * q0x * s021 * theta),
                -(q0x * qperpy * s012) + q0w * qperpz * s012
                    - q0w * qperpy * s022
                    - q0x * qperpz * s022
                    + q0x * qperpy * s112
                    - q0w * qperpz * s112
                    + q0w * qperpy * s122
                    + q0x * qperpz * s122
                    + 2.0 * q0y * q0y * s002 * theta
                    + 2.0 * q0z * q0z * s002 * theta
                    - 2.0 * qperpy * qperpy * s002 * theta
                    - 2.0 * qperpz * qperpz * s002 * theta
                    + 2.0 * qperpx * qperpy * s012 * theta
                    - 2.0 * qperpw * qperpz * s012 * theta
                    + 2.0 * qperpw * qperpy * s022 * theta
                    + 2.0 * qperpx * qperpz * s022 * theta
                    + q0y
                        * (-(qperpx * s012) - qperpw * s022
                            + 2.0 * qperpy * (s002 - s102)
                            + qperpx * s112
                            + qperpw * s122
                            - 2.0 * q0x * s012 * theta
                            - 2.0 * q0w * s022 * theta)
                    + q0z
                        * (2.0 * qperpz * s002 + qperpw * s012
                            - qperpx * s022
                            - 2.0 * qperpz * s102
                            - qperpw * s112
                            + qperpx * s122
                            + 2.0 * q0w * s012 * theta
                            - 2.0 * q0x * s022 * theta),
            );

            c5[0] = DerivativeTerm::new(
                0.0,
                2.0 * (qperpy * qperpy * s000 + qperpz * qperpz * s000 - qperpx * qperpy * s010
                    + qperpw * qperpz * s010
                    - qperpw * qperpy * s020
                    - qperpx * qperpz * s020
                    - qperpy * qperpy * s100
                    - qperpz * qperpz * s100
                    + q0y * q0y * (-s000 + s100)
                    + q0z * q0z * (-s000 + s100)
                    + qperpx * qperpy * s110
                    - qperpw * qperpz * s110
                    + q0y * (q0x * (s010 - s110) + q0w * (s020 - s120))
                    + qperpw * qperpy * s120
                    + qperpx * qperpz * s120
                    + q0z * (-(q0w * s010) + q0x * s020 + q0w * s110 - q0x * s120))
                    * theta,
                2.0 * (qperpy * qperpy * s001 + qperpz * qperpz * s001 - qperpx * qperpy * s011
                    + qperpw * qperpz * s011
                    - qperpw * qperpy * s021
                    - qperpx * qperpz * s021
                    - qperpy * qperpy * s101
                    - qperpz * qperpz * s101
                    + q0y * q0y * (-s001 + s101)
                    + q0z * q0z * (-s001 + s101)
                    + qperpx * qperpy * s111
                    - qperpw * qperpz * s111
                    + q0y * (q0x * (s011 - s111) + q0w * (s021 - s121))
                    + qperpw * qperpy * s121
                    + qperpx * qperpz * s121
                    + q0z * (-(q0w * s011) + q0x * s021 + q0w * s111 - q0x * s121))
                    * theta,
                2.0 * (qperpy * qperpy * s002 + qperpz * qperpz * s002 - qperpx * qperpy * s012
                    + qperpw * qperpz * s012
                    - qperpw * qperpy * s022
                    - qperpx * qperpz * s022
                    - qperpy * qperpy * s102
                    - qperpz * qperpz * s102
                    + q0y * q0y * (-s002 + s102)
                    + q0z * q0z * (-s002 + s102)
                    + qperpx * qperpy * s112
                    - qperpw * qperpz * s112
                    + q0y * (q0x * (s012 - s112) + q0w * (s022 - s122))
                    + qperpw * qperpy * s122
                    + qperpx * qperpz * s122
                    + q0z * (-(q0w * s012) + q0x * s022 + q0w * s112 - q0x * s122))
                    * theta,
            );

            c1[1] = DerivativeTerm::new(
                -t0y + t1y,
                -(qperpx * qperpy * s000) - qperpw * qperpz * s000 - s010
                    + q0z * q0z * s010
                    + qperpx * qperpx * s010
                    + qperpz * qperpz * s010
                    - q0y * q0z * s020
                    + qperpw * qperpx * s020
                    - qperpy * qperpz * s020
                    + qperpx * qperpy * s100
                    + qperpw * qperpz * s100
                    + q0w * q0z * (-s000 + s100)
                    + q0x * q0x * (s010 - s110)
                    + s110
                    - q0z * q0z * s110
                    - qperpx * qperpx * s110
                    - qperpz * qperpz * s110
                    + q0x * (q0y * (-s000 + s100) + q0w * (s020 - s120))
                    + q0y * q0z * s120
                    - qperpw * qperpx * s120
                    + qperpy * qperpz * s120,
                -(qperpx * qperpy * s001) - qperpw * qperpz * s001 - s011
                    + q0z * q0z * s011
                    + qperpx * qperpx * s011
                    + qperpz * qperpz * s011
                    - q0y * q0z * s021
                    + qperpw * qperpx * s021
                    - qperpy * qperpz * s021
                    + qperpx * qperpy * s101
                    + qperpw * qperpz * s101
                    + q0w * q0z * (-s001 + s101)
                    + q0x * q0x * (s011 - s111)
                    + s111
                    - q0z * q0z * s111
                    - qperpx * qperpx * s111
                    - qperpz * qperpz * s111
                    + q0x * (q0y * (-s001 + s101) + q0w * (s021 - s121))
                    + q0y * q0z * s121
                    - qperpw * qperpx * s121
                    + qperpy * qperpz * s121,
                -(qperpx * qperpy * s002) - qperpw * qperpz * s002 - s012
                    + q0z * q0z * s012
                    + qperpx * qperpx * s012
                    + qperpz * qperpz * s012
                    - q0y * q0z * s022
                    + qperpw * qperpx * s022
                    - qperpy * qperpz * s022
                    + qperpx * qperpy * s102
                    + qperpw * qperpz * s102
                    + q0w * q0z * (-s002 + s102)
                    + q0x * q0x * (s012 - s112)
                    + s112
                    - q0z * q0z * s112
                    - qperpx * qperpx * s112
                    - qperpz * qperpz * s112
                    + q0x * (q0y * (-s002 + s102) + q0w * (s022 - s122))
                    + q0y * q0z * s122
                    - qperpw * qperpx * s122
                    + qperpy * qperpz * s122,
            );

            c2[1] = DerivativeTerm::new(
                0.0,
                qperpx * qperpy * s000 + qperpw * qperpz * s000 + q0z * q0z * s010
                    - qperpx * qperpx * s010
                    - qperpz * qperpz * s010
                    - q0y * q0z * s020
                    - qperpw * qperpx * s020
                    + qperpy * qperpz * s020
                    - qperpx * qperpy * s100
                    - qperpw * qperpz * s100
                    + q0x * q0x * (s010 - s110)
                    - q0z * q0z * s110
                    + qperpx * qperpx * s110
                    + qperpz * qperpz * s110
                    + q0y * q0z * s120
                    + qperpw * qperpx * s120
                    - qperpy * qperpz * s120
                    + 2.0 * q0z * qperpw * s000 * theta
                    + 2.0 * q0y * qperpx * s000 * theta
                    - 4.0 * q0z * qperpz * s010 * theta
                    + 2.0 * q0z * qperpy * s020 * theta
                    + 2.0 * q0y * qperpz * s020 * theta
                    + q0x
                        * (q0w * s020 + q0y * (-s000 + s100) - q0w * s120
                            + 2.0 * qperpy * s000 * theta
                            - 4.0 * qperpx * s010 * theta
                            - 2.0 * qperpw * s020 * theta)
                    + q0w
                        * (-(q0z * s000) + q0z * s100 + 2.0 * qperpz * s000 * theta
                            - 2.0 * qperpx * s020 * theta),
                qperpx * qperpy * s001 + qperpw * qperpz * s001 + q0z * q0z * s011
                    - qperpx * qperpx * s011
                    - qperpz * qperpz * s011
                    - q0y * q0z * s021
                    - qperpw * qperpx * s021
                    + qperpy * qperpz * s021
                    - qperpx * qperpy * s101
                    - qperpw * qperpz * s101
                    + q0x * q0x * (s011 - s111)
                    - q0z * q0z * s111
                    + qperpx * qperpx * s111
                    + qperpz * qperpz * s111
                    + q0y * q0z * s121
                    + qperpw * qperpx * s121
                    - qperpy * qperpz * s121
                    + 2.0 * q0z * qperpw * s001 * theta
                    + 2.0 * q0y * qperpx * s001 * theta
                    - 4.0 * q0z * qperpz * s011 * theta
                    + 2.0 * q0z * qperpy * s021 * theta
                    + 2.0 * q0y * qperpz * s021 * theta
                    + q0x
                        * (q0w * s021 + q0y * (-s001 + s101) - q0w * s121
                            + 2.0 * qperpy * s001 * theta
                            - 4.0 * qperpx * s011 * theta
                            - 2.0 * qperpw * s021 * theta)
                    + q0w
                        * (-(q0z * s001) + q0z * s101 + 2.0 * qperpz * s001 * theta
                            - 2.0 * qperpx * s021 * theta),
                qperpx * qperpy * s002 + qperpw * qperpz * s002 + q0z * q0z * s012
                    - qperpx * qperpx * s012
                    - qperpz * qperpz * s012
                    - q0y * q0z * s022
                    - qperpw * qperpx * s022
                    + qperpy * qperpz * s022
                    - qperpx * qperpy * s102
                    - qperpw * qperpz * s102
                    + q0x * q0x * (s012 - s112)
                    - q0z * q0z * s112
                    + qperpx * qperpx * s112
                    + qperpz * qperpz * s112
                    + q0y * q0z * s122
                    + qperpw * qperpx * s122
                    - qperpy * qperpz * s122
                    + 2.0 * q0z * qperpw * s002 * theta
                    + 2.0 * q0y * qperpx * s002 * theta
                    - 4.0 * q0z * qperpz * s012 * theta
                    + 2.0 * q0z * qperpy * s022 * theta
                    + 2.0 * q0y * qperpz * s022 * theta
                    + q0x
                        * (q0w * s022 + q0y * (-s002 + s102) - q0w * s122
                            + 2.0 * qperpy * s002 * theta
                            - 4.0 * qperpx * s012 * theta
                            - 2.0 * qperpw * s022 * theta)
                    + q0w
                        * (-(q0z * s002) + q0z * s102 + 2.0 * qperpz * s002 * theta
                            - 2.0 * qperpx * s022 * theta),
            );

            c3[1] = DerivativeTerm::new(
                0.0,
                2.0 * (-(q0x * qperpy * s000) - q0w * qperpz * s000
                    + 2.0 * q0x * qperpx * s010
                    + q0x * qperpw * s020
                    + q0w * qperpx * s020
                    + q0x * qperpy * s100
                    + q0w * qperpz * s100
                    - 2.0 * q0x * qperpx * s110
                    - q0x * qperpw * s120
                    - q0w * qperpx * s120
                    + q0z
                        * (2.0 * qperpz * s010 - qperpy * s020 + qperpw * (-s000 + s100)
                            - 2.0 * qperpz * s110
                            + qperpy * s120)
                    + q0y * (-(qperpx * s000) - qperpz * s020 + qperpx * s100 + qperpz * s120))
                    * theta,
                2.0 * (-(q0x * qperpy * s001) - q0w * qperpz * s001
                    + 2.0 * q0x * qperpx * s011
                    + q0x * qperpw * s021
                    + q0w * qperpx * s021
                    + q0x * qperpy * s101
                    + q0w * qperpz * s101
                    - 2.0 * q0x * qperpx * s111
                    - q0x * qperpw * s121
                    - q0w * qperpx * s121
                    + q0z
                        * (2.0 * qperpz * s011 - qperpy * s021 + qperpw * (-s001 + s101)
                            - 2.0 * qperpz * s111
                            + qperpy * s121)
                    + q0y * (-(qperpx * s001) - qperpz * s021 + qperpx * s101 + qperpz * s121))
                    * theta,
                2.0 * (-(q0x * qperpy * s002) - q0w * qperpz * s002
                    + 2.0 * q0x * qperpx * s012
                    + q0x * qperpw * s022
                    + q0w * qperpx * s022
                    + q0x * qperpy * s102
                    + q0w * qperpz * s102
                    - 2.0 * q0x * qperpx * s112
                    - q0x * qperpw * s122
                    - q0w * qperpx * s122
                    + q0z
                        * (2.0 * qperpz * s012 - qperpy * s022 + qperpw * (-s002 + s102)
                            - 2.0 * qperpz * s112
                            + qperpy * s122)
                    + q0y * (-(qperpx * s002) - qperpz * s022 + qperpx * s102 + qperpz * s122))
                    * theta,
            );

            c4[1] = DerivativeTerm::new(
                0.0,
                -(q0x * qperpy * s000) - q0w * qperpz * s000
                    + 2.0 * q0x * qperpx * s010
                    + q0x * qperpw * s020
                    + q0w * qperpx * s020
                    + q0x * qperpy * s100
                    + q0w * qperpz * s100
                    - 2.0 * q0x * qperpx * s110
                    - q0x * qperpw * s120
                    - q0w * qperpx * s120
                    + 2.0 * qperpx * qperpy * s000 * theta
                    + 2.0 * qperpw * qperpz * s000 * theta
                    + 2.0 * q0x * q0x * s010 * theta
                    + 2.0 * q0z * q0z * s010 * theta
                    - 2.0 * qperpx * qperpx * s010 * theta
                    - 2.0 * qperpz * qperpz * s010 * theta
                    + 2.0 * q0w * q0x * s020 * theta
                    - 2.0 * qperpw * qperpx * s020 * theta
                    + 2.0 * qperpy * qperpz * s020 * theta
                    + q0y
                        * (-(qperpx * s000) - qperpz * s020 + qperpx * s100 + qperpz * s120
                            - 2.0 * q0x * s000 * theta)
                    + q0z
                        * (2.0 * qperpz * s010 - qperpy * s020 + qperpw * (-s000 + s100)
                            - 2.0 * qperpz * s110
                            + qperpy * s120
                            - 2.0 * q0w * s000 * theta
                            - 2.0 * q0y * s020 * theta),
                -(q0x * qperpy * s001) - q0w * qperpz * s001
                    + 2.0 * q0x * qperpx * s011
                    + q0x * qperpw * s021
                    + q0w * qperpx * s021
                    + q0x * qperpy * s101
                    + q0w * qperpz * s101
                    - 2.0 * q0x * qperpx * s111
                    - q0x * qperpw * s121
                    - q0w * qperpx * s121
                    + 2.0 * qperpx * qperpy * s001 * theta
                    + 2.0 * qperpw * qperpz * s001 * theta
                    + 2.0 * q0x * q0x * s011 * theta
                    + 2.0 * q0z * q0z * s011 * theta
                    - 2.0 * qperpx * qperpx * s011 * theta
                    - 2.0 * qperpz * qperpz * s011 * theta
                    + 2.0 * q0w * q0x * s021 * theta
                    - 2.0 * qperpw * qperpx * s021 * theta
                    + 2.0 * qperpy * qperpz * s021 * theta
                    + q0y
                        * (-(qperpx * s001) - qperpz * s021 + qperpx * s101 + qperpz * s121
                            - 2.0 * q0x * s001 * theta)
                    + q0z
                        * (2.0 * qperpz * s011 - qperpy * s021 + qperpw * (-s001 + s101)
                            - 2.0 * qperpz * s111
                            + qperpy * s121
                            - 2.0 * q0w * s001 * theta
                            - 2.0 * q0y * s021 * theta),
                -(q0x * qperpy * s002) - q0w * qperpz * s002
                    + 2.0 * q0x * qperpx * s012
                    + q0x * qperpw * s022
                    + q0w * qperpx * s022
                    + q0x * qperpy * s102
                    + q0w * qperpz * s102
                    - 2.0 * q0x * qperpx * s112
                    - q0x * qperpw * s122
                    - q0w * qperpx * s122
                    + 2.0 * qperpx * qperpy * s002 * theta
                    + 2.0 * qperpw * qperpz * s002 * theta
                    + 2.0 * q0x * q0x * s012 * theta
                    + 2.0 * q0z * q0z * s012 * theta
                    - 2.0 * qperpx * qperpx * s012 * theta
                    - 2.0 * qperpz * qperpz * s012 * theta
                    + 2.0 * q0w * q0x * s022 * theta
                    - 2.0 * qperpw * qperpx * s022 * theta
                    + 2.0 * qperpy * qperpz * s022 * theta
                    + q0y
                        * (-(qperpx * s002) - qperpz * s022 + qperpx * s102 + qperpz * s122
                            - 2.0 * q0x * s002 * theta)
                    + q0z
                        * (2.0 * qperpz * s012 - qperpy * s022 + qperpw * (-s002 + s102)
                            - 2.0 * qperpz * s112
                            + qperpy * s122
                            - 2.0 * q0w * s002 * theta
                            - 2.0 * q0y * s022 * theta),
            );

            c5[1] = DerivativeTerm::new(
                0.,
                -2.0 * (qperpx * qperpy * s000 + qperpw * qperpz * s000 + q0z * q0z * s010
                    - qperpx * qperpx * s010
                    - qperpz * qperpz * s010
                    - q0y * q0z * s020
                    - qperpw * qperpx * s020
                    + qperpy * qperpz * s020
                    - qperpx * qperpy * s100
                    - qperpw * qperpz * s100
                    + q0w * q0z * (-s000 + s100)
                    + q0x * q0x * (s010 - s110)
                    - q0z * q0z * s110
                    + qperpx * qperpx * s110
                    + qperpz * qperpz * s110
                    + q0x * (q0y * (-s000 + s100) + q0w * (s020 - s120))
                    + q0y * q0z * s120
                    + qperpw * qperpx * s120
                    - qperpy * qperpz * s120)
                    * theta,
                -2.0 * (qperpx * qperpy * s001 + qperpw * qperpz * s001 + q0z * q0z * s011
                    - qperpx * qperpx * s011
                    - qperpz * qperpz * s011
                    - q0y * q0z * s021
                    - qperpw * qperpx * s021
                    + qperpy * qperpz * s021
                    - qperpx * qperpy * s101
                    - qperpw * qperpz * s101
                    + q0w * q0z * (-s001 + s101)
                    + q0x * q0x * (s011 - s111)
                    - q0z * q0z * s111
                    + qperpx * qperpx * s111
                    + qperpz * qperpz * s111
                    + q0x * (q0y * (-s001 + s101) + q0w * (s021 - s121))
                    + q0y * q0z * s121
                    + qperpw * qperpx * s121
                    - qperpy * qperpz * s121)
                    * theta,
                -2.0 * (qperpx * qperpy * s002 + qperpw * qperpz * s002 + q0z * q0z * s012
                    - qperpx * qperpx * s012
                    - qperpz * qperpz * s012
                    - q0y * q0z * s022
                    - qperpw * qperpx * s022
                    + qperpy * qperpz * s022
                    - qperpx * qperpy * s102
                    - qperpw * qperpz * s102
                    + q0w * q0z * (-s002 + s102)
                    + q0x * q0x * (s012 - s112)
                    - q0z * q0z * s112
                    + qperpx * qperpx * s112
                    + qperpz * qperpz * s112
                    + q0x * (q0y * (-s002 + s102) + q0w * (s022 - s122))
                    + q0y * q0z * s122
                    + qperpw * qperpx * s122
                    - qperpy * qperpz * s122)
                    * theta,
            );

            c1[2] = DerivativeTerm::new(
                -t0z + t1z,
                qperpw * qperpy * s000
                    - qperpx * qperpz * s000
                    - q0y * q0z * s010
                    - qperpw * qperpx * s010
                    - qperpy * qperpz * s010
                    - s020
                    + q0y * q0y * s020
                    + qperpx * qperpx * s020
                    + qperpy * qperpy * s020
                    - qperpw * qperpy * s100
                    + qperpx * qperpz * s100
                    + q0x * q0z * (-s000 + s100)
                    + q0y * q0z * s110
                    + qperpw * qperpx * s110
                    + qperpy * qperpz * s110
                    + q0w * (q0y * (s000 - s100) + q0x * (-s010 + s110))
                    + q0x * q0x * (s020 - s120)
                    + s120
                    - q0y * q0y * s120
                    - qperpx * qperpx * s120
                    - qperpy * qperpy * s120,
                qperpw * qperpy * s001
                    - qperpx * qperpz * s001
                    - q0y * q0z * s011
                    - qperpw * qperpx * s011
                    - qperpy * qperpz * s011
                    - s021
                    + q0y * q0y * s021
                    + qperpx * qperpx * s021
                    + qperpy * qperpy * s021
                    - qperpw * qperpy * s101
                    + qperpx * qperpz * s101
                    + q0x * q0z * (-s001 + s101)
                    + q0y * q0z * s111
                    + qperpw * qperpx * s111
                    + qperpy * qperpz * s111
                    + q0w * (q0y * (s001 - s101) + q0x * (-s011 + s111))
                    + q0x * q0x * (s021 - s121)
                    + s121
                    - q0y * q0y * s121
                    - qperpx * qperpx * s121
                    - qperpy * qperpy * s121,
                qperpw * qperpy * s002
                    - qperpx * qperpz * s002
                    - q0y * q0z * s012
                    - qperpw * qperpx * s012
                    - qperpy * qperpz * s012
                    - s022
                    + q0y * q0y * s022
                    + qperpx * qperpx * s022
                    + qperpy * qperpy * s022
                    - qperpw * qperpy * s102
                    + qperpx * qperpz * s102
                    + q0x * q0z * (-s002 + s102)
                    + q0y * q0z * s112
                    + qperpw * qperpx * s112
                    + qperpy * qperpz * s112
                    + q0w * (q0y * (s002 - s102) + q0x * (-s012 + s112))
                    + q0x * q0x * (s022 - s122)
                    + s122
                    - q0y * q0y * s122
                    - qperpx * qperpx * s122
                    - qperpy * qperpy * s122,
            );

            c2[2] = DerivativeTerm::new(
                0.0,
                q0w * q0y * s000 - q0x * q0z * s000 - qperpw * qperpy * s000
                    + qperpx * qperpz * s000
                    - q0w * q0x * s010
                    - q0y * q0z * s010
                    + qperpw * qperpx * s010
                    + qperpy * qperpz * s010
                    + q0x * q0x * s020
                    + q0y * q0y * s020
                    - qperpx * qperpx * s020
                    - qperpy * qperpy * s020
                    - q0w * q0y * s100
                    + q0x * q0z * s100
                    + qperpw * qperpy * s100
                    - qperpx * qperpz * s100
                    + q0w * q0x * s110
                    + q0y * q0z * s110
                    - qperpw * qperpx * s110
                    - qperpy * qperpz * s110
                    - q0x * q0x * s120
                    - q0y * q0y * s120
                    + qperpx * qperpx * s120
                    + qperpy * qperpy * s120
                    - 2.0 * q0y * qperpw * s000 * theta
                    + 2.0 * q0z * qperpx * s000 * theta
                    - 2.0 * q0w * qperpy * s000 * theta
                    + 2.0 * q0x * qperpz * s000 * theta
                    + 2.0 * q0x * qperpw * s010 * theta
                    + 2.0 * q0w * qperpx * s010 * theta
                    + 2.0 * q0z * qperpy * s010 * theta
                    + 2.0 * q0y * qperpz * s010 * theta
                    - 4.0 * q0x * qperpx * s020 * theta
                    - 4.0 * q0y * qperpy * s020 * theta,
                q0w * q0y * s001 - q0x * q0z * s001 - qperpw * qperpy * s001
                    + qperpx * qperpz * s001
                    - q0w * q0x * s011
                    - q0y * q0z * s011
                    + qperpw * qperpx * s011
                    + qperpy * qperpz * s011
                    + q0x * q0x * s021
                    + q0y * q0y * s021
                    - qperpx * qperpx * s021
                    - qperpy * qperpy * s021
                    - q0w * q0y * s101
                    + q0x * q0z * s101
                    + qperpw * qperpy * s101
                    - qperpx * qperpz * s101
                    + q0w * q0x * s111
                    + q0y * q0z * s111
                    - qperpw * qperpx * s111
                    - qperpy * qperpz * s111
                    - q0x * q0x * s121
                    - q0y * q0y * s121
                    + qperpx * qperpx * s121
                    + qperpy * qperpy * s121
                    - 2.0 * q0y * qperpw * s001 * theta
                    + 2.0 * q0z * qperpx * s001 * theta
                    - 2.0 * q0w * qperpy * s001 * theta
                    + 2.0 * q0x * qperpz * s001 * theta
                    + 2.0 * q0x * qperpw * s011 * theta
                    + 2.0 * q0w * qperpx * s011 * theta
                    + 2.0 * q0z * qperpy * s011 * theta
                    + 2.0 * q0y * qperpz * s011 * theta
                    - 4.0 * q0x * qperpx * s021 * theta
                    - 4.0 * q0y * qperpy * s021 * theta,
                q0w * q0y * s002 - q0x * q0z * s002 - qperpw * qperpy * s002
                    + qperpx * qperpz * s002
                    - q0w * q0x * s012
                    - q0y * q0z * s012
                    + qperpw * qperpx * s012
                    + qperpy * qperpz * s012
                    + q0x * q0x * s022
                    + q0y * q0y * s022
                    - qperpx * qperpx * s022
                    - qperpy * qperpy * s022
                    - q0w * q0y * s102
                    + q0x * q0z * s102
                    + qperpw * qperpy * s102
                    - qperpx * qperpz * s102
                    + q0w * q0x * s112
                    + q0y * q0z * s112
                    - qperpw * qperpx * s112
                    - qperpy * qperpz * s112
                    - q0x * q0x * s122
                    - q0y * q0y * s122
                    + qperpx * qperpx * s122
                    + qperpy * qperpy * s122
                    - 2.0 * q0y * qperpw * s002 * theta
                    + 2.0 * q0z * qperpx * s002 * theta
                    - 2.0 * q0w * qperpy * s002 * theta
                    + 2.0 * q0x * qperpz * s002 * theta
                    + 2.0 * q0x * qperpw * s012 * theta
                    + 2.0 * q0w * qperpx * s012 * theta
                    + 2.0 * q0z * qperpy * s012 * theta
                    + 2.0 * q0y * qperpz * s012 * theta
                    - 4.0 * q0x * qperpx * s022 * theta
                    - 4.0 * q0y * qperpy * s022 * theta,
            );

            c3[2] = DerivativeTerm::new(
                0.0,
                -2.0 * (-(q0w * qperpy * s000)
                    + q0x * qperpz * s000
                    + q0x * qperpw * s010
                    + q0w * qperpx * s010
                    - 2.0 * q0x * qperpx * s020
                    + q0w * qperpy * s100
                    - q0x * qperpz * s100
                    - q0x * qperpw * s110
                    - q0w * qperpx * s110
                    + q0z * (qperpx * s000 + qperpy * s010 - qperpx * s100 - qperpy * s110)
                    + 2.0 * q0x * qperpx * s120
                    + q0y
                        * (qperpz * s010 - 2.0 * qperpy * s020 + qperpw * (-s000 + s100)
                            - qperpz * s110
                            + 2.0 * qperpy * s120))
                    * theta,
                -2.0 * (-(q0w * qperpy * s001)
                    + q0x * qperpz * s001
                    + q0x * qperpw * s011
                    + q0w * qperpx * s011
                    - 2.0 * q0x * qperpx * s021
                    + q0w * qperpy * s101
                    - q0x * qperpz * s101
                    - q0x * qperpw * s111
                    - q0w * qperpx * s111
                    + q0z * (qperpx * s001 + qperpy * s011 - qperpx * s101 - qperpy * s111)
                    + 2.0 * q0x * qperpx * s121
                    + q0y
                        * (qperpz * s011 - 2.0 * qperpy * s021 + qperpw * (-s001 + s101)
                            - qperpz * s111
                            + 2.0 * qperpy * s121))
                    * theta,
                -2.0 * (-(q0w * qperpy * s002)
                    + q0x * qperpz * s002
                    + q0x * qperpw * s012
                    + q0w * qperpx * s012
                    - 2.0 * q0x * qperpx * s022
                    + q0w * qperpy * s102
                    - q0x * qperpz * s102
                    - q0x * qperpw * s112
                    - q0w * qperpx * s112
                    + q0z * (qperpx * s002 + qperpy * s012 - qperpx * s102 - qperpy * s112)
                    + 2.0 * q0x * qperpx * s122
                    + q0y
                        * (qperpz * s012 - 2.0 * qperpy * s022 + qperpw * (-s002 + s102)
                            - qperpz * s112
                            + 2.0 * qperpy * s122))
                    * theta,
            );

            c4[2] = DerivativeTerm::new(
                0.0,
                q0w * qperpy * s000
                    - q0x * qperpz * s000
                    - q0x * qperpw * s010
                    - q0w * qperpx * s010
                    + 2.0 * q0x * qperpx * s020
                    - q0w * qperpy * s100
                    + q0x * qperpz * s100
                    + q0x * qperpw * s110
                    + q0w * qperpx * s110
                    - 2.0 * q0x * qperpx * s120
                    - 2.0 * qperpw * qperpy * s000 * theta
                    + 2.0 * qperpx * qperpz * s000 * theta
                    - 2.0 * q0w * q0x * s010 * theta
                    + 2.0 * qperpw * qperpx * s010 * theta
                    + 2.0 * qperpy * qperpz * s010 * theta
                    + 2.0 * q0x * q0x * s020 * theta
                    + 2.0 * q0y * q0y * s020 * theta
                    - 2.0 * qperpx * qperpx * s020 * theta
                    - 2.0 * qperpy * qperpy * s020 * theta
                    + q0z
                        * (-(qperpx * s000) - qperpy * s010 + qperpx * s100 + qperpy * s110
                            - 2.0 * q0x * s000 * theta)
                    + q0y
                        * (-(qperpz * s010)
                            + 2.0 * qperpy * s020
                            + qperpw * (s000 - s100)
                            + qperpz * s110
                            - 2.0 * qperpy * s120
                            + 2.0 * q0w * s000 * theta
                            - 2.0 * q0z * s010 * theta),
                q0w * qperpy * s001
                    - q0x * qperpz * s001
                    - q0x * qperpw * s011
                    - q0w * qperpx * s011
                    + 2.0 * q0x * qperpx * s021
                    - q0w * qperpy * s101
                    + q0x * qperpz * s101
                    + q0x * qperpw * s111
                    + q0w * qperpx * s111
                    - 2.0 * q0x * qperpx * s121
                    - 2.0 * qperpw * qperpy * s001 * theta
                    + 2.0 * qperpx * qperpz * s001 * theta
                    - 2.0 * q0w * q0x * s011 * theta
                    + 2.0 * qperpw * qperpx * s011 * theta
                    + 2.0 * qperpy * qperpz * s011 * theta
                    + 2.0 * q0x * q0x * s021 * theta
                    + 2.0 * q0y * q0y * s021 * theta
                    - 2.0 * qperpx * qperpx * s021 * theta
                    - 2.0 * qperpy * qperpy * s021 * theta
                    + q0z
                        * (-(qperpx * s001) - qperpy * s011 + qperpx * s101 + qperpy * s111
                            - 2.0 * q0x * s001 * theta)
                    + q0y
                        * (-(qperpz * s011)
                            + 2.0 * qperpy * s021
                            + qperpw * (s001 - s101)
                            + qperpz * s111
                            - 2.0 * qperpy * s121
                            + 2.0 * q0w * s001 * theta
                            - 2.0 * q0z * s011 * theta),
                q0w * qperpy * s002
                    - q0x * qperpz * s002
                    - q0x * qperpw * s012
                    - q0w * qperpx * s012
                    + 2.0 * q0x * qperpx * s022
                    - q0w * qperpy * s102
                    + q0x * qperpz * s102
                    + q0x * qperpw * s112
                    + q0w * qperpx * s112
                    - 2.0 * q0x * qperpx * s122
                    - 2.0 * qperpw * qperpy * s002 * theta
                    + 2.0 * qperpx * qperpz * s002 * theta
                    - 2.0 * q0w * q0x * s012 * theta
                    + 2.0 * qperpw * qperpx * s012 * theta
                    + 2.0 * qperpy * qperpz * s012 * theta
                    + 2.0 * q0x * q0x * s022 * theta
                    + 2.0 * q0y * q0y * s022 * theta
                    - 2.0 * qperpx * qperpx * s022 * theta
                    - 2.0 * qperpy * qperpy * s022 * theta
                    + q0z
                        * (-(qperpx * s002) - qperpy * s012 + qperpx * s102 + qperpy * s112
                            - 2.0 * q0x * s002 * theta)
                    + q0y
                        * (-(qperpz * s012)
                            + 2.0 * qperpy * s022
                            + qperpw * (s002 - s102)
                            + qperpz * s112
                            - 2.0 * qperpy * s122
                            + 2.0 * q0w * s002 * theta
                            - 2.0 * q0z * s012 * theta),
            );

            c5[2] = DerivativeTerm::new(
                0.0,
                2.0 * (qperpw * qperpy * s000 - qperpx * qperpz * s000 + q0y * q0z * s010
                    - qperpw * qperpx * s010
                    - qperpy * qperpz * s010
                    - q0y * q0y * s020
                    + qperpx * qperpx * s020
                    + qperpy * qperpy * s020
                    + q0x * q0z * (s000 - s100)
                    - qperpw * qperpy * s100
                    + qperpx * qperpz * s100
                    + q0w * (q0y * (-s000 + s100) + q0x * (s010 - s110))
                    - q0y * q0z * s110
                    + qperpw * qperpx * s110
                    + qperpy * qperpz * s110
                    + q0y * q0y * s120
                    - qperpx * qperpx * s120
                    - qperpy * qperpy * s120
                    + q0x * q0x * (-s020 + s120))
                    * theta,
                2.0 * (qperpw * qperpy * s001 - qperpx * qperpz * s001 + q0y * q0z * s011
                    - qperpw * qperpx * s011
                    - qperpy * qperpz * s011
                    - q0y * q0y * s021
                    + qperpx * qperpx * s021
                    + qperpy * qperpy * s021
                    + q0x * q0z * (s001 - s101)
                    - qperpw * qperpy * s101
                    + qperpx * qperpz * s101
                    + q0w * (q0y * (-s001 + s101) + q0x * (s011 - s111))
                    - q0y * q0z * s111
                    + qperpw * qperpx * s111
                    + qperpy * qperpz * s111
                    + q0y * q0y * s121
                    - qperpx * qperpx * s121
                    - qperpy * qperpy * s121
                    + q0x * q0x * (-s021 + s121))
                    * theta,
                2.0 * (qperpw * qperpy * s002 - qperpx * qperpz * s002 + q0y * q0z * s012
                    - qperpw * qperpx * s012
                    - qperpy * qperpz * s012
                    - q0y * q0y * s022
                    + qperpx * qperpx * s022
                    + qperpy * qperpy * s022
                    + q0x * q0z * (s002 - s102)
                    - qperpw * qperpy * s102
                    + qperpx * qperpz * s102
                    + q0w * (q0y * (-s002 + s102) + q0x * (s012 - s112))
                    - q0y * q0z * s112
                    + qperpw * qperpx * s112
                    + qperpy * qperpz * s112
                    + q0y * q0y * s122
                    - qperpx * qperpx * s122
                    - qperpy * qperpy * s122
                    + q0x * q0x * (-s022 + s122))
                    * theta,
            );
        }

        Self {
            start_transform,
            start_time,
            end_transform,
            end_time,
            is_animated: true,
            has_rotation: false,
            translation: Some(translation),
            rotation: Some(rotation),
            scale: Some(scale),
            c1: Some(c1),
            c2: Some(c2),
            c3: Some(c3),
            c4: Some(c4),
            c5: Some(c5),
        }
    }

    pub fn decompose(m: &Mat4) -> (Vec3, Quaternion, Mat4) {
        // Extract translation from transformation matrix.
        let translation = Vec3::new(m.m[0][3], m.m[1][3], m.m[2][3]);

        // Compute new transformation matrix without translation.
        let mut transform_m = m.clone();
        for i in 0..3 {
            transform_m.m[i][3] = 0.0;
            transform_m.m[3][i] = 0.0;
        }
        transform_m.m[3][3] = 1.0;

        // Extract rotation from transformation matrix.
        let mut count = 0;
        let mut rotation_m = transform_m.clone();
        loop {
            // Compute next matrix in series.
            let mut rot_next = Mat4::default();
            let rot_it = rotation_m.transpose().inverse();

            for i in 0..4 {
                for j in 0..4 {
                    rot_next.m[i][j] = 0.5 * (rotation_m.m[i][j] + rot_it.m[i][j]);
                }
            }

            // Compute norm of difference.
            let mut norm: Float = 0.0;
            for i in 0..3 {
                let n = (rotation_m.m[i][0] - rot_next.m[i][0]).abs()
                    + (rotation_m.m[i][1] - rot_next.m[i][1]).abs()
                    + (rotation_m.m[i][2] - rot_next.m[i][2]).abs();
                norm = norm.max(n);
            }
            rotation_m = rot_next;

            count += 1;
            if count < 100 && norm > 0.0001 {
                break;
            }
        }

        // Compute scale using rotation and original matrix.
        let scale = Mat4::mul(&rotation_m.inverse(), &transform_m);

        let rotation = Quaternion::from(rotation_m);

        (translation, rotation, scale)
    }

    pub fn interpolate(&self, time: Float, t: &mut Transform) {
        // Handle boundary conditions for matrix interpolation.
        if !self.is_animated || time <= self.start_time {
            t.clone_from(&self.start_transform);
            return;
        }
        if time >= self.end_time {
            t.clone_from(&self.end_transform);
            return;
        }

        // Interpolate translation at dt.
        let translation = self.translation.as_ref().unwrap();
        let dt = (time - self.start_time) / (self.end_time - self.start_time);
        let translate = (1.0 - dt) * translation[0] + dt * translation[1];

        // Interpolate rotation at dt.
        let rotation = self.rotation.as_ref().unwrap();
        let rotate = Quaternion::slerp(dt, &rotation[0], &rotation[1]);

        // Interpolate scale at dt.
        let scale = self.scale.as_ref().unwrap();
        let mut scaling = Mat4::default();
        for i in 0..3 {
            for j in 0..3 {
                scaling.m[i][j] = lerp(dt, scale[0].m[i][j], scale[1].m[i][j]);
            }
        }

        // Compute interpolated matrix as product of interpolated components.
        let scaling_inverse = scaling.inverse();
        t.clone_from(
            &(Transform::translate(&translate)
                * Transform::from(rotate)
                * Transform::new(scaling, scaling_inverse)),
        );
    }

    pub fn motion_bounds(&self, b: &Bounds3) -> Bounds3 {
        if !self.is_animated {
            return self.start_transform.transform_bounds(b);
        }
        if !self.has_rotation {
            return self
                .start_transform
                .transform_bounds(b)
                .union(&self.end_transform.transform_bounds(b));
        }
        let mut bounds = Bounds3::default();
        for corner in 0..8 {
            bounds.clone_from(&bounds.union(&self.bound_point_motion(&b.corner(corner))));
        }
        bounds
    }

    pub fn bound_point_motion(&self, p: &Point3) -> Bounds3 {
        if !self.is_animated {
            return Bounds3::from(p.transform(&self.start_transform));
        }

        let mut bounds = Bounds3::new(
            &p.transform(&self.start_transform),
            &p.transform(&self.end_transform),
        );

        let rotation = self.rotation.as_ref().unwrap();
        let cos_theta = rotation[0].dot(&rotation[1]);
        let theta = cos_theta.clamp(-1.0, 1.0).acos();

        let c1 = self.c1.as_ref().unwrap();
        let c2 = self.c2.as_ref().unwrap();
        let c3 = self.c3.as_ref().unwrap();
        let c4 = self.c4.as_ref().unwrap();
        let c5 = self.c5.as_ref().unwrap();

        for c in 0..3 {
            // Find any motion derivative zeros for the component.
            let mut zeros = [0.0; 8];
            let mut n_zeros = 0;

            Interval::find_zeros(
                c1[c].eval(p),
                c2[c].eval(p),
                c3[c].eval(p),
                c4[c].eval(p),
                c5[c].eval(p),
                theta,
                Interval::new(0.0, 1.0),
                &mut zeros,
                &mut n_zeros,
                8,
            );

            // Expand bounding box for any motion derivative zeros found.
            for i in 0..n_zeros {
                let pz = p.animated_transform(
                    self,
                    lerp(zeros[i as usize], self.start_time, self.end_time),
                );
                bounds = bounds.union_point(&pz);
            }
        }

        bounds
    }

    pub fn has_scale(&self) -> bool {
        self.start_transform.has_scale() || self.end_transform.has_scale()
    }
}

impl DerivativeTerm {
    pub fn new(kc: Float, kx: Float, ky: Float, kz: Float) -> Self {
        Self { kc, kx, ky, kz }
    }

    pub fn eval(&self, p: &Point3) -> Float {
        self.kc + self.kx * p.x + self.ky * p.y + self.kz * p.z
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

impl Default for DerivativeTerm {
    fn default() -> Self {
        Self {
            kc: 0.0,
            kx: 0.0,
            ky: 0.0,
            kz: 0.0,
        }
    }
}

impl From<Mat4> for Transform {
    fn from(m: Mat4) -> Self {
        let m_inverse = m.inverse();
        Self { m, m_inverse }
    }
}

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

impl Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Transform::new(
            Mat4::mul(&self.m, &rhs.m),
            Mat4::mul(&rhs.m_inverse, &self.m_inverse),
        )
    }
}

impl Mul for &Transform {
    type Output = Transform;

    fn mul(self, rhs: Self) -> Self::Output {
        Transform::new(
            Mat4::mul(&self.m, &rhs.m),
            Mat4::mul(&rhs.m_inverse, &self.m_inverse),
        )
    }
}

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
