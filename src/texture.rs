use crate::math::vec3::Vec3;

pub trait Texture {
    fn value(&self, u: f32, v: f32, p: &Vec3) -> Vec3;
}

pub struct ColorTexture {
    color: Vec3,
}

impl ColorTexture {
    pub fn new(color: &Vec3) -> Self {
        Self {
            color: color.clone(),
        }
    }
}

impl Texture for ColorTexture {
    fn value(&self, _u: f32, _v: f32, _p: &Vec3) -> Vec3 {
        self.color
    }
}
