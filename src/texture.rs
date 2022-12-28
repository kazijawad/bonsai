use crate::interactions::surface::SurfaceInteraction;

pub trait Texture<T>: Send + Sync {
    fn evaluate(&self, interaction: &mut SurfaceInteraction) -> T;
}
