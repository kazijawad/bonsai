use crate::{base::texture::Texture, interactions::surface::SurfaceInteraction};

pub struct ConstantTexture<T: Copy + Send + Sync> {
    pub value: T,
}

impl<T: Copy + Send + Sync> Texture<T> for ConstantTexture<T> {
    fn evaluate(&self, _si: &SurfaceInteraction) -> T {
        self.value
    }
}
