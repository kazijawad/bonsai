use std::sync::Mutex;

use image::{ImageBuffer, Rgb32FImage};

use crate::{
    base::{
        filter::Filter,
        spectrum::{xyz_to_rgb, CoefficientSpectrum, Spectrum, RGB, XYZ},
    },
    geometries::{bounds2::Bounds2, point2::Point2, vec2::Vec2},
    utils::math::Float,
};

const FILTER_TABLE_WIDTH: usize = 16;

#[derive(Debug, Clone)]
pub struct Pixel {
    xyz: [Float; 3],
    splat_xyz: [Float; 3],
    filter_weight_sum: Float,
}

#[derive(Debug, Clone)]
pub struct FilmTilePixel {
    contribution_sum: Spectrum,
    filter_weight_sum: Float,
}

pub struct Film {
    pub full_resolution: Point2,
    pub diagonal: Float,
    pub filter: Box<dyn Filter>,
    pub filename: String,
    pub bounds: Bounds2,
    pixels: Mutex<Vec<Pixel>>,
    filter_table: [Float; FILTER_TABLE_WIDTH * FILTER_TABLE_WIDTH],
    scale: Float,
    max_sample_luminance: Float,
}

pub struct FilmTile<'a> {
    pixel_bounds: Bounds2,
    pixels: Vec<FilmTilePixel>,
    film: &'a Film,
}

pub struct FilmDescriptor {
    pub filename: String,
    pub x_resolution: Float,
    pub y_resolution: Float,
    pub crop_window: [Float; 4],
    pub scale: Float,
    pub diagonal: Float,
    pub max_sample_luminance: Float,
}

impl Film {
    pub fn create(descriptor: &FilmDescriptor, filter: Box<dyn Filter>) -> Self {
        Self::new(
            &Point2::new(descriptor.x_resolution, descriptor.y_resolution),
            &Bounds2::new(
                &Point2::new(descriptor.crop_window[0], descriptor.crop_window[1]),
                &Point2::new(descriptor.crop_window[2], descriptor.crop_window[3]),
            ),
            filter,
            descriptor.diagonal,
            descriptor.filename.clone(),
            descriptor.scale,
            descriptor.max_sample_luminance,
        )
    }

    pub fn new(
        resolution: &Point2,
        crop_window: &Bounds2,
        filter: Box<dyn Filter>,
        diagonal: Float,
        filename: String,
        scale: Float,
        max_sample_luminance: Float,
    ) -> Self {
        let diagonal = diagonal * 0.001;

        // Compute film image bounds.
        let bounds = Bounds2::new(
            &Point2::new(
                (resolution.x * crop_window.min.x).ceil(),
                (resolution.y * crop_window.min.y).ceil(),
            ),
            &Point2::new(
                (resolution.x * crop_window.max.x).ceil(),
                (resolution.y * crop_window.max.y).ceil(),
            ),
        );

        // Allocate film image storage.
        let pixels = vec![Pixel::default(); bounds.area() as usize];

        // Precompute filter weight table.
        let mut offset = 0;
        let mut filter_table = [0.0; FILTER_TABLE_WIDTH * FILTER_TABLE_WIDTH];
        for y in 0..FILTER_TABLE_WIDTH {
            for x in 0..FILTER_TABLE_WIDTH {
                let px = (x as Float + 0.5) * filter.radius().x / FILTER_TABLE_WIDTH as Float;
                let py = (y as Float + 0.5) * filter.radius().y / FILTER_TABLE_WIDTH as Float;
                filter_table[offset] = filter.evaluate(&Point2::new(px, py));
                offset += 1;
            }
        }

        Self {
            full_resolution: resolution.clone(),
            diagonal,
            filter,
            filename,
            bounds,
            pixels: Mutex::new(pixels),
            filter_table,
            scale,
            max_sample_luminance,
        }
    }

    pub fn sample_bounds(&self) -> Bounds2 {
        Bounds2::new(
            &(self.bounds.min + Vec2::new(0.5, 0.5) - self.filter.radius()).floor(),
            &(self.bounds.max - Vec2::new(0.5, 0.5) + self.filter.radius()).ceil(),
        )
    }

    pub fn create_film_tile(&self, bounds: &Bounds2) -> Box<FilmTile> {
        // Bound image pixels that samples in bounds contribute to.
        let half_pixel = Vec2::new(0.5, 0.5);

        let p0 = (bounds.min - half_pixel - self.filter.radius()).ceil();
        let p1 = (bounds.max - half_pixel + self.filter.radius()).floor() + Point2::new(1.0, 1.0);

        let pixel_bounds = Bounds2::new(&p0, &p1).intersect(&self.bounds);

        Box::new(FilmTile::new(pixel_bounds, self))
    }

    pub fn merge_film_tile(&self, mut tile: Box<FilmTile>) {
        let mut pixels = self.pixels.lock().unwrap();
        let bounds = tile.pixel_bounds.clone();

        bounds.traverse(|pixel: Point2| {
            let tile_pixel = tile.get_pixel(&pixel);
            let merge_pixel = pixels.get_mut(self.get_pixel_index(&pixel)).unwrap();

            let mut xyz: XYZ = [0.0; 3];
            tile_pixel.contribution_sum.to_xyz(&mut xyz);
            for i in 0..3 {
                merge_pixel.xyz[i] += xyz[i];
            }

            merge_pixel.filter_weight_sum += tile_pixel.filter_weight_sum;
        });
    }

    pub fn write_image(&self, splat_scale: Float) {
        let mut image = vec![0.0; self.bounds.area() as usize * 3];
        let mut offset = 0;

        let pixels = self.pixels.lock().unwrap();
        self.bounds.traverse(|point: Point2| {
            let pixel = pixels.get(self.get_pixel_index(&point)).unwrap();
            let mut rgb: RGB = [0.0; 3];

            // Convert pixel XYZ color to RGB.
            xyz_to_rgb(&pixel.xyz, &mut rgb);

            // Normalize pixel with weighted sum.
            let filter_weight_sum = pixel.filter_weight_sum;
            if filter_weight_sum != 0.0 {
                let inverse_weight = 1.0 / filter_weight_sum;
                rgb[0] = Float::max(0.0, rgb[0] * inverse_weight);
                rgb[1] = Float::max(0.0, rgb[1] * inverse_weight);
                rgb[2] = Float::max(0.0, rgb[2] * inverse_weight);
            }

            // Add splat value at pixel.
            let splat_xyz: XYZ = [pixel.splat_xyz[0], pixel.splat_xyz[1], pixel.splat_xyz[2]];
            let mut splat_rgb: RGB = [0.0; 3];
            xyz_to_rgb(&splat_xyz, &mut splat_rgb);
            rgb[0] += splat_scale * splat_rgb[0];
            rgb[1] += splat_scale * splat_rgb[1];
            rgb[2] += splat_scale * splat_rgb[2];

            // Scale pixel value.
            rgb[0] *= self.scale;
            rgb[1] *= self.scale;
            rgb[2] *= self.scale;

            // Copy over values to image vector.
            image[3 * offset] = rgb[0];
            image[3 * offset + 1] = rgb[1];
            image[3 * offset + 2] = rgb[2];

            offset += 1;
        });

        // Write image.
        let buf: Rgb32FImage = ImageBuffer::from_raw(
            (self.bounds.max.x - self.bounds.min.x) as u32,
            (self.bounds.max.y - self.bounds.min.y) as u32,
            image,
        )
        .unwrap();

        match buf.save(self.filename.clone()) {
            Ok(_) => return,
            Err(err) => panic!("Failed to save file: {:?}", err),
        }
    }

    fn get_pixel_index(&self, point: &Point2) -> usize {
        debug_assert!(self.bounds.inside_exclusive(point));
        let width = self.bounds.max.x - self.bounds.min.x;
        let offset = (point.x - self.bounds.min.x) + (point.y - self.bounds.min.y) * width;
        offset as usize
    }
}

impl<'a> FilmTile<'a> {
    pub fn new(pixel_bounds: Bounds2, film: &'a Film) -> Self {
        Self {
            pixel_bounds,
            pixels: vec![FilmTilePixel::default(); pixel_bounds.area().max(0.0) as usize],
            film,
        }
    }

    pub fn add_sample(&mut self, sample: &Point2, mut radiance: Spectrum, sample_weight: Float) {
        if radiance.y() > self.film.max_sample_luminance {
            radiance *= self.film.max_sample_luminance / radiance.y();
        }

        // Compute sample's raster bounds.
        let sample = sample - &Vec2::new(0.5, 0.5);
        let p0 = (sample - self.film.filter.radius())
            .ceil()
            .max(&self.pixel_bounds.min);
        let p1 = ((sample + self.film.filter.radius()).floor() + Point2::new(1.0, 1.0))
            .min(&self.pixel_bounds.max);

        // Precompute x and y filter table offsets.
        let mut ix = vec![0; p1.x as usize - p0.x as usize];
        for x in (p0.x as usize)..(p1.x as usize) {
            let v = ((x as Float - sample.x)
                * self.film.filter.inverse_radius().x
                * FILTER_TABLE_WIDTH as Float)
                .abs();
            ix[x - p0.x as usize] = (v.floor() as i32).min(FILTER_TABLE_WIDTH as i32 - 1);
        }
        let mut iy = vec![0; p1.y as usize - p0.y as usize];
        for y in (p0.y as usize)..(p1.y as usize) {
            let v = ((y as Float - sample.y)
                * self.film.filter.inverse_radius().y
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
                let filter_weight = self.film.filter_table[offset as usize];

                // Update pixel values with filtered sample contribution.
                let pixel = self.get_pixel(&Point2::new(x as Float, y as Float));
                pixel.contribution_sum += radiance * sample_weight * filter_weight;
                pixel.filter_weight_sum += filter_weight;
            }
        }
    }

    pub fn get_pixel(&mut self, point: &Point2) -> &mut FilmTilePixel {
        debug_assert!(self.pixel_bounds.inside_exclusive(point));
        let width = self.pixel_bounds.max.x - self.pixel_bounds.min.x;
        let offset =
            (point.x - self.pixel_bounds.min.x) + (point.y - self.pixel_bounds.min.y) * width;
        self.pixels.get_mut(offset as usize).unwrap()
    }
}

impl Default for FilmDescriptor {
    fn default() -> Self {
        Self {
            filename: String::from("result.exr"),
            x_resolution: 1280.0,
            y_resolution: 720.0,
            crop_window: [0.0, 0.0, 1.0, 1.0],
            scale: 1.0,
            diagonal: 35.0,
            max_sample_luminance: Float::INFINITY,
        }
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            xyz: [0.0; 3],
            splat_xyz: [0.0; 3],
            filter_weight_sum: 0.0,
        }
    }
}

impl Default for FilmTilePixel {
    fn default() -> Self {
        Self {
            contribution_sum: Spectrum::default(),
            filter_weight_sum: 0.0,
        }
    }
}
