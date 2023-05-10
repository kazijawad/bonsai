use std::sync::Mutex;

use image::{ImageBuffer, Rgb32FImage};

use crate::{
    base::{
        constants::Float,
        filter::Filter,
        spectrum::{xyz_to_rgb, Spectrum, RGB, XYZ},
    },
    geometries::{
        bounds2::{Bounds2F, Bounds2I},
        point2::{Point2F, Point2I},
        vec2::Vec2F,
    },
    spectra::rgb::RGBSpectrum,
};

const FILTER_TABLE_WIDTH: usize = 16;

#[derive(Debug, Clone)]
pub struct Pixel {
    xyz: [Float; 3],
    filter_weight_sum: Float,
}

pub struct SampledPixel {
    contribution_sum: RGBSpectrum,
    filter_weight_sum: Float,
}

pub struct Film {
    pub full_resolution: Point2F,
    pub bounds: Bounds2I,
    pub filter: Box<dyn Filter>,
    pub filename: String,
    pixels: Mutex<Vec<Pixel>>,
    filter_table: [Float; FILTER_TABLE_WIDTH * FILTER_TABLE_WIDTH],
    scale: Float,
    max_sample_luminance: Float,
}

pub struct FilmOptions<'a> {
    pub resolution: Point2F,
    pub crop_window: Bounds2F,
    pub filter: Box<dyn Filter>,
    pub filename: &'a str,
    pub scale: Float,
    pub max_sample_luminance: Float,
}

impl Film {
    pub fn new(opts: FilmOptions) -> Self {
        // Compute film image bounds.
        let bounds = Bounds2I::new(
            &Point2I::new(
                (opts.resolution.x * opts.crop_window.min.x).ceil() as i32,
                (opts.resolution.y * opts.crop_window.min.y).ceil() as i32,
            ),
            &Point2I::new(
                (opts.resolution.x * opts.crop_window.max.x).ceil() as i32,
                (opts.resolution.y * opts.crop_window.max.y).ceil() as i32,
            ),
        );

        // Allocate film image storage.
        let pixels = Mutex::new(vec![Pixel::default(); bounds.area() as usize]);

        // Precompute filter weight table.
        let mut offset = 0;
        let mut filter_table = [0.0; FILTER_TABLE_WIDTH * FILTER_TABLE_WIDTH];
        for y in 0..FILTER_TABLE_WIDTH {
            for x in 0..FILTER_TABLE_WIDTH {
                let px = (x as Float + 0.5) * opts.filter.radius().x / FILTER_TABLE_WIDTH as Float;
                let py = (y as Float + 0.5) * opts.filter.radius().y / FILTER_TABLE_WIDTH as Float;
                filter_table[offset] = opts.filter.evaluate(&Point2F::new(px, py));
                offset += 1;
            }
        }

        Self {
            full_resolution: opts.resolution,
            filter: opts.filter,
            filename: String::from(opts.filename),
            bounds,
            pixels,
            filter_table,
            scale: opts.scale,
            max_sample_luminance: opts.max_sample_luminance,
        }
    }

    pub fn add_sample(
        &self,
        sampled_pixel: &mut SampledPixel,
        film_point: &Point2F,
        mut radiance: RGBSpectrum,
        sample_weight: Float,
    ) {
        if radiance.y() > self.max_sample_luminance {
            radiance *= self.max_sample_luminance / radiance.y();
        }

        // Compute sample's raster bounds.
        let film_point = film_point - &Vec2F::new(0.5, 0.5);
        let p0 = (film_point - self.filter.radius())
            .ceil()
            .max(&self.bounds.min.into());
        let p1 = ((film_point + self.filter.radius()).floor() + Point2F::new(1.0, 1.0))
            .min(&self.bounds.max.into());

        // Precompute x and y filter table offsets.
        let mut ix = vec![0; p1.x as usize - p0.x as usize];
        for x in (p0.x as usize)..(p1.x as usize) {
            let v = ((x as Float - film_point.x)
                * self.filter.inverse_radius().x
                * FILTER_TABLE_WIDTH as Float)
                .abs();
            ix[x - p0.x as usize] = (v.floor() as i32).min(FILTER_TABLE_WIDTH as i32 - 1);
        }
        let mut iy = vec![0; p1.y as usize - p0.y as usize];
        for y in (p0.y as usize)..(p1.y as usize) {
            let v = ((y as Float - film_point.y)
                * self.filter.inverse_radius().y
                * FILTER_TABLE_WIDTH as Float)
                .abs();
            iy[y - p0.y as usize] = (v.floor() as i32).min(FILTER_TABLE_WIDTH as i32 - 1);
        }

        // Loop over filter support and add sample to pixel arrays.
        for y in (p0.y as usize)..(p1.y as usize) {
            for x in (p0.x as usize)..(p1.x as usize) {
                // Evaluate filter value at pixel.
                let offset =
                    iy[y - p0.y as usize] * FILTER_TABLE_WIDTH as i32 + ix[x - p0.x as usize];
                let filter_weight = self.filter_table[offset as usize];

                // Update pixel values with filtered sample contribution.
                sampled_pixel.contribution_sum += radiance * sample_weight * filter_weight;
                sampled_pixel.filter_weight_sum += filter_weight;
            }
        }
    }

    pub fn merge_samples(&self, pixel: Point2I, sampled_pixel: SampledPixel) {
        let mut xyz: XYZ = [0.0; 3];
        sampled_pixel.contribution_sum.to_xyz(&mut xyz);

        let width = self.bounds.max.x - self.bounds.min.x;
        let mut pixels = self.pixels.lock().unwrap();

        let mut pixel = &mut pixels[(pixel.y * width + pixel.x) as usize];
        for i in 0..3 {
            pixel.xyz[i] += xyz[i];
        }
        pixel.filter_weight_sum += sampled_pixel.filter_weight_sum;
    }

    pub fn write_image(&self) {
        let pixels = self.pixels.lock().unwrap();
        let mut image = Vec::with_capacity(self.bounds.area() as usize * 3);

        for pixel in pixels.iter() {
            let mut rgb: RGB = [0.0; 3];

            // Convert pixel XYZ color to RGB.
            xyz_to_rgb(&pixel.xyz, &mut rgb);

            // Normalize pixel with weighted sum.
            let filter_weight_sum = pixel.filter_weight_sum;
            if filter_weight_sum != 0.0 {
                let inverse_weight = 1.0 / filter_weight_sum;
                rgb[0] = (rgb[0] * inverse_weight).max(0.0);
                rgb[1] = (rgb[1] * inverse_weight).max(0.0);
                rgb[2] = (rgb[2] * inverse_weight).max(0.0);
            }

            // Scale pixel value.
            rgb[0] *= self.scale;
            rgb[1] *= self.scale;
            rgb[2] *= self.scale;

            // Copy over values to image vector.
            image.push(rgb[0]);
            image.push(rgb[1]);
            image.push(rgb[2]);
        }

        // Write image.
        let buf: Rgb32FImage = ImageBuffer::from_raw(
            (self.bounds.max.x - self.bounds.min.x) as u32,
            (self.bounds.max.y - self.bounds.min.y) as u32,
            image,
        )
        .expect("Failed to write image buffer");

        buf.save(&self.filename).expect("Failed to save file");
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            xyz: [0.0; 3],
            filter_weight_sum: 0.0,
        }
    }
}

impl Default for SampledPixel {
    fn default() -> Self {
        Self {
            contribution_sum: RGBSpectrum::default(),
            filter_weight_sum: 0.0,
        }
    }
}
