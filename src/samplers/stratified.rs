use rand::prelude::*;

use crate::{
    base::sampler::Sampler,
    geometries::point2::Point2,
    utils::math::{Float, ONE_MINUS_EPSILON},
};

pub struct StratifiedSampler {
    samples_per_pixel: usize,

    current_pixel: Point2,
    current_pixel_sample_index: usize,

    float_batch: Vec<Vec<Float>>,
    point_batch: Vec<Vec<Point2>>,

    float_batch_offset: usize,
    point_batch_offset: usize,

    float_batch_sizes: Vec<usize>,
    point_batch_sizes: Vec<usize>,

    float_samples: Vec<Vec<Float>>,
    point_samples: Vec<Vec<Point2>>,

    current_float_dim: usize,
    current_point_dim: usize,

    x_samples: usize,
    y_samples: usize,
    jitter_samples: bool,

    rng: ThreadRng,
}

impl StratifiedSampler {
    pub fn new(
        x_samples: usize,
        y_samples: usize,
        jitter_samples: bool,
        num_dimensions: usize,
    ) -> Self {
        let samples_per_pixel = x_samples * y_samples;

        let float_samples: Vec<Vec<Float>> = vec![vec![0.0; samples_per_pixel]; num_dimensions];
        let point_samples: Vec<Vec<Point2>> =
            vec![vec![Point2::default(); samples_per_pixel]; num_dimensions];

        Self {
            samples_per_pixel,

            current_pixel: Point2::default(),
            current_pixel_sample_index: 0,

            float_batch: vec![],
            point_batch: vec![],

            float_batch_offset: 0,
            point_batch_offset: 0,

            float_batch_sizes: vec![],
            point_batch_sizes: vec![],

            float_samples,
            point_samples,

            current_float_dim: 0,
            current_point_dim: 0,

            x_samples,
            y_samples,
            jitter_samples,

            rng: rand::thread_rng(),
        }
    }

    fn stratified_sample_float(samples: &mut [Float], rng: &mut ThreadRng, jitter_samples: bool) {
        let num_samples = samples.len();
        let inverse_num_samples = 1.0 / num_samples as Float;
        for i in 0..num_samples {
            let delta = if jitter_samples {
                rng.gen_range(0.0..1.0)
            } else {
                0.5
            };
            samples[i] = Float::min(
                (i as Float + delta) * inverse_num_samples,
                ONE_MINUS_EPSILON,
            )
        }
    }

    fn stratified_sample_point(
        samples: &mut [Point2],
        nx: usize,
        ny: usize,
        rng: &mut ThreadRng,
        jitter_samples: bool,
    ) {
        let dx = 1.0 / nx as Float;
        let dy = 1.0 / ny as Float;
        let mut i = 0;

        for y in 0..ny {
            for x in 0..nx {
                let jx = if jitter_samples {
                    rng.gen_range(0.0..1.0)
                } else {
                    0.5
                };
                let jy = if jitter_samples {
                    rng.gen_range(0.0..1.0)
                } else {
                    0.5
                };
                samples[i].x = Float::min((x as Float + jx) * dx, ONE_MINUS_EPSILON);
                samples[i].y = Float::min((y as Float + jy) * dy, ONE_MINUS_EPSILON);
                i += 1;
            }
        }
    }

    fn latin_hypercube(samples: &mut [Point2], num_dims: usize, rng: &mut ThreadRng) {
        // Generate LHS samples along diagonal.
        let num_samples = samples.len();
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
                let other = j + rng.gen_range(0..usize::MAX);
                samples.swap(num_dims * j + i, num_dims * other + i);
            }
        }
    }
}

impl Sampler for StratifiedSampler {
    fn start_pixel(&mut self, pixel: &Point2) {
        // Generate single stratified samples for pixel.
        for samples in self.float_samples.iter_mut() {
            Self::stratified_sample_float(samples, &mut self.rng, self.jitter_samples);
            Self::shuffle(samples, self.samples_per_pixel, 1, &mut self.rng);
        }
        for samples in self.point_samples.iter_mut() {
            Self::stratified_sample_point(
                samples,
                self.x_samples,
                self.y_samples,
                &mut self.rng,
                self.jitter_samples,
            );
            Self::shuffle(samples, self.samples_per_pixel, 1, &mut self.rng);
        }

        // Generate arrays of stratified samples for pixel.
        for i in 0..self.float_batch_sizes.len() {
            for j in 0..self.samples_per_pixel {
                let count = self.float_batch_sizes[i];
                Self::stratified_sample_float(
                    &mut self.float_batch[i][j * count..count],
                    &mut self.rng,
                    self.jitter_samples,
                );
                Self::shuffle(
                    &mut self.float_batch[i][j * count..count],
                    count,
                    1,
                    &mut self.rng,
                );
            }
        }
        for i in 0..self.point_batch_sizes.len() {
            for j in 0..self.samples_per_pixel {
                let count = self.point_batch_sizes[i];
                Self::latin_hypercube(&mut self.point_batch[i][j * count..count], 2, &mut self.rng);
            }
        }

        self.current_pixel = pixel.clone();
        self.current_pixel_sample_index = 0;
        self.float_batch_offset = 0;
        self.point_batch_offset = 0;
    }

    fn get_float(&mut self) -> Float {
        debug_assert!(self.current_pixel_sample_index < self.samples_per_pixel);
        if self.current_float_dim < self.float_samples.len() {
            let dim = self.current_float_dim;
            self.current_float_dim += 1;
            self.float_samples[dim][self.current_pixel_sample_index]
        } else {
            self.rng.gen_range(0.0..1.0)
        }
    }

    fn get_point(&mut self) -> Point2 {
        debug_assert!(self.current_pixel_sample_index < self.samples_per_pixel);
        if self.current_point_dim < self.point_samples.len() {
            let dim = self.current_point_dim;
            self.current_point_dim += 1;
            self.point_samples[dim][self.current_pixel_sample_index]
        } else {
            Point2::new(self.rng.gen_range(0.0..1.0), self.rng.gen_range(0.0..1.0))
        }
    }

    fn request_float_batch(&mut self, n: usize) {
        debug_assert_eq!(self.round_count(n), n);

        self.float_batch_sizes.push(n);
        self.float_batch.push(vec![0.0; n * self.samples_per_pixel]);
    }

    fn request_point_batch(&mut self, n: usize) {
        debug_assert_eq!(self.round_count(n), n);

        self.point_batch_sizes.push(n);
        self.point_batch
            .push(vec![Point2::default(); n * self.samples_per_pixel]);
    }

    fn get_float_batch(&mut self, n: usize) -> Vec<Float> {
        if self.float_batch_offset == self.float_batch.len() {
            return vec![];
        }

        debug_assert_eq!(self.float_batch_sizes[self.float_batch_offset], n);
        debug_assert!(self.current_pixel_sample_index < self.samples_per_pixel);

        let dim = self.float_batch_offset;
        self.float_batch_offset += 1;
        self.float_batch[dim][self.current_pixel_sample_index * n..].to_vec()
    }

    fn get_point_batch(&mut self, n: usize) -> Vec<Point2> {
        if self.point_batch_offset == self.point_batch.len() {
            return vec![];
        }

        debug_assert_eq!(self.point_batch_sizes[self.point_batch_offset], n);
        debug_assert!(self.current_pixel_sample_index < self.samples_per_pixel);

        let dim = self.point_batch_offset;
        self.point_batch_offset += 1;
        self.point_batch[dim][self.current_pixel_sample_index * n..].to_vec()
    }

    fn start_next_sample(&mut self) -> bool {
        self.current_float_dim = 0;
        self.current_point_dim = 0;

        self.float_batch_offset = 0;
        self.point_batch_offset = 0;

        self.current_pixel_sample_index += 1;
        self.current_pixel_sample_index < self.samples_per_pixel
    }

    fn set_sample_number(&mut self, sample_number: usize) -> bool {
        self.current_float_dim = 0;
        self.current_point_dim = 0;

        self.float_batch_offset = 0;
        self.point_batch_offset = 0;

        self.current_pixel_sample_index = sample_number;
        self.current_pixel_sample_index < self.samples_per_pixel
    }
}
