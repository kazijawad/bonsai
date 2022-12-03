use crate::geometries::{
    animated_transform::AnimatedTransform, mat4::Mat4, point3::Point3, quaternion::Quaternion,
    vec3::Vec3,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    m: Mat4,
    m_inverse: Mat4,
    animated_transform: Option<AnimatedTransform>,
    quaternion: Option<Quaternion>,
}

impl Transform {
    pub fn new(m: Mat4, m_inverse: Mat4) -> Self {
        Self {
            m,
            m_inverse,
            animated_transform: None,
            quaternion: None,
        }
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

    pub fn has_scale(&self) -> bool {
        todo!()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            m: Mat4::default(),
            m_inverse: Mat4::default(),
            animated_transform: None,
            quaternion: None,
        }
    }
}
