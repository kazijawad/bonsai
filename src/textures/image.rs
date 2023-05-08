use crate::{
    base::{
        constants::Float,
        mipmap::{ImageWrap, MIPMap},
        texture::{Texture, TextureMapping2D},
    },
    geometries::vec2::Vec2,
    interactions::surface::SurfaceInteraction,
    io::image::{inverse_gamma_correct, read_image},
    spectra::rgb::RGBSpectrum,
};

pub struct ImageTexture {
    mapping: Box<dyn TextureMapping2D>,
    mipmap: MIPMap,
}

pub struct ImageTextureOptions<'a> {
    pub path: &'a str,
    pub mapping: Box<dyn TextureMapping2D>,
    pub max_anisotropy: Float,
    pub wrap_mode: ImageWrap,
    pub is_gamma_corrected: bool,
    pub use_trilinear: bool,
}

impl ImageTexture {
    pub fn new<'a>(opts: ImageTextureOptions<'a>) -> Self {
        let (mut resolution, mut texels) = read_image(opts.path);

        // Flip image in Y. UV space has (0,0) at the lower left corner.
        for y in 0..(resolution.y as usize / 2) {
            for x in 0..(resolution.x as usize) {
                let o1 = y * resolution.x as usize + x;
                let o2 = (resolution.y as usize - 1 - y) * resolution.x as usize + x;
                texels.swap(o1, o2);
            }
        }

        if opts.is_gamma_corrected {
            for texel in texels.iter_mut() {
                texel[0] = inverse_gamma_correct(texel[0]);
                texel[1] = inverse_gamma_correct(texel[1]);
                texel[2] = inverse_gamma_correct(texel[2]);
            }
        }

        let mipmap = MIPMap::new(
            texels,
            &mut resolution,
            opts.max_anisotropy,
            opts.wrap_mode,
            opts.use_trilinear,
        );

        Self {
            mapping: opts.mapping,
            mipmap,
        }
    }
}

impl Texture<RGBSpectrum> for ImageTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> RGBSpectrum {
        let mut dstdx = Vec2::default();
        let mut dstdy = Vec2::default();
        let mut st = self.mapping.map(si, &mut dstdx, &mut dstdy);
        self.mipmap.lookup(&mut st, &mut dstdx, &mut dstdy)
    }
}
