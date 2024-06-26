use std::{
    mem,
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::base::{
    constants::{Float, MACHINE_EPSILON},
    math::{next_float_down, next_float_up},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EFloat {
    v: Float,
    low: Float,
    high: Float,
}

impl EFloat {
    pub fn new(v: Float, err: Float) -> Self {
        // Compute conservative bounds by rounding the endpoints away
        // from the middle. This will be over-conservative in
        // cases where +- error are exactly representable in
        // floating-point.
        if err == 0.0 {
            Self { v, low: v, high: v }
        } else {
            Self {
                v,
                low: next_float_down(v - err),
                high: next_float_up(v + err),
            }
        }
    }

    pub fn quadratic(a: Self, b: Self, c: Self, t0: &mut Self, t1: &mut Self) -> bool {
        // Find quadratic discriminant.
        let discrim = (b.v as f64) * (b.v as f64) - 4.0 * (a.v as f64) * (c.v as f64);
        if discrim < 0.0 {
            return false;
        }

        let root_discrim = discrim.sqrt() as Float;
        let root_discrim_f = EFloat::new(root_discrim, MACHINE_EPSILON * root_discrim);

        // Compute quadratic t values.
        let q = if Float::from(b) < 0.0 {
            -0.5 * (b - root_discrim_f)
        } else {
            -0.5 * (b + root_discrim_f)
        };

        *t0 = q / a;
        *t1 = c / q;
        if Float::from(*t0) > Float::from(*t1) {
            mem::swap(t0, t1)
        }

        true
    }

    pub fn lower_bound(&self) -> Float {
        self.low
    }

    pub fn upper_bound(&self) -> Float {
        self.high
    }

    pub fn absolute_error(&self) -> Float {
        next_float_up((self.high - self.v).abs().max((self.v - self.low).abs()))
    }

    pub fn sqrt(&self) -> Self {
        let v = self.v.sqrt();
        let low = next_float_down(self.low.sqrt());
        let high = next_float_up(self.high.sqrt());

        let f = Self { v, low, high };
        f.check();

        f
    }

    pub fn abs(&self) -> Self {
        if self.low >= 0.0 {
            // The entire interval is greater than zero.
            self.clone()
        } else if self.high <= 0.0 {
            // The entire interval is less than zero.
            let v = -self.v;
            let low = -self.high;
            let high = -self.low;

            let f = Self { v, low, high };
            f.check();

            f
        } else {
            // The interval straddles zero.
            let v = self.v.abs();
            let low = 0.0;
            let high = (-self.low).max(self.high);

            let f = Self { v, low, high };
            f.check();

            f
        }
    }

    pub fn check(&self) {
        if self.low.is_finite()
            && !self.low.is_nan()
            && self.high.is_finite()
            && !self.high.is_nan()
        {
            assert!(self.low <= self.high)
        }
    }
}

impl Default for EFloat {
    fn default() -> Self {
        Self {
            v: 0.0,
            low: 0.0,
            high: 0.0,
        }
    }
}

impl From<Float> for EFloat {
    fn from(v: Float) -> Self {
        Self::new(v, 0.0)
    }
}

impl From<EFloat> for Float {
    fn from(x: EFloat) -> Self {
        x.v
    }
}

impl Add for EFloat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let v = self.v + rhs.v;

        // Interval arithemetic addition, with the result rounded away from
        // the value in order to be conservative.
        let low = next_float_down(self.lower_bound() + rhs.lower_bound());
        let high = next_float_up(self.upper_bound() + rhs.upper_bound());

        let f = Self::Output { v, low, high };
        f.check();

        f
    }
}

impl Add<EFloat> for Float {
    type Output = EFloat;

    fn add(self, rhs: EFloat) -> Self::Output {
        EFloat::new(self, 0.0) + rhs
    }
}

impl Sub for EFloat {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let v = self.v - rhs.v;

        let low = next_float_down(self.lower_bound() - rhs.upper_bound());
        let high = next_float_up(self.upper_bound() - rhs.lower_bound());

        let f = Self::Output { v, low, high };
        f.check();

        f
    }
}

impl Sub<EFloat> for Float {
    type Output = EFloat;

    fn sub(self, rhs: EFloat) -> Self::Output {
        EFloat::new(self, 0.0) - rhs
    }
}

impl Sub<Float> for EFloat {
    type Output = Self;

    fn sub(self, rhs: Float) -> Self::Output {
        self - EFloat::new(rhs, 0.0)
    }
}

impl Mul for EFloat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let v = self.v * rhs.v;

        let products = [
            self.lower_bound() * rhs.lower_bound(),
            self.upper_bound() * rhs.lower_bound(),
            self.lower_bound() * rhs.upper_bound(),
            self.upper_bound() * rhs.upper_bound(),
        ];

        let low = next_float_down(
            products[0]
                .min(products[1])
                .min(products[2])
                .min(products[3]),
        );
        let high = next_float_up(
            products[0]
                .max(products[1])
                .max(products[2])
                .max(products[3]),
        );

        let f = Self::Output { v, low, high };
        f.check();

        f
    }
}

impl Mul<EFloat> for Float {
    type Output = EFloat;

    fn mul(self, rhs: EFloat) -> Self::Output {
        EFloat::new(self, 0.0) * rhs
    }
}

impl Div for EFloat {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let v = self.v / rhs.v;

        let quotients = [
            self.lower_bound() / rhs.lower_bound(),
            self.upper_bound() / rhs.lower_bound(),
            self.lower_bound() / rhs.upper_bound(),
            self.upper_bound() / rhs.upper_bound(),
        ];

        // The interval we're dividing by straddles zero, so
        // return an interval of everything.
        let low = if rhs.low < 0.0 && rhs.high > 0.0 {
            Float::NEG_INFINITY
        } else {
            next_float_down(
                quotients[0]
                    .min(quotients[1])
                    .min(quotients[2])
                    .min(quotients[3]),
            )
        };

        let high = if rhs.low < 0.0 && rhs.high > 0.0 {
            Float::INFINITY
        } else {
            next_float_up(
                quotients[0]
                    .max(quotients[1])
                    .max(quotients[2])
                    .max(quotients[3]),
            )
        };

        let f = Self::Output { v, low, high };
        f.check();

        f
    }
}

impl Div<EFloat> for Float {
    type Output = EFloat;

    fn div(self, rhs: EFloat) -> Self::Output {
        EFloat::new(self, 0.0) / rhs
    }
}

impl Neg for EFloat {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let v = -self.v;
        let low = -self.high;
        let high = -self.low;

        let f = Self::Output { v, low, high };
        f.check();

        f
    }
}
