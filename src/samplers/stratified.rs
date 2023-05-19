use rand::prelude::*;

use crate::{
    base::{
        constants::Float,
        sampler::Sampler,
        sampling::{latin_hypercube, shuffle, stratified_sample_1d, stratified_sample_2d},
    },
    geometries::point2::{Point2F, Point2I},
};

#[derive(Debug, Clone)]
pub struct StratifiedSampler {
    samples_per_pixel: usize,

    pixel: Point2I,
    sample_index: usize,

    samples_1d_batch_sizes: Vec<usize>,
    samples_2d_batch_sizes: Vec<usize>,

    sample_batch_1d: Vec<Vec<Float>>,
    sample_batch_2d: Vec<Vec<Point2F>>,

    batch_1d_offset: usize,
    batch_2d_offset: usize,

    samples_1d: Vec<Vec<Float>>,
    samples_2d: Vec<Vec<Point2F>>,

    current_1d_dim: usize,
    current_2d_dim: usize,

    rng: StdRng,

    x_pixel_samples: usize,
    y_pixel_samples: usize,
    jitter_samples: bool,
}

pub struct StratifiedSamplerOptions {
    pub x_pixel_samples: usize,
    pub y_pixel_samples: usize,
    pub sampled_dimensions: usize,
    pub jitter_samples: bool,
}

impl StratifiedSampler {
    pub fn new(opts: StratifiedSamplerOptions) -> Self {
        let x_pixel_samples = opts.x_pixel_samples;
        let y_pixel_samples = opts.y_pixel_samples;

        let samples_per_pixel = x_pixel_samples * y_pixel_samples;
        let sampled_dimensions = opts.sampled_dimensions;

        let samples_1d: Vec<Vec<Float>> = vec![vec![0.0; samples_per_pixel]; sampled_dimensions];
        let samples_2d: Vec<Vec<Point2F>> =
            vec![vec![Point2F::default(); samples_per_pixel]; sampled_dimensions];

        let jitter_samples = opts.jitter_samples;

        Self {
            samples_per_pixel,

            pixel: Point2I::default(),
            sample_index: 0,

            samples_1d_batch_sizes: vec![],
            samples_2d_batch_sizes: vec![],

            sample_batch_1d: vec![],
            sample_batch_2d: vec![],

            batch_1d_offset: 0,
            batch_2d_offset: 0,

            samples_1d,
            samples_2d,

            current_1d_dim: 0,
            current_2d_dim: 0,

            rng: StdRng::from_entropy(),

            x_pixel_samples,
            y_pixel_samples,
            jitter_samples,
        }
    }
}

impl Sampler for StratifiedSampler {
    fn seed(&self, seed: i32) -> Box<dyn Sampler> {
        let mut sampler = self.clone();
        sampler.rng = StdRng::seed_from_u64(seed as u64);
        Box::new(sampler)
    }

    fn start_pixel(&mut self, p: &Point2I) {
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
        for i in 0..self.samples_1d_batch_sizes.len() {
            for j in 0..self.samples_per_pixel {
                let count = self.samples_1d_batch_sizes[i];
                let offset = j * count;
                stratified_sample_1d(
                    &mut self.sample_batch_1d[i][offset..(offset + count)],
                    count,
                    &mut self.rng,
                    self.jitter_samples,
                );
                shuffle(
                    &mut self.sample_batch_1d[i][offset..(offset + count)],
                    count,
                    1,
                    &mut self.rng,
                );
            }
        }
        for i in 0..self.samples_2d_batch_sizes.len() {
            for j in 0..self.samples_per_pixel {
                let count = self.samples_2d_batch_sizes[i];
                let offset = j * count;
                latin_hypercube(
                    &mut self.sample_batch_2d[i][offset..(offset + count)],
                    count,
                    2,
                    &mut self.rng,
                );
            }
        }

        self.pixel = p.clone();
        self.sample_index = 0;

        self.batch_1d_offset = 0;
        self.batch_2d_offset = 0;
    }

    fn get_1d(&mut self) -> Float {
        debug_assert!(self.sample_index < self.samples_per_pixel);
        if self.current_1d_dim < self.samples_1d.len() {
            let dim = self.current_1d_dim;
            self.current_1d_dim += 1;
            self.samples_1d[dim][self.sample_index]
        } else {
            self.rng.gen_range(0.0..1.0)
        }
    }

    fn get_2d(&mut self) -> Point2F {
        debug_assert!(self.sample_index < self.samples_per_pixel);
        if self.current_2d_dim < self.samples_2d.len() {
            let dim = self.current_2d_dim;
            self.current_2d_dim += 1;
            self.samples_2d[dim][self.sample_index]
        } else {
            Point2F::new(self.rng.gen_range(0.0..1.0), self.rng.gen_range(0.0..1.0))
        }
    }

    fn request_1d_batch(&mut self, n: usize) {
        self.samples_1d_batch_sizes.push(n);
        self.sample_batch_1d
            .push(vec![0.0; n * self.samples_per_pixel]);
    }

    fn request_2d_batch(&mut self, n: usize) {
        self.samples_2d_batch_sizes.push(n);
        self.sample_batch_2d
            .push(vec![Point2F::default(); n * self.samples_per_pixel]);
    }

    fn get_1d_batch(&mut self, n: usize) -> Vec<Float> {
        if self.batch_1d_offset == self.sample_batch_1d.len() {
            return vec![];
        }

        debug_assert_eq!(self.samples_1d_batch_sizes[self.batch_1d_offset], n);
        debug_assert!(self.sample_index < self.samples_per_pixel);

        let dim = self.batch_1d_offset;
        self.batch_1d_offset += 1;
        self.sample_batch_1d[dim][self.sample_index * n..].to_vec()
    }

    fn get_2d_batch(&mut self, n: usize) -> Vec<Point2F> {
        if self.batch_2d_offset == self.sample_batch_2d.len() {
            return vec![];
        }

        debug_assert_eq!(self.samples_2d_batch_sizes[self.batch_2d_offset], n);
        debug_assert!(self.sample_index < self.samples_per_pixel);

        let dim = self.batch_2d_offset;
        self.batch_2d_offset += 1;
        self.sample_batch_2d[dim][self.sample_index * n..].to_vec()
    }

    fn start_next_sample(&mut self) -> bool {
        self.current_1d_dim = 0;
        self.current_2d_dim = 0;

        self.batch_1d_offset = 0;
        self.batch_2d_offset = 0;

        self.sample_index += 1;
        self.sample_index < self.samples_per_pixel
    }

    fn set_sample_number(&mut self, sample_number: usize) -> bool {
        self.current_1d_dim = 0;
        self.current_2d_dim = 0;

        self.batch_1d_offset = 0;
        self.batch_2d_offset = 0;

        self.sample_index = sample_number;
        self.sample_index < self.samples_per_pixel
    }

    fn samples_per_pixel(&self) -> usize {
        self.samples_per_pixel
    }

    fn current_sample_number(&self) -> usize {
        self.sample_index
    }
}
