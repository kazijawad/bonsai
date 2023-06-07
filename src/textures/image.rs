use crate::{
    base::{
        interaction::Interaction,
        mipmap::MIPMap,
        texture::{Texture, TextureMapping2D},
    },
    geometries::vec2::Vec2F,
    io::image::{Image, ImageWrapMode},
    spectra::rgb::RGBSpectrum,
};

pub struct ImageTexture {
    pub mipmap: MIPMap,
    mapping: Box<dyn TextureMapping2D>,
}

pub struct ImageTextureOptions<'a> {
    pub path: &'a str,
    pub mapping: Box<dyn TextureMapping2D>,
    pub wrap_mode: ImageWrapMode,
}

impl ImageTexture {
    pub fn new<'a>(opts: ImageTextureOptions<'a>) -> Self {
        let image = Image::read(opts.path);
        let mipmap = MIPMap::new(image, opts.wrap_mode);

        Self {
            mipmap,
            mapping: opts.mapping,
        }
    }
}

impl Texture<RGBSpectrum> for ImageTexture {
    fn evaluate(&self, si: &Interaction) -> RGBSpectrum {
        let mut dstdx = Vec2F::default();
        let mut dstdy = Vec2F::default();
        let mut st = self.mapping.map(si, &mut dstdx, &mut dstdy);
        self.mipmap.filter(&mut st, &mut dstdx, &mut dstdy)
    }
}
