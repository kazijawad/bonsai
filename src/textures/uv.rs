use crate::{
    base::{
        spectrum::{Spectrum, RGB},
        texture::{Texture, TextureMapping2D},
    },
    geometries::vec2::Vec2F,
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
};

pub struct UVTexture {
    pub mapping: Box<dyn TextureMapping2D>,
}

impl Texture<RGBSpectrum> for UVTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> RGBSpectrum {
        let mut dstdx = Vec2F::default();
        let mut dstdy = Vec2F::default();
        let st = self.mapping.map(si, &mut dstdx, &mut dstdy);
        let rgb: RGB = [st.x - st.x.floor(), st.y - st.y.floor(), 0.0];
        RGBSpectrum::from_rgb(&rgb)
    }
}
