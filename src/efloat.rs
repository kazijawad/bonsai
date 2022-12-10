use std::{mem, ops};

use crate::math::{self, Float};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EFloat {
    v: f32,
    low: f32,
    high: f32,
}

impl EFloat {
    pub fn new(v: f32, err: f32) -> Self {
        // Compute conservative bounds by rounding the endpoints away
        // from the middle. This will be over-conservative in
        // cases where +- error are exactly representable in
        // floating-point.
        let low = if err == 0.0 {
            v
        } else {
            math::next_down(v - err)
        };
        let high = if err == 0.0 {
            v
        } else {
            math::next_up(v + err)
        };
        Self { v, low, high }
    }

    pub fn quadratic(a: Self, b: Self, c: Self, t0: &mut Self, t1: &mut Self) -> bool {
        // Find quadratic discriminant.
        let discriminant = (b.v as f64) * (b.v as f64) - 4.0 * (a.v as f64) * (c.v as f64);
        if discriminant < 0.0 {
            return false;
        }
        let root_discriminant = discriminant.sqrt();

        let float_root_discriminant = EFloat::new(
            root_discriminant as f32,
            (f64::EPSILON * root_discriminant) as f32,
        );

        // Compute quadratic t values.
        let q = if Float::from(b) < 0.0 {
            -0.5 * (b - float_root_discriminant)
        } else {
            -0.5 * (b + float_root_discriminant)
        };

        *t0 = q / a;
        *t1 = c / q;
        if Float::from(*t0) > Float::from(*t1) {
            mem::swap(t0, t1)
        }

        true
    }

    pub fn lower_bound(&self) -> f32 {
        self.low
    }

    pub fn upper_bound(&self) -> f32 {
        self.high
    }

    pub fn absolute_error(&self) -> f32 {
        math::next_up((self.high - self.v).abs().max((self.v - self.low).abs()))
    }

    pub fn sqrt(&self) -> Self {
        let v = self.v.sqrt();
        let low = math::next_down(self.low.sqrt());
        let high = math::next_up(self.high.sqrt());

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

// TYPE CONVERSION

impl From<EFloat> for Float {
    fn from(x: EFloat) -> Self {
        x.v
    }
}

// ADDITION

impl ops::Add for EFloat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let v = self.v + rhs.v;

        // Interval arithemetic addition, with the result rounded away from
        // the value in order to be conservative.
        let low = math::next_down(self.lower_bound() + rhs.lower_bound());
        let high = math::next_up(self.upper_bound() + rhs.upper_bound());

        let f = Self::Output { v, low, high };
        f.check();

        f
    }
}

impl ops::Add<EFloat> for f32 {
    type Output = EFloat;

    fn add(self, rhs: EFloat) -> Self::Output {
        EFloat::new(self, 0.0) + rhs
    }
}

// SUBTRACTION

impl ops::Sub for EFloat {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let v = self.v - rhs.v;

        let low = math::next_down(self.lower_bound() - rhs.lower_bound());
        let high = math::next_up(self.upper_bound() - rhs.upper_bound());

        let f = Self::Output { v, low, high };
        f.check();

        f
    }
}

impl ops::Sub<EFloat> for f32 {
    type Output = EFloat;

    fn sub(self, rhs: EFloat) -> Self::Output {
        EFloat::new(self, 0.0) - rhs
    }
}

// MULTIPLICATION

impl ops::Mul for EFloat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let v = self.v * rhs.v;

        let products = [
            self.lower_bound() * rhs.lower_bound(),
            self.upper_bound() * rhs.lower_bound(),
            self.lower_bound() * rhs.upper_bound(),
            self.upper_bound() * rhs.upper_bound(),
        ];

        let low = math::next_down(
            products[0]
                .min(products[1])
                .min(products[2])
                .min(products[3]),
        );
        let high = math::next_up(
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

impl ops::Mul<EFloat> for f32 {
    type Output = EFloat;

    fn mul(self, rhs: EFloat) -> Self::Output {
        EFloat::new(self, 0.0) * rhs
    }
}

// DIVISION

impl ops::Div for EFloat {
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
            f32::NEG_INFINITY
        } else {
            math::next_down(
                quotients[0]
                    .min(quotients[1])
                    .min(quotients[2])
                    .min(quotients[3]),
            )
        };

        let high = if rhs.low < 0.0 && rhs.high > 0.0 {
            f32::INFINITY
        } else {
            math::next_down(
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

impl ops::Div<EFloat> for f32 {
    type Output = EFloat;

    fn div(self, rhs: EFloat) -> Self::Output {
        EFloat::new(self, 0.0) / rhs
    }
}

// NEGATION

impl ops::Neg for EFloat {
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
