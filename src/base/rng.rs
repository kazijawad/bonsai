use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, Rng, SeedableRng};

use crate::base::constants::Float;

#[derive(Debug, Clone)]
pub struct RNG {
    rng: StdRng,
    continuous_dist: Uniform<Float>,
    discrete_dist: Uniform<usize>,
}

impl RNG {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
            continuous_dist: Uniform::from(0.0..1.0),
            discrete_dist: Uniform::from(0..1),
        }
    }

    pub fn seed(&mut self, x: u64) {
        self.rng = StdRng::seed_from_u64(x);
    }

    pub fn uniform_continuous(&mut self) -> Float {
        self.continuous_dist.sample(&mut self.rng)
    }

    pub fn uniform_continuous_range(&mut self, min: Float, max: Float) -> Float {
        if min != 0.0 && max != 1.0 {
            self.rng.gen_range(min..max)
        } else {
            self.continuous_dist.sample(&mut self.rng)
        }
    }

    pub fn uniform_discrete(&mut self) -> usize {
        self.discrete_dist.sample(&mut self.rng)
    }

    pub fn uniform_discrete_range(&mut self, min: usize, max: usize) -> usize {
        if min != 0 && max != 1 {
            self.rng.gen_range(min..max)
        } else {
            self.discrete_dist.sample(&mut self.rng)
        }
    }
}
