use crate::{
    base::{
        spectrum::{Spectrum, RGB},
        texture::{Texture, TextureMapping2D},
    },
    geometries::vec2::Vec2,
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
};

pub struct UVTexture {
    mapping: Box<dyn TextureMapping2D>,
}

impl UVTexture {
    pub fn new(mapping: Box<dyn TextureMapping2D>) -> Self {
        Self { mapping }
    }
}

impl Texture<RGBSpectrum> for UVTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> RGBSpectrum {
        let mut dstdx = Vec2::default();
        let mut dstdy = Vec2::default();
        let st = self.mapping.map(si, &mut dstdx, &mut dstdy);
        let rgb: RGB = [st.x - st.x.floor(), st.y - st.y.floor(), 0.0];
        RGBSpectrum::from_rgb(&rgb)
    }
}
