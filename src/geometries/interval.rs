use std::{
    mem,
    ops::{Add, Mul, Sub},
};

use crate::base::constants::{Float, PI, PI_OVER_TWO};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub min: Float,
    pub max: Float,
}

impl Interval {
    pub fn new(x: Float, y: Float) -> Self {
        Self {
            min: x.min(y),
            max: x.max(y),
        }
    }

    pub fn find_zeros(
        c1: Float,
        c2: Float,
        c3: Float,
        c4: Float,
        c5: Float,
        theta: Float,
        t_interval: Self,
        zeros: &mut [Float; 8],
        zero_count: &mut usize,
        depth: i32,
    ) {
        // Evaluate motion derivative in interval form, return if no zeros.
        let range = Interval::from(c1)
            + (Interval::from(c2) + Interval::from(c3) * t_interval)
                * (Interval::from(2.0 * theta) * t_interval).cos()
            + (Interval::from(c4) + Interval::from(c5) * t_interval)
                * (Interval::from(2.0 * theta) * t_interval).sin();

        if range.min > 0.0 || range.max < 0.0 || range.min == range.max {
            return;
        }

        let mut mid = (t_interval.min + t_interval.max) * 0.5;
        if depth > 0 {
            // Split interval and check both resulting intervals.

            Interval::find_zeros(
                c1,
                c2,
                c3,
                c4,
                c5,
                theta,
                Interval::new(t_interval.min, mid),
                zeros,
                zero_count,
                depth - 1,
            );

            Interval::find_zeros(
                c1,
                c2,
                c3,
                c4,
                c5,
                theta,
                Interval::new(mid, t_interval.max),
                zeros,
                zero_count,
                depth - 1,
            );
        } else {
            // Use Newton's method to refine zero.
            for _ in 0..4 {
                let f_newton = c1
                    + (c2 + c3 * mid) * (2.0 * theta * mid).cos()
                    + (c4 + c5 * mid) * (2.0 * theta * mid).sin();

                let f_prime_newton = (c3 + 2.0 * (c4 + c5 * mid) * theta)
                    * (2.0 * mid * theta).cos()
                    + (c5 - 2.0 * (c2 + c3 * mid) * theta) * (2.0 * mid * theta).sin();

                if f_newton == 0.0 || f_prime_newton == 0.0 {
                    break;
                }

                mid = mid - f_newton / f_prime_newton;
            }

            if mid >= t_interval.min - 1e-3 && mid < t_interval.max + 1e-3 {
                zeros[*zero_count] = mid;
                *zero_count += 1;
            }
        }
    }

    pub fn sin(&self) -> Self {
        debug_assert!(self.min >= 0.0 && self.max <= 2.0001 * PI);

        let mut sin_min = self.min.sin();
        let mut sin_max = self.max.sin();

        if sin_min > sin_max {
            mem::swap(&mut sin_min, &mut sin_max);
        }

        if self.min < PI_OVER_TWO && self.max > PI_OVER_TWO {
            sin_max = 1.0;
        }

        if self.min < (3.0 / 2.0) * PI && self.max > (3.0 / 2.0) * PI {
            sin_min = -1.0;
        }

        Interval::new(sin_min, sin_max)
    }

    pub fn cos(&self) -> Self {
        debug_assert!(self.min >= 0.0 && self.max <= 2.0001 * PI);

        let mut cos_min = self.min.cos();
        let mut cos_max = self.max.cos();

        if cos_min > cos_max {
            mem::swap(&mut cos_min, &mut cos_max);
        }

        if self.min < PI && self.max > PI {
            cos_min = -1.0;
        }

        Interval::new(cos_min, cos_max)
    }
}

impl From<Float> for Interval {
    fn from(x: Float) -> Self {
        Self { min: x, max: x }
    }
}

impl Add for Interval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            min: self.min + rhs.min,
            max: self.max + rhs.max,
        }
    }
}

impl Add for &Interval {
    type Output = Interval;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            min: self.min + rhs.min,
            max: self.max + rhs.max,
        }
    }
}

impl Sub for Interval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            min: self.min - rhs.max,
            max: self.max - rhs.min,
        }
    }
}

impl Sub for &Interval {
    type Output = Interval;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            min: self.min - rhs.max,
            max: self.max - rhs.min,
        }
    }
}

impl Mul for Interval {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            min: (self.min * rhs.min)
                .min(self.max * rhs.min)
                .min(self.min * rhs.max)
                .min(self.max * rhs.max),
            max: (self.min * rhs.min)
                .max(self.max * rhs.min)
                .max(self.min * rhs.max)
                .max(self.max * rhs.max),
        }
    }
}

impl Mul for &Interval {
    type Output = Interval;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            min: (self.min * rhs.min)
                .min(self.max * rhs.min)
                .min(self.min * rhs.max)
                .min(self.max * rhs.max),
            max: (self.min * rhs.min)
                .max(self.max * rhs.min)
                .max(self.min * rhs.max)
                .max(self.max * rhs.max),
        }
    }
}
