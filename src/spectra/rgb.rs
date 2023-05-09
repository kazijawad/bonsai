use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

use crate::base::{
    constants::Float,
    spectrum::{rgb_to_xyz, xyz_to_rgb, Spectrum, RGB, XYZ},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGBSpectrum {
    components: [Float; 3],
}

impl RGBSpectrum {
    pub fn new(v: Float) -> Self {
        Self { components: [v; 3] }
    }

    pub fn splat(x: Float, y: Float, z: Float) -> Self {
        Self {
            components: [x, y, z],
        }
    }
}

impl Spectrum for RGBSpectrum {
    fn from_xyz(xyz: &XYZ) -> Self {
        let mut rgb = Self::default();
        xyz_to_rgb(xyz, &mut rgb.components);
        rgb
    }

    fn from_rgb(rgb: &RGB) -> Self {
        debug_assert!(!rgb[0].is_nan() && !rgb[1].is_nan() && !rgb[2].is_nan());
        Self {
            components: rgb.clone(),
        }
    }

    fn lerp(t: Float, a: &Self, b: &Self) -> Self {
        &(1.0 - t) * a + &t * b
    }

    fn len(&self) -> usize {
        self.components.len()
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

    fn powf(&self, e: Float) -> Self {
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
        self[0] == 0.0 && self[1] == 0.0 && self[2] == 0.0
    }

    fn is_nan(&self) -> bool {
        self[0].is_nan() || self[1].is_nan() || self[2].is_nan()
    }
}

impl Default for RGBSpectrum {
    fn default() -> Self {
        Self {
            components: [0.0; 3],
        }
    }
}

impl Add for RGBSpectrum {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] + rhs[0], self[1] + rhs[1], self[2] + rhs[2]],
        }
    }
}

impl Add for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] + rhs[0], self[1] + rhs[1], self[2] + rhs[2]],
        }
    }
}

impl Add<Float> for RGBSpectrum {
    type Output = Self;

    fn add(self, rhs: Float) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] + rhs, self[1] + rhs, self[2] + rhs],
        }
    }
}

impl Add<&Float> for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn add(self, rhs: &Float) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] + rhs, self[1] + rhs, self[2] + rhs],
        }
    }
}

impl Add<RGBSpectrum> for Float {
    type Output = RGBSpectrum;

    fn add(self, rhs: RGBSpectrum) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self + rhs[0], self + rhs[1], self + rhs[2]],
        }
    }
}

impl Add<&RGBSpectrum> for &Float {
    type Output = RGBSpectrum;

    fn add(self, rhs: &RGBSpectrum) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self + rhs[0], self + rhs[1], self + rhs[2]],
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

impl AddAssign<Float> for RGBSpectrum {
    fn add_assign(&mut self, rhs: Float) {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        self[0] += rhs;
        self[1] += rhs;
        self[2] += rhs;
    }
}

impl Sub for RGBSpectrum {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] - rhs[0], self[1] - rhs[1], self[2] - rhs[2]],
        }
    }
}

impl Sub for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] - rhs[0], self[1] - rhs[1], self[2] - rhs[2]],
        }
    }
}

impl Sub<Float> for RGBSpectrum {
    type Output = Self;

    fn sub(self, rhs: Float) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] - rhs, self[1] - rhs, self[2] - rhs],
        }
    }
}

impl Sub<&Float> for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn sub(self, rhs: &Float) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] - rhs, self[1] - rhs, self[2] - rhs],
        }
    }
}

impl Sub<RGBSpectrum> for Float {
    type Output = RGBSpectrum;

    fn sub(self, rhs: RGBSpectrum) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self - rhs[0], self - rhs[1], self - rhs[2]],
        }
    }
}

impl Sub<&RGBSpectrum> for &Float {
    type Output = RGBSpectrum;

    fn sub(self, rhs: &RGBSpectrum) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self - rhs[0], self - rhs[1], self - rhs[2]],
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

impl SubAssign<Float> for RGBSpectrum {
    fn sub_assign(&mut self, rhs: Float) {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        self[0] -= rhs;
        self[1] -= rhs;
        self[2] -= rhs;
    }
}

impl Mul for RGBSpectrum {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] * rhs[0], self[1] * rhs[1], self[2] * rhs[2]],
        }
    }
}

impl Mul for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn mul(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] * rhs[0], self[1] * rhs[1], self[2] * rhs[2]],
        }
    }
}

impl Mul<Float> for RGBSpectrum {
    type Output = Self;

    fn mul(self, rhs: Float) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] * rhs, self[1] * rhs, self[2] * rhs],
        }
    }
}

impl Mul<&Float> for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn mul(self, rhs: &Float) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] * rhs, self[1] * rhs, self[2] * rhs],
        }
    }
}

impl Mul<RGBSpectrum> for Float {
    type Output = RGBSpectrum;

    fn mul(self, rhs: RGBSpectrum) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self * rhs[0], self * rhs[1], self * rhs[2]],
        }
    }
}

impl Mul<&RGBSpectrum> for &Float {
    type Output = RGBSpectrum;

    fn mul(self, rhs: &RGBSpectrum) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self * rhs[0], self * rhs[1], self * rhs[2]],
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
        Self::Output {
            components: [self[0] / rhs[0], self[1] / rhs[1], self[2] / rhs[2]],
        }
    }
}

impl Div for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn div(self, rhs: Self) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan());
        Self::Output {
            components: [self[0] / rhs[0], self[1] / rhs[1], self[2] / rhs[2]],
        }
    }
}

impl Div<Float> for RGBSpectrum {
    type Output = Self;

    fn div(self, rhs: Float) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan() && rhs != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            components: [self[0] * inverse, self[1] * inverse, self[2] * inverse],
        }
    }
}

impl Div<&Float> for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn div(self, rhs: &Float) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan() && (*rhs) != 0.0);
        let inverse = 1.0 / rhs;
        Self::Output {
            components: [self[0] * inverse, self[1] * inverse, self[2] * inverse],
        }
    }
}

impl Div<RGBSpectrum> for Float {
    type Output = RGBSpectrum;

    fn div(self, rhs: RGBSpectrum) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan() && rhs != RGBSpectrum::default());
        Self::Output {
            components: [self / rhs[0], self / rhs[1], self / rhs[2]],
        }
    }
}

impl Div<&RGBSpectrum> for &Float {
    type Output = RGBSpectrum;

    fn div(self, rhs: &RGBSpectrum) -> Self::Output {
        debug_assert!(!self.is_nan() && !rhs.is_nan() && rhs != &RGBSpectrum::default());
        Self::Output {
            components: [self / rhs[0], self / rhs[1], self / rhs[2]],
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
        Self::Output {
            components: [-self[0], -self[1], -self[2]],
        }
    }
}

impl Neg for &RGBSpectrum {
    type Output = RGBSpectrum;

    fn neg(self) -> Self::Output {
        debug_assert!(!self.is_nan());
        Self::Output {
            components: [-self[0], -self[1], -self[2]],
        }
    }
}

impl Index<usize> for RGBSpectrum {
    type Output = Float;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < 3);
        &self.components[index]
    }
}

impl IndexMut<usize> for RGBSpectrum {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < 3);
        &mut self.components[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::spectra::rgb::RGBSpectrum;

    #[test]
    fn add() {
        let a = RGBSpectrum::new(1.0);
        let b = RGBSpectrum::new(2.0);
        let c = RGBSpectrum::new(3.0);

        let x = 2.0;

        assert_eq!(a + b, c);
        assert_eq!(&a + &b, c);

        assert_eq!(a + x, c);
        assert_eq!(&a + &x, c);

        assert_eq!(x + a, c);
        assert_eq!(&x + &a, c);
    }

    #[test]
    fn add_assign() {
        let mut a = RGBSpectrum::new(1.0);
        a += RGBSpectrum::new(1.0);
        a += 3.0;
        assert_eq!(a, RGBSpectrum::new(5.0));
    }

    #[test]
    fn sub() {
        let a = RGBSpectrum::new(1.0);
        let b = RGBSpectrum::new(2.0);
        let c = RGBSpectrum::new(3.0);

        let x = 2.0;

        assert_eq!(c - b, a);
        assert_eq!(&c - &b, a);

        assert_eq!(c - x, a);
        assert_eq!(&c - &x, a);

        assert_eq!(x - a, a);
        assert_eq!(&x - &a, a);
    }

    #[test]
    fn sub_assign() {
        let mut a = RGBSpectrum::new(3.0);
        a -= RGBSpectrum::new(2.0);
        a -= 5.0;
        assert_eq!(a, RGBSpectrum::new(-4.0));
    }

    #[test]
    fn mul() {
        let a = RGBSpectrum::new(1.0);
        let b = RGBSpectrum::new(2.0);
        let c = RGBSpectrum::new(2.0);

        let x = 2.0;

        assert_eq!(a * b, c);
        assert_eq!(&a * &b, c);

        assert_eq!(a * x, c);
        assert_eq!(&a * &x, c);

        assert_eq!(x * a, c);
        assert_eq!(&x * &a, c);
    }

    #[test]
    fn mul_assign() {
        let mut a = RGBSpectrum::new(1.0);
        a *= RGBSpectrum::new(2.0);
        a *= 2.0;
        assert_eq!(a, RGBSpectrum::new(4.0));
    }

    #[test]
    fn div() {
        let a = RGBSpectrum::new(1.0);
        let b = RGBSpectrum::new(2.0);
        let c = RGBSpectrum::new(0.5);

        let x = 2.0;

        assert_eq!(a / b, c);
        assert_eq!(&a / &b, c);

        assert_eq!(a / x, c);
        assert_eq!(&a / &x, c);

        assert_eq!(x / a, 1.0 / c);
        assert_eq!(&x / &a, 1.0 / c);
    }

    #[test]
    fn div_assign() {
        let mut a = RGBSpectrum::new(1.0);
        a /= RGBSpectrum::new(2.0);
        a /= 2.0;
        assert_eq!(a, RGBSpectrum::new(0.25));
    }

    #[test]
    fn neg() {
        let a = -RGBSpectrum::new(1.0);
        let b = -&RGBSpectrum::new(1.0);
        let c = RGBSpectrum::new(-1.0);
        assert_eq!(a, c);
        assert_eq!(b, c);
    }

    #[test]
    fn index() {
        let a = RGBSpectrum::new(0.5);
        assert_eq!(a[0], 0.5);
        assert_eq!(a[1], 0.5);
        assert_eq!(a[2], 0.5);
    }

    #[test]
    fn index_mut() {
        let mut a = RGBSpectrum::new(0.5);
        a[1] -= 0.5;
        assert_eq!(a[0], 0.5);
        assert_eq!(a[1], 0.0);
        assert_eq!(a[2], 0.5);
    }
}
