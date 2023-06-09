use std::sync::Mutex;

use crate::{
    base::{
        constants::Float,
        filter::Filter,
        spectrum::{xyz_to_rgb, Spectrum},
    },
    geometries::{
        bounds2::{Bounds2F, Bounds2I},
        point2::{Point2F, Point2I},
        vec2::Vec2F,
    },
    io::image::Image,
    spectra::rgb::RGBSpectrum,
};

const FILTER_TABLE_WIDTH: usize = 16;

#[derive(Clone)]
pub struct Pixel {
    xyz: [Float; 3],
    splat_xyz: [Float; 3],
    filter_weight_sum: Float,
}

#[derive(Clone)]
pub struct FilmTilePixel {
    contribution_sum: RGBSpectrum,
    filter_weight_sum: Float,
}

pub struct Film {
    pub full_resolution: Point2F,
    pub cropped_pixel_bounds: Bounds2I,
    filter: Box<dyn Filter>,
    filename: String,
    pixels: Mutex<Vec<Pixel>>,
    filter_table: Vec<Float>,
    scale: Float,
    max_sample_luminance: Float,
}

pub struct FilmTile<'a> {
    pixel_bounds: Bounds2I,
    filter_radius: Vec2F,
    inv_filter_radius: Vec2F,
    filter_table: &'a [Float],
    pixels: Vec<FilmTilePixel>,
    max_sample_luminance: Float,
}

pub struct FilmOptions<'a> {
    pub full_resolution: Point2F,
    pub crop_window: Bounds2F,
    pub filter: Box<dyn Filter>,
    pub filename: &'a str,
    pub scale: Float,
    pub max_sample_luminance: Float,
}

impl Film {
    pub fn new(opts: FilmOptions) -> Self {
        let full_resolution = opts.full_resolution;
        let crop_window = opts.crop_window;

        // Compute film image bounds.
        let cropped_pixel_bounds = Bounds2I::new(
            &Point2I::new(
                (full_resolution.x * crop_window.min.x).ceil() as i32,
                (full_resolution.y * crop_window.min.y).ceil() as i32,
            ),
            &Point2I::new(
                (full_resolution.x * crop_window.max.x).ceil() as i32,
                (full_resolution.y * crop_window.max.y).ceil() as i32,
            ),
        );

        // Allocate film image storage.
        let pixels = Mutex::new(vec![Pixel::default(); cropped_pixel_bounds.area() as usize]);

        // Precompute filter weight table.
        let filter = opts.filter;
        let mut filter_table = Vec::with_capacity(FILTER_TABLE_WIDTH * FILTER_TABLE_WIDTH);
        for y in 0..FILTER_TABLE_WIDTH {
            for x in 0..FILTER_TABLE_WIDTH {
                let px = (x as Float + 0.5) * filter.radius().x / FILTER_TABLE_WIDTH as Float;
                let py = (y as Float + 0.5) * filter.radius().y / FILTER_TABLE_WIDTH as Float;
                filter_table.push(filter.evaluate(&Point2F::new(px, py)));
            }
        }

        let filename = String::from(opts.filename);
        let scale = opts.scale;
        let max_sample_luminance = opts.max_sample_luminance;

        Self {
            full_resolution,
            cropped_pixel_bounds,
            filter,
            filename,
            pixels,
            filter_table,
            scale,
            max_sample_luminance,
        }
    }

    pub fn sample_bounds(&self) -> Bounds2I {
        let bounds = Bounds2F::new(
            &(Point2F::from(self.cropped_pixel_bounds.min) + Vec2F::new(0.5, 0.5)
                - self.filter.radius())
            .floor(),
            &(Point2F::from(self.cropped_pixel_bounds.max) - Vec2F::new(0.5, 0.5)
                + self.filter.radius())
            .ceil(),
        );
        Bounds2I::from(bounds)
    }

    pub fn get_film_tile(&self, sample_bounds: &Bounds2I) -> FilmTile {
        let half_pixel = Vec2F::new(0.5, 0.5);
        let bounds = Bounds2F::from(*sample_bounds);

        let p0 = Point2I::from((bounds.min - half_pixel - self.filter.radius()).ceil());
        let p1 = Point2I::from((bounds.max - half_pixel + self.filter.radius()).floor());

        let tile_bounds = Bounds2I::new(&p0, &p1).intersect(&self.cropped_pixel_bounds);

        FilmTile::new(
            tile_bounds,
            self.filter.radius(),
            &self.filter_table,
            self.max_sample_luminance,
        )
    }

    pub fn merge_film_tile(&self, mut tile: FilmTile) {
        let mut pixels = self.pixels.lock().unwrap();

        let pixel_bounds = tile.pixel_bounds;
        pixel_bounds.traverse(|pixel| {
            let tile_pixel = tile.get_pixel(&pixel);
            let merge_pixel = &mut pixels[self.get_pixel_offset(&pixel)];

            let mut xyz: [Float; 3] = [0.0; 3];
            tile_pixel.contribution_sum.to_xyz(&mut xyz);

            for i in 0..3 {
                merge_pixel.xyz[i] += xyz[i];
            }
            merge_pixel.filter_weight_sum += tile_pixel.filter_weight_sum;
        });
    }

    pub fn write_image(&self, splat_scale: Float) {
        let pixels = self.pixels.lock().unwrap();

        let mut image = vec![0.0; self.cropped_pixel_bounds.area() as usize * 3];
        let mut offset = 0;
        self.cropped_pixel_bounds.traverse(|p| {
            // Convert pixel XYZ color to RGB.
            let pixel = &pixels[self.get_pixel_offset(&p)];
            let mut rgb = [0.0; 3];
            xyz_to_rgb(&pixel.xyz, &mut rgb);

            // Normalize pixel with weight sum.
            let filter_weight_sum = pixel.filter_weight_sum;
            if filter_weight_sum != 0.0 {
                let inverse_weight = 1.0 / filter_weight_sum;
                rgb[0] = (rgb[0] * inverse_weight).max(0.0);
                rgb[1] = (rgb[1] * inverse_weight).max(0.0);
                rgb[2] = (rgb[2] * inverse_weight).max(0.0);
            }

            // Add splat value at pixel.
            let mut splat_rgb = [0.0; 3];
            xyz_to_rgb(&pixel.splat_xyz, &mut splat_rgb);
            rgb[0] += splat_scale * splat_rgb[0];
            rgb[1] += splat_scale * splat_rgb[1];
            rgb[2] += splat_scale * splat_rgb[2];

            // Scale pixel value.
            rgb[0] *= self.scale;
            rgb[1] *= self.scale;
            rgb[2] *= self.scale;

            image[3 * offset] = rgb[0];
            image[3 * offset + 1] = rgb[1];
            image[3 * offset + 2] = rgb[2];

            offset += 1;
        });

        // Write image.
        let resolution = Point2I::new(
            self.cropped_pixel_bounds.max.x - self.cropped_pixel_bounds.min.x,
            self.cropped_pixel_bounds.max.y - self.cropped_pixel_bounds.min.y,
        );

        Image::write(resolution, image, &self.filename);
    }

    fn get_pixel_offset(&self, p: &Point2I) -> usize {
        debug_assert!(self.cropped_pixel_bounds.inside_exclusive(p));

        let width = self.cropped_pixel_bounds.max.x - self.cropped_pixel_bounds.min.x;
        let offset = (p.x - self.cropped_pixel_bounds.min.x)
            + (p.y - self.cropped_pixel_bounds.min.y) * width;

        offset as usize
    }
}

impl<'a> FilmTile<'a> {
    pub fn new(
        pixel_bounds: Bounds2I,
        filter_radius: Vec2F,
        filter_table: &'a [Float],
        max_sample_luminance: Float,
    ) -> Self {
        Self {
            pixel_bounds,
            filter_radius,
            inv_filter_radius: Vec2F::new(1.0 / filter_radius.x, 1.0 / filter_radius.y),
            filter_table,
            pixels: vec![FilmTilePixel::default(); pixel_bounds.area().max(0) as usize],
            max_sample_luminance,
        }
    }

    pub fn add_sample(
        &mut self,
        film_point: Point2F,
        mut radiance: RGBSpectrum,
        sample_weight: Float,
    ) {
        if radiance.y() > self.max_sample_luminance {
            radiance *= self.max_sample_luminance / radiance.y();
        }

        // Compute sample's raster bounds.
        let film_point = film_point - Vec2F::new(0.5, 0.5);

        let p0 = Point2I::from((film_point - self.filter_radius).ceil());
        let p0 = p0.max(&self.pixel_bounds.min);

        let p1 = Point2I::from((film_point + self.filter_radius).floor()) + Point2I::new(1, 1);
        let p1 = p1.min(&self.pixel_bounds.max);

        // Precompute x and y filter table offsets.
        let mut ifx = vec![0; (p1.x - p0.x) as usize];
        for x in p0.x..p1.x {
            let fx = ((x as Float - film_point.x)
                * self.inv_filter_radius.x
                * FILTER_TABLE_WIDTH as Float)
                .abs();
            ifx[(x - p0.x) as usize] = (fx.floor() as i32).min(FILTER_TABLE_WIDTH as i32 - 1);
        }

        let mut ify = vec![0; (p1.y - p0.y) as usize];
        for y in p0.y..p1.y {
            let fy = ((y as Float - film_point.y)
                * self.inv_filter_radius.y
                * FILTER_TABLE_WIDTH as Float)
                .abs();
            ify[(y - p0.y) as usize] = (fy.floor() as i32).min(FILTER_TABLE_WIDTH as i32 - 1);
        }

        // Loop over filter support and add sample to pixel arrays.
        for y in p0.y..p1.y {
            for x in p0.x..p1.x {
                // Evaluate filter value at (x,y) pixel.
                let offset =
                    ify[(y - p0.y) as usize] * FILTER_TABLE_WIDTH as i32 + ifx[(x - p0.x) as usize];
                let filter_weight = self.filter_table[offset as usize];

                // Update pixel values with filtered sample contribution.
                let pixel = self.get_pixel(&Point2I::new(x, y));
                pixel.contribution_sum += radiance * sample_weight * filter_weight;
                pixel.filter_weight_sum += filter_weight;
            }
        }
    }

    fn get_pixel(&mut self, p: &Point2I) -> &mut FilmTilePixel {
        debug_assert!(self.pixel_bounds.inside_exclusive(p));

        let width = self.pixel_bounds.max.x - self.pixel_bounds.min.x;
        let offset = (p.x - self.pixel_bounds.min.x) + (p.y - self.pixel_bounds.min.y) * width;

        self.pixels
            .get_mut(offset as usize)
            .expect("Failed to get pixel")
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
            contribution_sum: RGBSpectrum::default(),
            filter_weight_sum: 0.0,
        }
    }
}
