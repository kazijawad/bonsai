use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use crate::{
    base::spectrum::{
        rgb_to_xyz, xyz_to_rgb, CoefficientSpectrum, SpectrumType, CIE_LAMBDA, CIE_X, CIE_Y,
        CIE_Y_INTEGRAL, CIE_Z, NUM_CIE_SAMPLES, RGB, XYZ,
    },
    utils::math::Float,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGBSpectrum {
    components: [Float; 3],
}

impl CoefficientSpectrum for RGBSpectrum {
    const NUM_SAMPLES: usize = 3;

    fn new(v: Float) -> Self {
        Self {
            components: [v; Self::NUM_SAMPLES],
        }
    }

    fn from_sampled(lambda: &mut [Float], values: &mut [Float], n: usize) -> Self {
        // Sort samples if unordered, and use sorted for returned spectrum.
        if !Self::is_samples_sorted(lambda, n) {
            let mut lambda_segment: Vec<Float> = vec![];
            for i in (lambda[0] as usize)..(lambda[n] as usize) {
                lambda_segment.push(i as Float);
            }

            let mut values_segment: Vec<Float> = vec![];
            for i in (values[0] as usize)..(values[n] as usize) {
                values_segment.push(i as Float);
            }

            Self::sort_samples(&mut lambda_segment, &mut values_segment, n);
            return Self::from_sampled(&mut lambda_segment, &mut values_segment, n);
        }

        let mut xyz: XYZ = [0.0; 3];
        for i in 0..NUM_CIE_SAMPLES {
            let value = Self::interpolate_samples(lambda, values, n, CIE_LAMBDA[i]);
            xyz[0] += value * CIE_X[i];
            xyz[1] += value * CIE_Y[i];
            xyz[2] += value * CIE_Z[i];
        }

        let scale = (CIE_LAMBDA[NUM_CIE_SAMPLES - 1] - CIE_LAMBDA[0])
            / (CIE_Y_INTEGRAL * (NUM_CIE_SAMPLES as Float));
        xyz[0] *= scale;
        xyz[1] *= scale;
        xyz[2] *= scale;

        Self::from_xyz(&xyz, SpectrumType::Reflectance)
    }

    fn from_xyz(xyz: &XYZ, spectrum_type: SpectrumType) -> Self {
        let mut rgb = Self::default();
        xyz_to_rgb(xyz, &mut rgb.components);
        rgb
    }

    fn from_rgb(rgb: &RGB, spectrum_type: SpectrumType) -> Self {
        debug_assert!(!rgb[0].is_nan() && !rgb[1].is_nan() && !rgb[2].is_nan());
        Self {
            components: rgb.clone(),
        }
    }

    fn lerp(t: Float, a: &Self, b: &Self) -> Self {
        (*a) * (1.0 - t) + (*b) * t
    }

    fn sqrt(&self) -> Self {
        let r = self[0].sqrt();
        let g = self[1].sqrt();
        let b = self[2].sqrt();

        debug_assert!(!r.is_nan() && !g.is_nan() && !b.is_nan());

        Self {
            components: [r, g, b],
        }
    }

    fn pow(&self, e: Float) -> Self {
        let r = self[0].powf(e);
        let g = self[1].powf(e);
        let b = self[2].powf(e);

        debug_assert!(!r.is_nan() && !g.is_nan() && !b.is_nan());

        Self {
            components: [r, g, b],
        }
    }

    fn exp(&self) -> Self {
        let r = self[0].exp();
        let g = self[1].exp();
        let b = self[2].exp();

        debug_assert!(!r.is_nan() && !g.is_nan() && !b.is_nan());

        Self {
            components: [r, g, b],
        }
    }

    fn clamp(&self, min: Float, max: Float) -> Self {
        let r = self[0].clamp(min, max);
        let g = self[1].clamp(min, max);
        let b = self[2].clamp(min, max);

        debug_assert!(!r.is_nan() && !g.is_nan() && !b.is_nan());

        Self {
            components: [r, g, b],
        }
    }

    fn max_component_value(&self) -> Float {
        self[0].max(self[1]).max(self[2])
    }

    fn y(&self) -> Float {
        const W: [Float; 3] = [0.212671, 0.715160, 0.072169];
        W[0] * self[0] + W[1] * self[1] + W[2] * self[2]
    }

    fn to_xyz(&self, xyz: &mut XYZ) {
        rgb_to_xyz(&self.components, xyz)
    }

    fn to_rgb(&self, rgb: &mut RGB) {
        rgb[0] = self[0];
        rgb[1] = self[1];
        rgb[2] = self[2];
    }

    fn is_black(&self) -> bool {
        self[0] == 0.0 || self[1] == 0.0 || self[2] == 0.0
    }

    fn is_nan(&self) -> bool {
        self[0].is_nan() || self[1].is_nan() || self[2].is_nan()
    }

    fn is_initialized(&self) -> bool {
        true
    }
}

impl Default for RGBSpectrum {
    fn default() -> Self {
        Self {
            components: [0.0; Self::NUM_SAMPLES],
        }
    }
}

impl Add for RGBSpectrum {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self {
            components: [self[0] + rhs[0], self[1] + rhs[1], self[2] + rhs[2]],
        }
    }
}

impl Add for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        RGBSpectrum {
            components: [self[0] + rhs[0], self[1] + rhs[1], self[2] + rhs[2]],
        }
    }
}

impl AddAssign for RGBSpectrum {
    fn add_assign(&mut self, rhs: Self) {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        self[0] += rhs[0];
        self[1] += rhs[1];
        self[2] += rhs[2];
    }
}

impl Sub for RGBSpectrum {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self {
            components: [self[0] - rhs[0], self[1] - rhs[1], self[2] - rhs[2]],
        }
    }
}

impl Sub for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        RGBSpectrum {
            components: [self[0] - rhs[0], self[1] - rhs[1], self[2] - rhs[2]],
        }
    }
}

impl SubAssign for RGBSpectrum {
    fn sub_assign(&mut self, rhs: Self) {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        self[0] -= rhs[0];
        self[1] -= rhs[1];
        self[2] -= rhs[2];
    }
}

impl Mul for RGBSpectrum {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self {
            components: [self[0] * rhs[0], self[1] * rhs[1], self[2] * rhs[2]],
        }
    }
}

impl Mul for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn mul(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        RGBSpectrum {
            components: [self[0] * rhs[0], self[1] * rhs[1], self[2] * rhs[2]],
        }
    }
}

impl Mul<Float> for RGBSpectrum {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self {
            components: [self[0] * rhs, self[1] * rhs, self[2] * rhs],
        }
    }
}

impl MulAssign for RGBSpectrum {
    fn mul_assign(&mut self, rhs: Self) {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        self[0] *= rhs[0];
        self[1] *= rhs[1];
        self[2] *= rhs[2];
    }
}

impl MulAssign<Float> for RGBSpectrum {
    fn mul_assign(&mut self, rhs: Float) {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        self[0] *= rhs;
        self[1] *= rhs;
        self[2] *= rhs;
    }
}

impl Div for RGBSpectrum {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self {
            components: [self[0] / rhs[0], self[1] / rhs[1], self[2] / rhs[2]],
        }
    }
}

impl Div for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn div(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        RGBSpectrum {
            components: [self[0] / rhs[0], self[1] / rhs[1], self[2] / rhs[2]],
        }
    }
}

impl Div<Float> for RGBSpectrum {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan() && rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self {
            components: [self[0] * rhs, self[1] * rhs, self[2] * rhs],
        }
    }
}

impl DivAssign for RGBSpectrum {
    fn div_assign(&mut self, rhs: Self) {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        self[0] /= rhs[0];
        self[1] /= rhs[1];
        self[2] /= rhs[2];
    }
}

impl DivAssign<Float> for RGBSpectrum {
    fn div_assign(&mut self, rhs: Float) {
        debug_assert!(!self.is_nan() && !rhs.is_nan() && rhs != 0.0);
        let inverse = 1.0 / rhs;
        self[0] *= inverse;
        self[1] *= inverse;
        self[2] *= inverse;
    }
}

impl Neg for RGBSpectrum {
    type Output = Self;

    fn neg(self) -> Self::Output {
        debug_assert!(!self.is_nan());
        Self {
            components: [-self[0], -self[1], -self[2]],
        }
    }
}

impl Index<usize> for RGBSpectrum {
    type Output = Float;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < Self::NUM_SAMPLES);
        &self.components[index]
    }
}

impl IndexMut<usize> for RGBSpectrum {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.components[index]
    }
}
