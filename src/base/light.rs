use crate::base::scene::Scene;

pub type LightFlags = i32;

pub const DELTA_POSITION: LightFlags = 1;
pub const DELTA_DIRECTION: LightFlags = 2;
pub const AREA: LightFlags = 4;
pub const INFINITE: LightFlags = 8;

pub trait Light: Send + Sync {
    fn preprocess(&self, scene: &Scene);

    fn get_flags(&self) -> LightFlags;
}
