use crate::{base::texture::Texture, interactions::surface::SurfaceInteraction};

pub struct ConstantTexture<T: Copy + Send + Sync> {
    value: T,
}

impl<T: Copy + Send + Sync> ConstantTexture<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Copy + Send + Sync> Texture<T> for ConstantTexture<T> {
    fn evaluate(&self, _si: &SurfaceInteraction) -> T {
        self.value
    }
}
