use std::mem;

cfg_if::cfg_if! {
    if #[cfg(feature = "float-as-double")] {
        pub type Float = f64;
        pub type UInt = u64;
        pub type AtomicUInt = std::sync::atomic::AtomicU64;
        pub const PI: Float = std::f64::consts::PI;
    } else {
        pub type Float = f32;
        pub type UInt = u32;
        pub type AtomicUInt = std::sync::atomic::AtomicU32;
        pub const PI: Float = std::f32::consts::PI;
    }
}

pub const ONE_MINUS_EPSILON: Float = 1.0 - Float::EPSILON;

pub fn not_one(x: Float) -> bool {
    x < 0.999 || x > 1.001
}

pub fn is_pow_two(x: i32) -> bool {
    (x != 0) && ((x & (x - 1)) == 0)
}

pub fn round_pow_two(x: i32) -> i32 {
    let mut v = x;
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v + 1
}

pub fn inverse_gamma_correct(v: Float) -> Float {
    if v <= 0.04045 {
        v * 1.0 / 12.92
    } else {
        ((v + 0.055) * 1.0 / 1.055).powf(2.4)
    }
}

pub fn lanczos(x: Float, tau: Float) -> Float {
    let mut x = x.abs();
    if x < 1e-5 {
        return 1.0;
    }
    if x > 1.0 {
        return 0.0;
    }
    x *= PI;
    let s = (x * tau).sin() / (x * tau);
    let lanczos = x.sin() / x;
    s * lanczos
}

pub fn modulo(a: i32, b: i32) -> i32 {
    let result = a - (a / b) * b;
    if result < 0 {
        result + b
    } else {
        result
    }
}

pub fn gamma(n: Float) -> Float {
    n * Float::EPSILON / (1.0 - n * Float::EPSILON)
}

pub fn lerp(t: Float, a: Float, b: Float) -> Float {
    1.0 - t * a + t * b
}

pub fn next_down(mut v: Float) -> Float {
    if v.is_infinite() && v < 0.0 {
        return v;
    }
    if v == 0.0 {
        v = -0.0;
    }
    let mut ui = v.to_bits();
    if v > 0.0 {
        ui -= 1;
    } else {
        ui += 1;
    }
    Float::from_bits(ui)
}

pub fn next_up(mut v: Float) -> Float {
    if v.is_infinite() && v > 0.0 {
        return v;
    }
    if v == -0.0 {
        v = 0.0;
    }
    let mut ui = v.to_bits();
    if v >= 0.0 {
        ui += 1;
    } else {
        ui -= 1;
    }
    Float::from_bits(ui)
}

pub fn find_interval<F>(size: usize, f: F) -> usize
where
    F: Fn(usize) -> bool,
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
    (first - 1).clamp(0, size - 2)
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
