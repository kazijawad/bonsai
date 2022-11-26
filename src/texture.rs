use crate::math::vec3::Vec3;

pub trait Texture: Sync + Send {
    fn value(&self, u: f32, v: f32, p: &Vec3) -> Vec3;
    fn clone_dyn(&self) -> Box<dyn Texture>;
}

pub struct ColorTexture {
    color: Vec3,
}

impl Clone for Box<dyn Texture> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
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

    fn clone_dyn(&self) -> Box<dyn Texture> {
        Box::new(self.clone())
    }
}

impl Clone for ColorTexture {
    fn clone(&self) -> Self {
        Self {
            color: self.color.clone(),
        }
    }
}
