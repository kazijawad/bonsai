use crate::interactions::surface::SurfaceInteraction;

#[derive(Debug, Clone, Copy)]
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
}
