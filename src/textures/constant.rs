use crate::base::{interaction::Interaction, texture::Texture};

pub struct ConstantTexture<T: Copy + Send + Sync> {
    pub value: T,
}

impl<T: Copy + Send + Sync> Texture<T> for ConstantTexture<T> {
    fn evaluate(&self, _: &Interaction) -> T {
        self.value
    }
}
