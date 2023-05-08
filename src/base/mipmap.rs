use std::mem;

use crate::{
    base::{constants::Float, spectrum::Spectrum, texture::lanczos},
    geometries::{point2::Point2, vec2::Vec2},
    spectra::rgb::RGBSpectrum,
    utils::math::modulo,
};

const WEIGHT_LUT_SIZE: usize = 128;

pub enum ImageWrap {
    Repeat,
    Black,
    Clamp,
}

pub struct MIPMap {
    resolution: Point2,
    pyramid: Vec<Image>,
    weight_lut: [Float; WEIGHT_LUT_SIZE],
    max_anisotropy: Float,
    wrap_mode: ImageWrap,
    use_trilinear: bool,
}

struct Image {
    pub data: Vec<RGBSpectrum>,
    pub width: i32,
    pub height: i32,
}

struct ResampleWeight {
    first_texel: i32,
    weight: [Float; 4],
}

impl MIPMap {
    pub fn new(
        mut image: Vec<RGBSpectrum>,
        resolution: &mut Point2,
        max_anisotropy: Float,
        wrap_mode: ImageWrap,
        use_trilinear: bool,
    ) -> Self {
        let width = resolution[0] as u32;
        let height = resolution[1] as u32;

        // Resample image to power-of-two resolution.
        if !width.is_power_of_two() || !height.is_power_of_two() {
            let width_pow2 = width.next_power_of_two() as i32;
            let height_pow2 = height.next_power_of_two() as i32;

            let width = width as i32;
            let height = height as i32;

            // Resample image in s direction.
            let s_weights = Self::resample_weights(width, width_pow2);
            let mut resampled_image = Vec::with_capacity((width_pow2 * height_pow2) as usize);

            // Apply weights to zoom in s direction.
            for t in 0..height {
                for s in 0..width_pow2 {
                    // Compute (s,t) texel in s-zoomed image.
                    let resample_weight = &s_weights[s as usize];

                    for j in 0..4usize {
                        let mut source_s = resample_weight.first_texel + j as i32;

                        if let ImageWrap::Repeat = wrap_mode {
                            source_s = modulo(source_s, width);
                        } else if let ImageWrap::Clamp = wrap_mode {
                            source_s = source_s.clamp(0, width - 1);
                        }

                        if source_s >= 0 && source_s < width {
                            resampled_image.push(
                                image[(t * width + source_s) as usize] * resample_weight.weight[j],
                            );
                        }
                    }
                }
            }

            // Resample image in t direction.
            let t_weights = Self::resample_weights(height, height_pow2);
            let temp_image = resampled_image.clone();
            for s in 0..width_pow2 {
                for t in 0..height_pow2 {
                    let image_i = (t * width_pow2 + s) as usize;
                    let resample_weight = &t_weights[s as usize];

                    for j in 0..4usize {
                        let mut offset = resample_weight.first_texel + j as i32;

                        if let ImageWrap::Repeat = wrap_mode {
                            offset = modulo(offset, height);
                        } else if let ImageWrap::Clamp = wrap_mode {
                            offset = offset.clamp(0, height - 1);
                        }

                        if offset >= 0 && offset < height {
                            resampled_image[image_i] +=
                                &temp_image[image_i] * &resample_weight.weight[j];
                        }
                    }
                }
            }

            image = resampled_image;

            resolution[0] = width_pow2 as Float;
            resolution[1] = height_pow2 as Float;
        }

        // Initialize levels of MIPMap from image.
        let num_levels = 1 + resolution[0].max(resolution[1]).log2() as usize;

        let mut mipmap = Self {
            resolution: resolution.clone(),
            pyramid: Vec::with_capacity(num_levels),
            weight_lut: [0.0 as Float; WEIGHT_LUT_SIZE],
            max_anisotropy,
            wrap_mode,
            use_trilinear,
        };

        // Initialize most detailed level of MIPMap.
        mipmap.pyramid.push(Image {
            data: image,
            width: resolution[0] as i32,
            height: resolution[1] as i32,
        });

        for i in 1..num_levels {
            // Initialize ith MIPMap level from i-1 level.
            let last_image = &mipmap.pyramid[i - 1];
            let width = (last_image.width / 2).max(1);
            let height = (last_image.height / 2).max(1);

            // Filter four texels from finer level of pyramid.
            let mut data = Vec::with_capacity((width * height) as usize);
            for t in 0..height {
                for s in 0..width {
                    data.push(
                        0.25 * (mipmap.texel(i - 1, 2 * s, 2 * t)
                            + mipmap.texel(i - 1, 2 * s + 1, 2 * t)
                            + mipmap.texel(i - 1, 2 * s, 2 * t + 1)
                            + mipmap.texel(i - 1, 2 * s + 1, 2 * t + 1)),
                    );
                }
            }

            mipmap.pyramid.push(Image {
                data,
                width,
                height,
            })
        }

        // Initialize EWA filter weights if needed.
        if !use_trilinear {
            let alpha = -2.0;
            for i in 0..WEIGHT_LUT_SIZE {
                let r2 = i as Float / (WEIGHT_LUT_SIZE - 1) as Float;
                mipmap.weight_lut[i] = (alpha * r2).exp() - alpha.exp();
            }
        }

        mipmap
    }

    pub fn width(&self) -> Float {
        self.resolution.x
    }

    pub fn height(&self) -> Float {
        self.resolution.y
    }

    pub fn levels(&self) -> usize {
        self.pyramid.len()
    }

    pub fn lookup(&self, st: &mut Point2, dst0: &mut Vec2, dst1: &mut Vec2) -> RGBSpectrum {
        if self.use_trilinear {
            let width = dst0[0]
                .abs()
                .max(dst0[1].abs())
                .max(dst1[0].abs())
                .max(dst1[1].abs());
            return self.trilinear(st, width);
        }

        // Compute ellipse minor and major axes.
        if dst0.length_squared() < dst1.length_squared() {
            mem::swap(dst0, dst1);
        }
        let major_length = dst0.length();
        let mut minor_length = dst1.length();

        // Clamp ellipse eccentricity if too large.
        if minor_length * self.max_anisotropy < major_length && minor_length > 0.0 {
            let scale = major_length / (minor_length * self.max_anisotropy);
            *dst1 *= scale;
            minor_length *= scale;
        }
        if minor_length == 0.0 {
            return self.triangle(0, st);
        }

        // Choose level of detail for EWA lookup and perform EWA filtering.
        let lod = (self.pyramid.len() as Float - 1.0 + minor_length.log2()).max(0.0);
        let floor_lod = lod.floor();
        RGBSpectrum::lerp(
            lod - floor_lod,
            &self.ewa(floor_lod as usize, st, dst0, dst1),
            &self.ewa(floor_lod as usize + 1, st, dst0, dst1),
        )
    }

    pub fn trilinear(&self, st: &Point2, width: Float) -> RGBSpectrum {
        // Compute MIPMap level for trilinear filtering.
        let level = self.levels() as Float - 1.0 + width.max(1e-8).log2();

        // Perform trilinear interpolation at appropriate MIPMap level.
        if level < 0.0 {
            self.triangle(0, st)
        } else if level >= self.levels() as Float - 1.0 {
            self.texel(self.levels() - 1, 0, 0)
        } else {
            let level_floor = level.floor() as usize;
            let delta = level - level_floor as Float;
            RGBSpectrum::lerp(
                delta,
                &self.triangle(level_floor, st),
                &self.triangle(level_floor + 1, st),
            )
        }
    }

    fn triangle(&self, level: usize, st: &Point2) -> RGBSpectrum {
        let level = level.clamp(0, self.levels() - 1);
        let image = &self.pyramid[level];

        let s = st[0] * image.width as Float - 0.5;
        let t = st[1] * image.height as Float - 0.5;

        let sf = s.floor();
        let tf = t.floor();

        let ds = s - sf;
        let dt = t - tf;

        let sf = sf as i32;
        let tf = tf as i32;

        (1.0 - ds) * (1.0 - dt) * self.texel(level, sf, tf)
            + (1.0 - ds) * dt * self.texel(level, sf, tf + 1)
            + ds * (1.0 - dt) * self.texel(level, sf + 1, tf)
            + ds * dt * self.texel(level, sf + 1, tf + 1)
    }

    fn ewa(&self, level: usize, st: &mut Point2, dst0: &mut Vec2, dst1: &mut Vec2) -> RGBSpectrum {
        if level >= self.levels() {
            return self.texel(self.levels() - 1, 0, 0);
        }

        let image = &self.pyramid[level];

        // Convert EWA coordinates to appropriate scale for level.
        st[0] = st[0] * image.width as Float - 0.5;
        st[1] = st[1] * image.height as Float - 0.5;
        dst0[0] *= image.width as Float;
        dst0[1] *= image.height as Float;
        dst1[0] *= image.width as Float;
        dst1[1] *= image.height as Float;

        // Compute ellipse coefficients to bound EWA filter region.
        let mut a = dst0[1] * dst0[1] + dst1[1] * dst1[1] + 1.0;
        let mut b = -2.0 * (dst0[0] * dst0[1] + dst1[0] * dst1[1]);
        let mut c = dst0[0] * dst0[0] + dst1[0] * dst1[0] + 1.0;
        let inv_f = 1.0 / (a * c - b * b * 0.25);
        a *= inv_f;
        b *= inv_f;
        c *= inv_f;

        // Compute the ellipse's (s,t) bounding box in texture space.
        let det = -b * b + 4.0 * a * c;
        let inv_det = 1.0 / det;
        let u_sqrt = (det * c).sqrt();
        let v_sqrt = (a * det).sqrt();
        let s0 = (st[0] - 2.0 * inv_det * u_sqrt).ceil();
        let s1 = (st[0] + 2.0 * inv_det * u_sqrt).floor();
        let t0 = (st[1] - 2.0 * inv_det * v_sqrt).ceil();
        let t1 = (st[1] + 2.0 * inv_det * v_sqrt).floor();

        // Scan over ellipse bound and compute quadratic equation.
        let mut sum = RGBSpectrum::default();
        let mut sum_weights = 0.0;
        for it in (t0 as i32)..=(t1 as i32) {
            let tt = it as Float - st[1];
            for is in (s0 as i32)..=(s1 as i32) {
                let ss = is as Float - st[0];

                // Compute squared radius and filter texel if inside ellipse.
                let r2 = a * ss * ss + b * ss * tt + c * tt * tt;

                if r2 < 1.0 {
                    let index = (r2 * WEIGHT_LUT_SIZE as Float).min(WEIGHT_LUT_SIZE as Float - 1.0)
                        as usize;
                    let weight = self.weight_lut[index];
                    sum += self.texel(level, is, it) * weight;
                    sum_weights += weight;
                }
            }
        }

        sum / sum_weights
    }

    fn texel(&self, level: usize, mut s: i32, mut t: i32) -> RGBSpectrum {
        let image = &self.pyramid[level];

        // Compute texel (s,t) accounting for boundary conditions.
        match self.wrap_mode {
            ImageWrap::Repeat => {
                s = modulo(s, image.width);
                t = modulo(t, image.height);
            }
            ImageWrap::Clamp => {
                s = s.clamp(0, image.width - 1);
                t = t.clamp(0, image.height - 1);
            }
            ImageWrap::Black => {
                let black = RGBSpectrum::default();
                if s < 0 || s >= image.width || t < 0 || t >= image.height {
                    return black;
                }
            }
        }

        image.data[(t * image.width + s) as usize]
    }

    fn resample_weights(old: i32, new: i32) -> Vec<ResampleWeight> {
        debug_assert!(new >= old);

        let mut weights: Vec<ResampleWeight> = Vec::with_capacity(new as usize);
        let filter_width = 2.0;
        for i in 0..new {
            // Compute image resampling weights for ith texel.
            let center = (i as Float + 0.5) * (old as Float) / (new as Float);

            let first_texel = ((center - filter_width) + 0.5).floor();
            let mut weight = [0.0 as Float; 4];

            for j in 0..4 {
                let p = first_texel + j as Float + 0.5;
                weight[j] = lanczos((p - center) / filter_width, 2.0);
            }

            // Normalize filter weights for texel resampling.
            let inverse_weight_sum = 1.0 / (weight[0] + weight[1] + weight[2] + weight[3]);
            for j in 0..4 {
                weight[j] *= inverse_weight_sum;
            }

            weights.push(ResampleWeight {
                first_texel: first_texel as i32,
                weight,
            })
        }

        weights
    }
}
