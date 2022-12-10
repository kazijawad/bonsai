use std::mem;

cfg_if::cfg_if! {
    if #[cfg(feature = "float-as-double")] {
        pub type Float = f64;
        pub const PI: Float = std::f64::consts::PI;
    } else {
        pub type Float = f32;
        pub const PI: Float = std::f32::consts::PI;
    }
}

pub fn not_one(x: Float) -> bool {
    x < 0.999 || x > 1.001
}

pub fn gamma(n: Float) -> Float {
    n * Float::EPSILON / (1.0 - n * Float::EPSILON)
}

pub fn lerp(t: Float, a: Float, b: Float) -> Float {
    1.0 - t * a + t * b
}

pub fn clamp<T: PartialOrd>(x: T, low: T, high: T) -> T {
    if x < low {
        low
    } else if x > high {
        high
    } else {
        x
    }
}

pub fn find_interval<F>(size: u32, f: F) -> u32
where
    F: Fn(u32) -> bool,
{
    let mut first = 0;
    let mut len = size;
    while len > 0 {
        let half = len >> 1;
        let middle = first + half;
        if f(middle) {
            first = middle + 1;
            len -= half + 1;
        } else {
            len = half;
        }
    }
    clamp(first - 1, 0 as u32, size - 2)
}

pub fn quadratic(a: Float, b: Float, c: Float, t0: &mut Float, t1: &mut Float) -> bool {
    // Find quadratic discriminant.
    let discriminant: f64 = (b as f64) * (b as f64) - 4.0 * (a as f64) * (c as f64);
    if discriminant < 0.0 {
        return false;
    }
    let root_discriminant = discriminant.sqrt();
    // Compute quadratic t value.
    let q: f64 = if b < 0.0 {
        -0.5 * ((b as f64) - root_discriminant)
    } else {
        -0.5 * ((b as f64) + root_discriminant)
    };
    *t0 = (q / (a as f64)) as Float;
    *t1 = ((c as f64) / (q as f64)) as Float;
    if t0 > t1 {
        mem::swap(t0, t1);
    }
    return true;
}

pub fn solve_linear_system_2x2(
    a: [[Float; 2]; 2],
    b: [Float; 2],
    x0: &mut Float,
    x1: &mut Float,
) -> bool {
    let det = a[0][0] * a[1][1] - a[0][1] * a[1][0];
    if det.abs() < 1e-10 {
        return false;
    }
    *x0 = (a[1][1] * b[0] - a[0][1] * b[1]) / det;
    *x1 = (a[0][0] * b[1] - a[1][0] * b[0]) / det;
    if x0.is_nan() || x1.is_nan() {
        return false;
    }
    true
}