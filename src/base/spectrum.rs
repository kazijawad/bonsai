use std::{fmt::Debug, ops};

use crate::utils::math::Float;

pub enum SpectrumType {
    Reflectance,
    Illuminant,
}

pub type XYZ = [Float; 3];
pub type RGB = [Float; 3];

pub trait CoefficientSpectrum:
    Debug
    + Clone
    + Copy
    + PartialEq
    + Send
    + Sync
    + Sized
    + Default
    + ops::Add
    + ops::AddAssign
    + ops::Sub
    + ops::SubAssign
    + ops::Mul
    + ops::Mul<Float>
    + ops::MulAssign
    + ops::MulAssign<Float>
    + ops::Div
    + ops::Div<Float>
    + ops::DivAssign
    + ops::DivAssign<Float>
    + ops::Neg
    + ops::Index<usize>
    + ops::IndexMut<usize>
{
    const NUM_SAMPLES: usize;

    fn new(v: Float) -> Self;

    fn lerp(t: Float, a: &Self, b: &Self) -> Self;

    fn sqrt(&self) -> Self;

    fn pow(&self, e: Float) -> Self;

    fn exp(&self) -> Self;

    fn clamp(&self, min: Float, max: Float) -> Self;

    fn max_component_value(&self) -> Float;

    fn y(&self) -> Float;

    fn is_black(&self) -> bool;

    fn is_nan(&self) -> bool;

    fn is_initialized(&self) -> bool;
}

pub fn xyz_to_rgb(xyz: &XYZ, rgb: &mut RGB) {
    rgb[0] = 3.240479 * xyz[0] - 1.537150 * xyz[1] - 0.498535 * xyz[2];
    rgb[1] = -0.969256 * xyz[0] + 1.875991 * xyz[1] + 0.041556 * xyz[2];
    rgb[2] = 0.055648 * xyz[0] - 0.204043 * xyz[1] + 1.057311 * xyz[2];
}

pub fn rgb_to_xyz(rgb: &RGB, xyz: &mut XYZ) {
    xyz[0] = 0.412453 * rgb[0] + 0.357580 * rgb[1] + 0.180423 * rgb[2];
    xyz[1] = 0.212671 * rgb[0] + 0.715160 * rgb[1] + 0.072169 * rgb[2];
    xyz[2] = 0.019334 * rgb[0] + 0.119193 * rgb[1] + 0.950227 * rgb[2];
}
