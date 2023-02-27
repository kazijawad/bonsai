use crate::{
    base::texture::{Texture, TextureMapping3D},
    geometries::vec3::Vec3,
    interactions::surface::SurfaceInteraction,
    utils::{math::Float, noise::fbm},
};

pub struct FBmTexture {
    mapping: Box<dyn TextureMapping3D>,
    omega: Float,
    octaves: i32,
}

impl FBmTexture {
    pub fn new(mapping: Box<dyn TextureMapping3D>, omega: Float, octaves: i32) -> Self {
        Self {
            mapping,
            omega,
            octaves,
        }
    }
}

impl Texture<Float> for FBmTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> Float {
        let mut dpdx = Vec3::default();
        let mut dpdy = Vec3::default();
        let p = self.mapping.map(si, &mut dpdx, &mut dpdy);
        fbm(&p, &dpdx, &dpdy, self.omega, self.octaves)
    }
}
