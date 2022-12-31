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

    fn is_black(&self) -> bool;

    fn is_nan(&self) -> bool;
}
