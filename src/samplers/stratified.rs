use rand::prelude::*;

use crate::{
    base::sampler::{shuffle, Sampler},
    geometries::point2::Point2,
    utils::math::{Float, ONE_MINUS_EPSILON},
};

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

impl StratifiedSampler {
    pub fn new(
        x_pixel_samples: usize,
        y_pixel_samples: usize,
        jitter_samples: bool,
        num_sampled_dimensions: usize,
    ) -> Self {
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

fn stratified_sample_1d(samples: &mut [Float], num_samples: usize, rng: &mut StdRng, jitter: bool) {
    let inverse_num_samples = 1.0 / num_samples as Float;
    for i in 0..num_samples {
        let delta = if jitter { rng.gen_range(0.0..1.0) } else { 0.5 };
        samples[i] = Float::min(
            (i as Float + delta) * inverse_num_samples,
            ONE_MINUS_EPSILON,
        );
    }
}

fn stratified_sample_2d(
    samples: &mut [Point2],
    nx: usize,
    ny: usize,
    rng: &mut StdRng,
    jitter: bool,
) {
    let dx = 1.0 / nx as Float;
    let dy = 1.0 / ny as Float;
    let mut i = 0;
    for y in 0..ny {
        for x in 0..nx {
            let jx = if jitter { rng.gen_range(0.0..1.0) } else { 0.5 };
            let jy = if jitter { rng.gen_range(0.0..1.0) } else { 0.5 };
            samples[i].x = Float::min((x as Float + jx) * dx, ONE_MINUS_EPSILON);
            samples[i].y = Float::min((y as Float + jy) * dy, ONE_MINUS_EPSILON);
            i += 1;
        }
    }
}

fn latin_hypercube(samples: &mut [Point2], num_samples: usize, num_dims: usize, rng: &mut StdRng) {
    // Generate LHS samples along diagonal.
    let inverse_num_samples = 1.0 / num_samples as Float;
    for i in 0..num_samples {
        for j in 0..num_dims {
            let sj = (i as Float + rng.gen_range(0.0..1.0)) * inverse_num_samples;
            samples[num_dims * i + j].x = sj.min(ONE_MINUS_EPSILON);
        }
    }

    // Permute LHS samples in each dimension.
    for i in 0..num_dims {
        for j in 0..num_samples {
            let other = j + rng.gen_range(0..(num_samples - j));
            samples.swap(num_dims * j + i, num_dims * other + i);
        }
    }
}
