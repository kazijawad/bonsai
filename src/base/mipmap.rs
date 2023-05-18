use crate::{
    base::{constants::Float, spectrum::Spectrum},
    geometries::{
        point2::{Point2F, Point2I},
        vec2::Vec2F,
    },
    io::image::{Image, ImageWrapMode, NUM_CHANNELS},
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

    pub fn width(&self) -> usize {
        self.pyramid[0].resolution.x as usize
    }

    pub fn height(&self) -> usize {
        self.pyramid[0].resolution.y as usize
    }

    pub fn levels(&self) -> usize {
        self.pyramid.len()
    }

    pub fn level_resolution(&self, level: usize) -> Point2I {
        self.pyramid[level].resolution
    }

    pub fn filter(&self, st: &mut Point2F, dst0: &mut Vec2F, dst1: &mut Vec2F) -> RGBSpectrum {
        let width = Float::max(
            Float::max(dst0[0].abs(), dst0[1].abs()),
            Float::max(dst1[0].abs(), dst1[1].abs()),
        );
        self.trilinear_filter(st, width)
    }

    pub fn trilinear_filter(&self, st: &Point2F, width: Float) -> RGBSpectrum {
        // Compute MIPMap level for trilinear filtering.
        let level = (self.levels() - 1) as Float + width.max(1e-8).log2();

        // Perform trilinear interpolation at appropriate MIPMap level.
        if level < 0.0 {
            self.triangle(0, st)
        } else if level >= (self.levels() - 1) as Float {
            self.texel(self.levels() - 1, &Point2I::new(0, 0))
        } else {
            let ilevel = level.floor() as usize;
            RGBSpectrum::lerp(
                level - level.floor(),
                &self.triangle(ilevel, st),
                &self.triangle(ilevel + 1, st),
            )
        }
    }

    pub fn export(&self, filename: &str) {
        let images = &self.pyramid;

        let width: i32 = images.iter().map(|image| image.resolution.x).sum();
        let height: i32 = images.iter().map(|image| image.resolution.y).max().unwrap();

        let mut pixels: Vec<Float> = Vec::with_capacity((width * height * 3) as usize);
        for y in 0..height {
            for image in images.iter() {
                for x in 0..image.resolution.x {
                    let offset = image.pixel_offset(&Point2I::new(x, y));

                    for c in 0..NUM_CHANNELS {
                        if let Some(pixel) = image.pixels.get(offset + c) {
                            pixels.push(pixel.clone())
                        } else {
                            pixels.push(0.0)
                        }
                    }
                }
            }
        }

        Image::write(Point2I::new(width, height), pixels, filename);
    }

    fn triangle(&self, level: usize, st: &Point2F) -> RGBSpectrum {
        let level = level.clamp(0, self.levels() - 1);

        let image = &self.pyramid[level];
        let s = st[0] * image.resolution.x as Float - 0.5;
        let t = st[1] * image.resolution.y as Float - 0.5;

        let ds = s - s.floor();
        let dt = t - t.floor();

        let s0 = s.floor() as i32;
        let t0 = t.floor() as i32;

        (1.0 - ds) * (1.0 - dt) * self.texel(level, &Point2I::new(s0, t0))
            + (1.0 - ds) * dt * self.texel(level, &Point2I::new(s0, t0 + 1))
            + ds * (1.0 - dt) * self.texel(level, &Point2I::new(s0 + 1, t0))
            + ds * dt * self.texel(level, &Point2I::new(s0 + 1, t0 + 1))
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
