use crate::{
    base::{
        filter::Filter,
        spectrum::{CoefficientSpectrum, Spectrum},
    },
    geometries::{bounds2::Bounds2, point2::Point2, vec2::Vec2},
    utils::{atomic::AtomicFloat, math::Float},
};

const FILTER_TABLE_WIDTH: usize = 16;

#[derive(Debug)]
struct Pixel {
    xyz: [Float; 3],
    splat_xyz: [AtomicFloat; 3],
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
    pub cropped_pixel_bounds: Bounds2,
    pixels: Vec<Pixel>,
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
        let cropped_pixel_bounds = Bounds2::new(
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
        let pixel_count = cropped_pixel_bounds.area() as usize;
        let mut pixels = vec![];
        pixels.reserve(pixel_count);
        for _ in 0..pixel_count {
            pixels.push(Pixel::default());
        }

        // Precompute filter weight table.`
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
            cropped_pixel_bounds,
            pixels,
            filter_table,
            scale,
            max_sample_luminance,
        }
    }

    fn get_sample_bounds(&self) -> Bounds2 {
        Bounds2::new(
            &(Point2::from(self.cropped_pixel_bounds.min) + Vec2::new(0.5, 0.5)
                - *self.filter.radius())
            .floor(),
            &(Point2::from(self.cropped_pixel_bounds.max) - Vec2::new(0.5, 0.5)
                + *self.filter.radius())
            .ceil(),
        )
    }

    fn get_film_tile(&self, sample_bounds: &Bounds2) -> Box<FilmTile> {
        // Bound image pixels that samples in bounds contribute to.
        let half_pixel = Vec2::new(0.5, 0.5);

        let p0 = (sample_bounds.min - half_pixel - *self.filter.radius()).ceil();
        let p1 = (sample_bounds.max - half_pixel + *self.filter.radius()).floor()
            + Point2::new(1.0, 1.0);

        let pixel_bounds = Bounds2::new(&p0, &p1).intersect(&self.cropped_pixel_bounds);

        Box::new(FilmTile::new(pixel_bounds, self))
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
        let p0 = (&sample - self.film.filter.radius())
            .ceil()
            .max(&self.pixel_bounds.min);
        let p1 = ((&sample + self.film.filter.radius()).floor() + Point2::new(1.0, 1.0))
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
            splat_xyz: [
                AtomicFloat::new(0.0),
                AtomicFloat::new(0.0),
                AtomicFloat::new(0.0),
            ],
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
