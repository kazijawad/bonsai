use crate::{
    base::{
        constants::Float,
        rng::RNG,
        sampler::Sampler,
        sampling::{shuffle, stratified_sample_1d, stratified_sample_2d},
    },
    geometries::point2::{Point2F, Point2I},
};

#[derive(Debug, Clone)]
pub struct StratifiedSampler {
    x_pixel_samples: usize,
    y_pixel_samples: usize,

    jitter_samples: bool,

    pixel: Point2I,

    sample_index: usize,

    samples_1d: Vec<Vec<Float>>,
    samples_2d: Vec<Vec<Point2F>>,

    offset_1d: usize,
    offset_2d: usize,

    rng: RNG,
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

        Self {
            x_pixel_samples,
            y_pixel_samples,

            jitter_samples: opts.jitter_samples,

            pixel: Point2I::default(),
            sample_index: 0,

            samples_1d,
            samples_2d,

            offset_1d: 0,
            offset_2d: 0,

            rng: RNG::new(),
        }
    }
}

impl Sampler for StratifiedSampler {
    fn seed(&self, seed: u64) -> Box<dyn Sampler> {
        let mut sampler = self.clone();
        sampler.rng.seed(seed);
        Box::new(sampler)
    }

    fn start_pixel_sample(&mut self, p: &Point2I) {
        // Generate single stratified samples for pixel.
        for samples in self.samples_1d.iter_mut() {
            stratified_sample_1d(&mut self.rng, samples, self.jitter_samples);
            shuffle(&mut self.rng, samples, 1);
        }
        for samples in self.samples_2d.iter_mut() {
            stratified_sample_2d(
                &mut self.rng,
                samples,
                self.x_pixel_samples,
                self.y_pixel_samples,
                self.jitter_samples,
            );
            shuffle(&mut self.rng, samples, 1);
        }

        self.pixel = p.clone();
        self.sample_index = 0;
    }

    fn get_1d(&mut self) -> Float {
        debug_assert!(self.sample_index < self.samples_per_pixel());

        if self.offset_1d < self.samples_1d.len() {
            let dim = self.offset_1d;
            self.offset_1d += 1;
            self.samples_1d[dim][self.sample_index]
        } else {
            self.rng.uniform_continuous()
        }
    }

    fn get_2d(&mut self) -> Point2F {
        debug_assert!(self.sample_index < self.samples_per_pixel());

        if self.offset_2d < self.samples_2d.len() {
            let dim = self.offset_2d;
            self.offset_2d += 1;
            self.samples_2d[dim][self.sample_index]
        } else {
            Point2F::new(self.rng.uniform_continuous(), self.rng.uniform_continuous())
        }
    }

    fn start_next_sample(&mut self) -> bool {
        self.offset_1d = 0;
        self.offset_2d = 0;

        self.sample_index += 1;
        self.sample_index < self.samples_per_pixel()
    }

    fn current_sample_index(&self) -> usize {
        self.sample_index
    }

    fn samples_per_pixel(&self) -> usize {
        self.x_pixel_samples * self.y_pixel_samples
    }
}
