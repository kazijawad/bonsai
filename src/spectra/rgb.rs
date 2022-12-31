use std::ops;

use crate::{base::spectrum::CoefficientSpectrum, utils::math::Float};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGBSpectrum([Float; 3]);

impl CoefficientSpectrum for RGBSpectrum {
    const NUM_SAMPLES: usize = 3;

    fn new(v: Float) -> Self {
        Self([v; Self::NUM_SAMPLES])
    }

    fn lerp(t: Float, a: &Self, b: &Self) -> Self {
        (*a) * (1.0 - t) + (*b) * t
    }

    fn sqrt(&self) -> Self {
        let result = Self([self[0].sqrt(), self[1].sqrt(), self[2].sqrt()]);
        debug_assert!(!result.is_nan());
        result
    }

    fn pow(&self, e: Float) -> Self {
        let result = Self([self[0].powf(e), self[1].powf(e), self[2].powf(e)]);
        debug_assert!(!result.is_nan());
        result
    }

    fn exp(&self) -> Self {
        let result = Self([self[0].exp(), self[1].exp(), self[2].exp()]);
        debug_assert!(!result.is_nan());
        result
    }

    fn clamp(&self, min: Float, max: Float) -> Self {
        let result = Self([
            self[0].clamp(min, max),
            self[1].clamp(min, max),
            self[2].clamp(min, max),
        ]);
        debug_assert!(!result.is_nan());
        result
    }

    fn max_component_value(&self) -> Float {
        self[0].max(self[1]).max(self[2])
    }

    fn is_black(&self) -> bool {
        for i in 0..Self::NUM_SAMPLES {
            if self[i] == 0.0 {
                return true;
            }
        }
        false
    }

    fn is_nan(&self) -> bool {
        for i in 0..Self::NUM_SAMPLES {
            if self[i].is_nan() {
                return true;
            }
        }
        false
    }
}

impl Default for RGBSpectrum {
    fn default() -> Self {
        Self([0.0; Self::NUM_SAMPLES])
    }
}

// ADDITION

impl ops::Add for RGBSpectrum {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self([self[0] + rhs[0], self[1] + rhs[1], self[2] + rhs[2]])
    }
}

impl ops::AddAssign for RGBSpectrum {
    fn add_assign(&mut self, rhs: Self) {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        self[0] += rhs[0];
        self[1] += rhs[1];
        self[2] += rhs[2];
    }
}

// SUBTRACTION

impl ops::Sub for RGBSpectrum {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self([self[0] - rhs[0], self[1] - rhs[1], self[2] - rhs[2]])
    }
}

impl ops::SubAssign for RGBSpectrum {
    fn sub_assign(&mut self, rhs: Self) {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        self[0] -= rhs[0];
        self[1] -= rhs[1];
        self[2] -= rhs[2];
    }
}

// MULTIPLICATION

impl ops::Mul for RGBSpectrum {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self([self[0] * rhs[0], self[1] * rhs[1], self[2] * rhs[2]])
    }
}

impl ops::Mul<Float> for RGBSpectrum {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self([self[0] * rhs, self[1] * rhs, self[2] * rhs])
    }
}

impl ops::MulAssign for RGBSpectrum {
    fn mul_assign(&mut self, rhs: Self) {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        self[0] *= rhs[0];
        self[1] *= rhs[1];
        self[2] *= rhs[2];
    }
}

impl ops::MulAssign<Float> for RGBSpectrum {
    fn mul_assign(&mut self, rhs: Float) {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        self[0] *= rhs;
        self[1] *= rhs;
        self[2] *= rhs;
    }
}

// DIVISION

impl ops::Div for RGBSpectrum {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self([self[0] / rhs[0], self[1] / rhs[1], self[2] / rhs[2]])
    }
}

impl ops::Div<Float> for RGBSpectrum {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan() && rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self([self[0] * rhs, self[1] * rhs, self[2] * rhs])
    }
}

impl ops::DivAssign for RGBSpectrum {
    fn div_assign(&mut self, rhs: Self) {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        self[0] /= rhs[0];
        self[1] /= rhs[1];
        self[2] /= rhs[2];
    }
}

impl ops::DivAssign<Float> for RGBSpectrum {
    fn div_assign(&mut self, rhs: Float) {
        debug_assert!(!self.is_nan() && !rhs.is_nan() && rhs != 0.0);
        let inverse = 1.0 / rhs;
        self[0] *= inverse;
        self[1] *= inverse;
        self[2] *= inverse;
    }
}

// NEGATION

impl ops::Neg for RGBSpectrum {
    type Output = Self;

    fn neg(self) -> Self::Output {
        debug_assert!(!self.is_nan());
        Self([-self[0], -self[1], -self[2]])
    }
}

// INDEXING

impl ops::Index<usize> for RGBSpectrum {
    type Output = Float;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < Self::NUM_SAMPLES);
        &self.0[index]
    }
}

impl ops::IndexMut<usize> for RGBSpectrum {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
