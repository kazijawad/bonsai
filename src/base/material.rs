use crate::{interactions::surface::SurfaceInteraction, texture::Texture, utils::math::Float};

pub enum TransportMode {
    Radiance,
    Importance,
}

pub trait Material: Send + Sync {
    fn compute_scattering_functions(
        &self,
        si: &mut SurfaceInteraction,
        mode: TransportMode,
        allow_multiple_lobes: bool,
    );

    fn bump(&self, d: &dyn Texture<Float>, si: &mut SurfaceInteraction) {
        todo!()
    }
}
