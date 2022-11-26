use crate::math::vec3::Vec3;

pub struct OrthonormalBasis {
    axis: Vec<Vec3>,
}

impl OrthonormalBasis {
    pub fn new() -> Self {
        Self {
            axis: vec![Vec3::zeros(), Vec3::zeros(), Vec3::zeros()],
        }
    }

    pub fn u(&self) -> Vec3 {
        self.axis[0]
    }

    pub fn v(&self) -> Vec3 {
        self.axis[1]
    }

    pub fn w(&self) -> Vec3 {
        self.axis[2]
    }

    pub fn local(&self, a: &Vec3) -> Vec3 {
        a.x * self.u() + a.y * self.v() + a.z * self.w()
    }

    pub fn build_from_w(&mut self, n: &Vec3) {
        self.axis[2] = Vec3::normalize(n);
        let a = if self.w().x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        self.axis[1] = Vec3::normalize(&Vec3::cross(&self.w(), &a));
        self.axis[0] = Vec3::cross(&self.w(), &self.v());
    }
}
