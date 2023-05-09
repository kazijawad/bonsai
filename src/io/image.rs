use image::{imageops::FilterType, io::Reader};

use crate::{base::constants::Float, geometries::point2::Point2I, utils::math::modulo};

const NUM_CHANNELS: usize = 3;

#[derive(Debug, Clone, Copy)]
pub enum ImageWrapMode {
    Repeat,
    Black,
    Clamp,
}

pub struct Image {
    pub resolution: Point2I,
    pixels: Vec<Float>,
}

impl Image {
    pub fn read(path: &str) -> Self {
        let image = Reader::open(path).expect("Failed to open image");
        let image = image.decode().expect("Failed to decode image");

        // Resample image to power-of-two resolution.
        let width = image.width();
        let height = image.height();
        if !width.is_power_of_two() || !height.is_power_of_two() {
            image.resize(
                width.next_power_of_two(),
                height.next_power_of_two(),
                FilterType::Lanczos3,
            );
        }

        let pixels = image.to_rgb32f().to_vec();

        Self {
            resolution: Point2I::new(width as i32, height as i32),
            pixels,
        }
    }

    pub fn generate_pyramid(image: Image) -> Vec<Image> {
        // Initialize levels of pyramid from image.
        let num_levels = 1 + image.resolution[0].max(image.resolution[1]).ilog2() as usize;
        let mut pyramid = Vec::with_capacity(num_levels);

        pyramid.push(image);
        for i in 1..num_levels {
            // Initialize ith MIPMap level from i-1 level.
            let last_image = &pyramid[i - 1];

            let width = (last_image.resolution[0] / 2).max(1);
            let height = (last_image.resolution[1] / 2).max(1);

            let resolution = Point2I::new(width, height);
            let mut pixels = vec![0.0; (width * height) as usize * NUM_CHANNELS];

            // Compute offsets from pixels to the four pixels used for downsampling.
            let mut src_deltas = [
                0,
                NUM_CHANNELS,
                NUM_CHANNELS * last_image.resolution[0] as usize,
                NUM_CHANNELS * (last_image.resolution[0] as usize + 1),
            ];
            if last_image.resolution[0] == 1 {
                src_deltas[1] = 0;
                src_deltas[3] -= NUM_CHANNELS;
            }
            if last_image.resolution[1] == 1 {
                src_deltas[2] = 0;
                src_deltas[3] -= NUM_CHANNELS * last_image.resolution[0] as usize;
            }

            // Downsample image to create next level and update pyramid.
            for y in 0..height {
                let mut src_offset =
                    Self::pixel_offset(&last_image.resolution, &Point2I::new(0, 2 * y));
                let mut next_offset = Self::pixel_offset(&resolution, &Point2I::new(0, y));

                for _ in 0..width {
                    for _ in 0..NUM_CHANNELS {
                        pixels[next_offset] = (last_image.pixels[src_offset]
                            + last_image.pixels[src_offset + src_deltas[1]]
                            + last_image.pixels[src_offset + src_deltas[2]]
                            + last_image.pixels[src_offset + src_deltas[3]])
                            * 0.25;

                        src_offset += 1;
                        next_offset += 1;
                    }

                    src_offset += NUM_CHANNELS;
                }
            }

            pyramid.push(Image { resolution, pixels });
        }

        pyramid
    }

    pub fn get_channel(&self, p: &Point2I, c: usize, wrap_mode: ImageWrapMode) -> Float {
        // Remap provided pixel coordinates before reading channel.
        let mut p = p.clone();
        if !self.remap_pixel(&mut p, wrap_mode) {
            return 0.0;
        }
        self.pixels[Self::pixel_offset(&self.resolution, &p) + c]
    }

    fn remap_pixel(&self, p: &mut Point2I, wrap_mode: ImageWrapMode) -> bool {
        for c in 0..2 {
            if p[c] >= 0 && p[c] < self.resolution[c] {
                continue;
            }

            match wrap_mode {
                ImageWrapMode::Repeat => p[c] = modulo(p[c], self.resolution[c]),
                ImageWrapMode::Clamp => p[c] = p[c].clamp(0, self.resolution[c] - 1),
                ImageWrapMode::Black => {
                    return false;
                }
            }
        }

        true
    }

    fn pixel_offset(resolution: &Point2I, p: &Point2I) -> usize {
        NUM_CHANNELS * (p.y * resolution.x + p.x) as usize
    }
}
