use crate::{
    base::{
        spectrum::{Spectrum, SpectrumType, RGB},
        texture::{Texture, TextureMapping2D},
    },
    geometries::vec2::Vec2,
    interactions::surface::SurfaceInteraction,
    CoefficientSpectrum,
};

pub struct UVTexture {
    mapping: Box<dyn TextureMapping2D>,
}

impl UVTexture {
    pub fn new(mapping: Box<dyn TextureMapping2D>) -> Self {
        Self { mapping }
    }
}

impl Texture<Spectrum> for UVTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        let mut dstdx = Vec2::default();
        let mut dstdy = Vec2::default();
        let st = self.mapping.map(si, &mut dstdx, &mut dstdy);
        let rgb: RGB = [st.x - st.x.floor(), st.y - st.y.floor(), 0.0];
        Spectrum::from_rgb(&rgb, SpectrumType::Ignore)
    }
}
