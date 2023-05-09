use crate::{
    base::constants::Float,
    geometries::{
        point2::{Point2F, Point2I},
        vec2::Vec2F,
    },
    io::image::{Image, ImageWrapMode},
    spectra::rgb::RGBSpectrum,
};

pub struct MIPMap {
    pyramid: Vec<Image>,
    wrap_mode: ImageWrapMode,
}

impl MIPMap {
    pub fn new(image: Image, wrap_mode: ImageWrapMode) -> Self {
        let pyramid = Image::generate_pyramid(image);
        Self { pyramid, wrap_mode }
    }

    pub fn levels(&self) -> usize {
        self.pyramid.len()
    }

    pub fn level_resolution(&self, level: usize) -> Point2I {
        self.pyramid[level].resolution
    }

    pub fn filter(&self, st: &mut Point2F, dst0: &mut Vec2F, dst1: &mut Vec2F) -> RGBSpectrum {
        let width = 2.0
            * dst0[0]
                .abs()
                .max(dst0[1].abs())
                .max(dst1[0].abs())
                .max(dst1[1].abs());

        // Compute MIPMap level for width and handle very wide filter.
        let num_levels = self.levels() as Float;

        let level = num_levels - 1.0 + width.max(1e-8).log2();
        if level >= num_levels - 1.0 {
            return self.texel(self.levels() - 1, &Point2I::new(0, 0));
        }

        let level_i = (level.floor() as usize).max(0);
        let resolution = self.level_resolution(level_i);
        let st_i = Point2I::new(
            (st[0] * resolution[0] as Float - 0.5).round() as i32,
            (st[1] * resolution[1] as Float - 0.5).round() as i32,
        );
        self.texel(level_i, &st_i)
    }

    fn texel(&self, level: usize, st: &Point2I) -> RGBSpectrum {
        let image = &self.pyramid[level];
        RGBSpectrum::splat(
            image.get_channel(st, 0, self.wrap_mode),
            image.get_channel(st, 1, self.wrap_mode),
            image.get_channel(st, 2, self.wrap_mode),
        )
    }
}