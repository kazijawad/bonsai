use rand::prelude::*;

use crate::{
    base::{
        constants::Float,
        sampler::Sampler,
        sampling::{latin_hypercube, shuffle, stratified_sample_1d, stratified_sample_2d},
    },
    geometries::point2::Point2,
};

#[derive(Clone)]
pub struct StratifiedSampler {
    samples_per_pixel: usize,

    current_pixel: Point2,
    current_pixel_sample_index: usize,

    samples_1d_sizes: Vec<usize>,
    samples_2d_sizes: Vec<usize>,

    offset_1d: usize,
    offset_2d: usize,

    sample_vec_1d: Vec<Vec<Float>>,
    sample_vec_2d: Vec<Vec<Point2>>,

    samples_1d: Vec<Vec<Float>>,
    samples_2d: Vec<Vec<Point2>>,

    current_1d_dim: usize,
    current_2d_dim: usize,

    x_pixel_samples: usize,
    y_pixel_samples: usize,
    jitter_samples: bool,

    rng: StdRng,
}

pub struct StratifiedSamplerOptions {
    pub x_pixel_samples: usize,
    pub y_pixel_samples: usize,
    pub dimensions: usize,
    pub jitter_samples: bool,
}

impl StratifiedSampler {
    pub fn new(opts: StratifiedSamplerOptions) -> Self {
        let x_pixel_samples = opts.x_pixel_samples;
        let y_pixel_samples = opts.y_pixel_samples;
        let num_sampled_dimensions = opts.dimensions;
        let jitter_samples = opts.jitter_samples;

        let samples_per_pixel = x_pixel_samples * y_pixel_samples;

        let samples_1d: Vec<Vec<Float>> =
            vec![vec![0.0; samples_per_pixel]; num_sampled_dimensions];
        let samples_2d: Vec<Vec<Point2>> =
            vec![vec![Point2::default(); samples_per_pixel]; num_sampled_dimensions];

        Self {
            samples_per_pixel,

            current_pixel: Point2::default(),
            current_pixel_sample_index: 0,

            samples_1d_sizes: vec![],
            samples_2d_sizes: vec![],

            offset_1d: 0,
            offset_2d: 0,

            sample_vec_1d: vec![],
            sample_vec_2d: vec![],

            samples_1d,
            samples_2d,

            current_1d_dim: 0,
            current_2d_dim: 0,

            x_pixel_samples,
            y_pixel_samples,
            jitter_samples,

            rng: StdRng::from_entropy(),
        }
    }
}

impl Sampler for StratifiedSampler {
    fn seed(&mut self, x: u64) {
        self.rng = StdRng::seed_from_u64(x);
    }

    fn start_pixel(&mut self, pixel: &Point2) {
        // Generate single stratified samples for pixel.
        for samples in self.samples_1d.iter_mut() {
            stratified_sample_1d(
                samples,
                self.samples_per_pixel,
                &mut self.rng,
                self.jitter_samples,
            );
            shuffle(samples, self.samples_per_pixel, 1, &mut self.rng);
        }
        for samples in self.samples_2d.iter_mut() {
            stratified_sample_2d(
                samples,
                self.x_pixel_samples,
                self.y_pixel_samples,
                &mut self.rng,
                self.jitter_samples,
            );
            shuffle(samples, self.samples_per_pixel, 1, &mut self.rng);
        }

        // Generate arrays of stratified samples for pixel.
        for i in 0..self.samples_1d_sizes.len() {
            for j in 0..self.samples_per_pixel {
                let count = self.samples_1d_sizes[i];
                let offset = j * count;
                stratified_sample_1d(
                    &mut self.sample_vec_1d[i][offset..(offset + count)],
                    count,
                    &mut self.rng,
                    self.jitter_samples,
                );
                shuffle(
                    &mut self.sample_vec_1d[i][offset..(offset + count)],
                    count,
                    1,
                    &mut self.rng,
                );
            }
        }
        for i in 0..self.samples_2d_sizes.len() {
            for j in 0..self.samples_per_pixel {
                let count = self.samples_2d_sizes[i];
                let offset = j * count;
                latin_hypercube(
                    &mut self.sample_vec_2d[i][offset..(offset + count)],
                    count,
                    2,
                    &mut self.rng,
                );
            }
        }

        self.current_pixel = pixel.clone();
        self.current_pixel_sample_index = 0;
        self.offset_1d = 0;
        self.offset_2d = 0;
    }

    fn get_1d(&mut self) -> Float {
        debug_assert!(self.current_pixel_sample_index < self.samples_per_pixel);
        if self.current_1d_dim < self.samples_1d.len() {
            let dim = self.current_1d_dim;
            self.current_1d_dim += 1;
            self.samples_1d[dim][self.current_pixel_sample_index]
        } else {
            self.rng.gen_range(0.0..1.0)
        }
    }

    fn get_2d(&mut self) -> Point2 {
        debug_assert!(self.current_pixel_sample_index < self.samples_per_pixel);
        if self.current_2d_dim < self.samples_2d.len() {
            let dim = self.current_2d_dim;
            self.current_2d_dim += 1;
            self.samples_2d[dim][self.current_pixel_sample_index]
        } else {
            Point2::new(self.rng.gen_range(0.0..1.0), self.rng.gen_range(0.0..1.0))
        }
    }

    fn request_1d_vec(&mut self, n: usize) {
        self.samples_1d_sizes.push(n);
        self.sample_vec_1d
            .push(vec![0.0; n * self.samples_per_pixel]);
    }

    fn request_2d_vec(&mut self, n: usize) {
        self.samples_2d_sizes.push(n);
        self.sample_vec_2d
            .push(vec![Point2::default(); n * self.samples_per_pixel]);
    }

    fn get_1d_vec(&mut self, n: usize) -> Vec<Float> {
        if self.offset_1d == self.sample_vec_1d.len() {
            return vec![];
        }

        debug_assert_eq!(self.samples_1d_sizes[self.offset_1d], n);
        debug_assert!(self.current_pixel_sample_index < self.samples_per_pixel);

        let dim = self.offset_1d;
        self.offset_1d += 1;
        self.sample_vec_1d[dim][self.current_pixel_sample_index * n..].to_vec()
    }

    fn get_2d_vec(&mut self, n: usize) -> Vec<Point2> {
        if self.offset_2d == self.sample_vec_2d.len() {
            return vec![];
        }

        debug_assert_eq!(self.samples_2d_sizes[self.offset_2d], n);
        debug_assert!(self.current_pixel_sample_index < self.samples_per_pixel);

        let dim = self.offset_2d;
        self.offset_2d += 1;
        self.sample_vec_2d[dim][self.current_pixel_sample_index * n..].to_vec()
    }

    fn start_next_sample(&mut self) -> bool {
        self.current_1d_dim = 0;
        self.current_2d_dim = 0;

        self.offset_1d = 0;
        self.offset_2d = 0;

        self.current_pixel_sample_index += 1;
        self.current_pixel_sample_index < self.samples_per_pixel
    }

    fn set_sample_number(&mut self, sample_number: usize) -> bool {
        self.current_1d_dim = 0;
        self.current_2d_dim = 0;

        self.offset_1d = 0;
        self.offset_2d = 0;

        self.current_pixel_sample_index = sample_number;
        self.current_pixel_sample_index < self.samples_per_pixel
    }

    fn samples_per_pixel(&self) -> usize {
        self.samples_per_pixel
    }

    fn current_sample_number(&self) -> usize {
        self.current_pixel_sample_index
    }
}
