use crate::base::constants::Float;

pub type XYZ = [Float; 3];
pub type RGB = [Float; 3];

pub trait Spectrum: Send + Sync {
    fn from_xyz(xyz: &XYZ) -> Self;
    fn from_rgb(rgb: &RGB) -> Self;

    fn lerp(t: Float, a: &Self, b: &Self) -> Self;

    fn len(&self) -> usize;

    fn sqrt(&self) -> Self;

    fn powf(&self, e: Float) -> Self;

    fn exp(&self) -> Self;

    fn clamp(&self, min: Float, max: Float) -> Self;

    fn max_component_value(&self) -> Float;

    fn y(&self) -> Float;

    fn to_xyz(&self, xyz: &mut XYZ);
    fn to_rgb(&self, rgb: &mut RGB);

    fn is_black(&self) -> bool;

    fn is_nan(&self) -> bool;
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
