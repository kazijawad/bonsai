use std::{mem, ops};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    low: f32,
    high: f32,
}

impl Interval {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            low: x.min(y),
            high: x.max(y),
        }
    }

    pub fn find_zeros(
        c1: f32,
        c2: f32,
        c3: f32,
        c4: f32,
        c5: f32,
        theta: f32,
        t_interval: Self,
        zeros: &mut [f32; 8],
        zero_count: &mut u32,
        depth: i32,
    ) {
        // Evaluate motion derivative in interval form, return if no zeros
        let range = Interval::from(c1)
            + (Interval::from(c2) + Interval::from(c3) * t_interval)
                * (Interval::from(2.0 * theta) * t_interval).cos()
            + (Interval::from(c4) + Interval::from(c5) * t_interval)
                * (Interval::from(2.0 * theta) * t_interval).sin();
        if range.low > 0.0 || range.high < 0.0 || range.low == range.high {
            return;
        }
        if depth > 0 {
            // Split interval and check both resulting intervals.
            let mid = (t_interval.low + t_interval.high) * 0.5;
            Interval::find_zeros(
                c1,
                c2,
                c3,
                c4,
                c5,
                theta,
                Interval::new(t_interval.low, mid),
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
                Interval::new(mid, t_interval.high),
                zeros,
                zero_count,
                depth - 1,
            );
        } else {
            // Use Newton's method to refine zero.
            let mut t_newton = (t_interval.low + t_interval.high) * 0.5;
            for _ in 0..4 {
                let f_newton = c1
                    + (c2 + c3 * t_newton) * (2.0 * theta * t_newton).cos()
                    + (c4 + c5 * t_newton) * (2.0 * theta * t_newton).sin();
                let f_prime_newton = (c3 + 2.0 * (c4 + c5 * t_newton) * theta)
                    * (2.0 * t_newton * theta).cos()
                    + (c5 - 2.0 * (c2 + c3 * t_newton) * theta) * (2.0 * t_newton * theta).sin();
                if f_newton == 0.0 || f_prime_newton == 0.0 {
                    break;
                }
                t_newton = t_newton - f_newton / f_prime_newton;
            }
            if t_newton >= t_interval.low - 1e-3 && t_newton < t_interval.high + 1e-3 {
                zeros[*zero_count as usize] = t_newton;
                *zero_count += 1;
            }
        }
    }

    pub fn sin(&self) -> Self {
        debug_assert!(self.low >= 0.0 && self.high <= 2.0001 * std::f32::consts::PI);
        let mut sin_low = self.low.sin();
        let mut sin_high = self.high.sin();

        if sin_low > sin_high {
            mem::swap(&mut sin_low, &mut sin_high);
        }

        if self.low < std::f32::consts::PI / 2.0 && self.high > std::f32::consts::PI / 2.0 {
            sin_high = 1.0;
        }

        if self.low < (3.0 / 2.0) * std::f32::consts::PI
            && self.high > (3.0 / 2.0) * std::f32::consts::PI
        {
            sin_low = -1.0;
        }

        Interval::new(sin_low, sin_high)
    }

    pub fn cos(&self) -> Self {
        debug_assert!(self.low >= 0.0 && self.high <= 2.0001 * std::f32::consts::PI);
        let mut cos_low = self.low.cos();
        let mut cos_high = self.high.cos();

        if cos_low > cos_high {
            mem::swap(&mut cos_low, &mut cos_high);
        }

        if self.low < std::f32::consts::PI && self.high > std::f32::consts::PI {
            cos_low = -1.0;
        }

        Interval::new(cos_low, cos_high)
    }
}

// TYPE CONVERSION

impl From<f32> for Interval {
    fn from(x: f32) -> Self {
        Self { low: x, high: x }
    }
}

// ADDITION

impl ops::Add for Interval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            low: self.low + rhs.low,
            high: self.high + rhs.high,
        }
    }
}

impl ops::Add for &Interval {
    type Output = Interval;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            low: self.low + rhs.low,
            high: self.high + rhs.high,
        }
    }
}

// SUBTRACTION

impl ops::Sub for Interval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            low: self.low - rhs.high,
            high: self.high - rhs.low,
        }
    }
}

impl ops::Sub for &Interval {
    type Output = Interval;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            low: self.low - rhs.high,
            high: self.high - rhs.low,
        }
    }
}

// MULTIPLICATION

impl ops::Mul for Interval {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            low: (self.low * rhs.low)
                .min(self.high * rhs.low)
                .min(self.low * rhs.high)
                .min(self.high * rhs.high),
            high: (self.low * rhs.low)
                .max(self.high * rhs.low)
                .max(self.low * rhs.high)
                .max(self.high * rhs.high),
        }
    }
}

impl ops::Mul for &Interval {
    type Output = Interval;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            low: (self.low * rhs.low)
                .min(self.high * rhs.low)
                .min(self.low * rhs.high)
                .min(self.high * rhs.high),
            high: (self.low * rhs.low)
                .max(self.high * rhs.low)
                .max(self.low * rhs.high)
                .max(self.high * rhs.high),
        }
    }
}
