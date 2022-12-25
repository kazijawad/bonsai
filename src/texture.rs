use crate::interaction::SurfaceInteraction;

pub trait Texture<T>: Send + Sync {
    fn evaluate(&self, interaction: &mut SurfaceInteraction) -> T;
}
