use crate::{
    base::spectrum::CoefficientSpectrum,
    geometries::point2::Point2,
    modulo,
    spectra::rgb::RGBSpectrum,
    utils::math::{is_pow_two, lanczos, round_pow_two, Float},
};

pub enum ImageWrap {
    Repeat,
    Black,
    Clamp,
}

pub struct MIPMap {
    resolution: Point2,
    max_anisotropy: Float,
    wrap_mode: ImageWrap,
    pyramid: Vec<Vec<RGBSpectrum>>,
}

#[derive(Clone)]
struct ResampleWeight {
    pub offset: i32,
    pub weights: [Float; 4],
}

impl MIPMap {
    pub fn new(
        image: Vec<RGBSpectrum>,
        resolution: Point2,
        max_anisotropy: Float,
        wrap_mode: ImageWrap,
    ) -> Self {
        let mut resolution = resolution;
        let res_x = resolution.x as i32;
        let res_y = resolution.y as i32;

        // Initialize levels of MIPMap from image.
        let num_levels = 1 + res_x.max(res_y).ilog2();
        let mut pyramid: Vec<Vec<RGBSpectrum>> = vec![vec![]; num_levels as usize];

        if !is_pow_two(res_x) || !is_pow_two(res_y) {
            // // Resample image to power-of-two resolution.
            let res_pow2_x = round_pow_two(res_x);
            let res_pow2_y = round_pow_two(res_y);

            // Resample image in s direction.
            let s_weights = Self::resample_weights(res_x as usize, res_pow2_x as usize);
            let mut resampled = vec![RGBSpectrum::default(); (res_pow2_x * res_pow2_y) as usize];

            // Apply weights to zoom in s direction.
            // TODO: Multithread.
            for t in 0..res_y {
                for s in 0..res_pow2_x {
                    let sample_i = (t * res_pow2_x + s) as usize;
                    let s = s as usize;

                    // Compute texel (s, t) in s-zoomed image.
                    resampled[sample_i] = RGBSpectrum::default();

                    for j in 0..4usize {
                        let mut offset = s_weights[s].offset + j as i32;
                        let weights = RGBSpectrum::new(s_weights[s].weights[j]);

                        if let ImageWrap::Repeat = wrap_mode {
                            offset = modulo(offset, res_x);
                        } else if let ImageWrap::Clamp = wrap_mode {
                            offset = offset.clamp(0, res_x - 1);
                        }

                        let image_i = (t * res_x + offset) as usize;
                        if offset >= 0 && offset < res_x {
                            resampled[sample_i] += weights * image[image_i];
                        }
                    }
                }
            }

            // Resample image in t direction.
            let t_weights = Self::resample_weights(res_y as usize, res_pow2_y as usize);
            for s in 0..res_pow2_x {
                for t in 0..res_pow2_y {
                    let sample_i = (t * res_pow2_x + s) as usize;
                    let t = t as usize;

                    resampled[sample_i] = RGBSpectrum::default();

                    for j in 0..4 {
                        let mut offset = t_weights[t].offset + j;
                        let weights = RGBSpectrum::new(t_weights[t].weights[j as usize]);

                        if let ImageWrap::Repeat = wrap_mode {
                            offset = modulo(offset, res_y);
                        } else if let ImageWrap::Clamp = wrap_mode {
                            offset = offset.clamp(0, res_y - 1);
                        }

                        let resample_i = (offset * res_pow2_x + s) as usize;
                        if offset >= 0 && offset < res_y {
                            let resample = resampled[resample_i];
                            resampled[sample_i] += weights * resample;
                        }
                    }
                }
            }

            resolution = Point2::new(res_pow2_x as Float, res_pow2_y as Float);

            // Initialize most detailed level of MIPMap.
            pyramid[0] = resampled
        } else {
            pyramid[0] = image
        }

        for i in 1..num_levels {
            // TODO: Initialize ith MIPMap level from i - 1 level.
        }

        Self {
            resolution,
            max_anisotropy,
            wrap_mode,
            pyramid,
        }
    }

    pub fn width(&self) -> i32 {
        self.resolution.x as i32
    }

    pub fn height(&self) -> i32 {
        self.resolution.y as i32
    }

    pub fn levels(&self) -> usize {
        self.pyramid.len()
    }

    pub fn texel(&self, level: usize, s: i32, t: i32) -> RGBSpectrum {
        debug_assert!(level < self.pyramid.len());

        let image = self.pyramid.get(level).unwrap();

        todo!()
    }

    pub fn lookup(&self, st: &Point2, width: Float) -> RGBSpectrum {
        // Compute MIPMap level for trilinear filtering.
        let level = self.levels() as Float - 1.0 + width.max(1e-8).log2();

        // Perform trilinear interpolation at appropriate MIPMap level.
        if level < 0.0 {
            self.triangle(0, st)
        } else if level >= self.levels() as Float - 1.0 {
            self.texel(self.levels() - 1, 0, 0)
        } else {
            let i_level = level.floor() as i32;
            let delta = level - (i_level as Float);
            RGBSpectrum::lerp(
                delta,
                &self.triangle(i_level, st),
                &self.triangle(i_level + 1, st),
            )
        }
    }

    fn triangle(&self, level: i32, st: &Point2) -> RGBSpectrum {
        let level = level.clamp(0, self.levels() as i32 - 1);
        todo!()
    }

    fn resample_weights(old_resolution: usize, new_resolution: usize) -> Vec<ResampleWeight> {
        debug_assert!(new_resolution >= old_resolution);

        let mut sampled_weights = vec![
            ResampleWeight {
                offset: 0,
                weights: [0.0; 4]
            };
            new_resolution
        ];

        let filter_weight = 2.0;
        for i in 0..new_resolution {
            // Compute image resampling weights for ith texel.
            let center = (i as Float + 0.5) * (old_resolution as Float) / (new_resolution as Float);
            sampled_weights[i].offset = ((center - filter_weight) + 0.5).floor() as i32;

            for j in 0..4 {
                let p = ((sampled_weights[i].offset + j) as Float) + 0.5;
                sampled_weights[i].weights[j as usize] = lanczos((p - center) / filter_weight, 2.0);
            }

            // Normalize filter weights for texel resampling.
            let inverse_weight_sum = 1.0
                / (sampled_weights[i].weights[0]
                    + sampled_weights[i].weights[1]
                    + sampled_weights[i].weights[2]
                    + sampled_weights[i].weights[3]);
            for j in 0..4 {
                sampled_weights[i].weights[j] *= inverse_weight_sum;
            }
        }

        sampled_weights
    }
}
