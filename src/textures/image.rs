use image::{open, GenericImageView};

use crate::{
    base::{
        mipmap::{ImageWrap, MIPMap},
        spectrum::{CoefficientSpectrum, SpectrumType, RGB},
        texture::TextureMapping2D,
    },
    geometries::point2::Point2,
    spectra::rgb::RGBSpectrum,
    utils::math::{inverse_gamma_correct, Float},
};

pub struct ImageTexture {
    mapping: Box<dyn TextureMapping2D>,
    mipmap: MIPMap,
}

impl ImageTexture {
    pub fn new(
        mapping: Box<dyn TextureMapping2D>,
        filename: &str,
        max_anisotropy: Float,
        wrap_mode: ImageWrap,
        scale: Float,
        gamma_corrected: bool,
    ) -> Self {
        let mut resolution = Point2::default();
        let mut texels = Self::read_image(filename, &mut resolution);

        // Flip image in Y. Texture coordinate space treats (0, 0)
        // at the lower left corner.
        let resolution_x = resolution.x as u32;
        let resolution_y = resolution.y as u32;
        for y in 0..(resolution_y / 2) {
            for x in 0..resolution_x {
                let o1 = y * resolution_x + x;
                let o2 = (resolution_y - 1 - y) * resolution_x + x;
                texels.swap(o1 as usize, o2 as usize);
            }
        }

        // Invert gamma correction.
        for texel in texels.iter_mut() {
            for i in 0..RGBSpectrum::NUM_SAMPLES {
                let color = if gamma_corrected {
                    inverse_gamma_correct(texel[i])
                } else {
                    texel[i]
                };
                texel[i] = scale * color;
            }
        }

        let mipmap = MIPMap::new(texels, resolution, max_anisotropy, wrap_mode);

        Self { mapping, mipmap }
    }

    fn read_image(filename: &str, resolution: &mut Point2) -> Vec<RGBSpectrum> {
        let image = open(filename).unwrap();

        let (width, height) = image.dimensions();
        resolution.x = width as Float;
        resolution.y = height as Float;

        let image = image.into_rgb32f();

        let mut texels: Vec<RGBSpectrum> = vec![RGBSpectrum::default(); (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y).0;
                let color: RGB = [pixel[0] / 255.0, pixel[1] / 255.0, pixel[2] / 255.0];
                texels[(y * width + x) as usize] =
                    RGBSpectrum::from_rgb(&color, SpectrumType::Ignore);
            }
        }

        texels
    }
}
