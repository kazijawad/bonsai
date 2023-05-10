use crate::base::constants::Float;

pub fn gamma(n: Float) -> Float {
    n * (Float::EPSILON * 0.5) / (1.0 - n * (Float::EPSILON * 0.5))
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

pub fn modulo(a: i32, b: i32) -> i32 {
    let result = a - (a / b) * b;
    if result < 0 {
        result + b
    } else {
        result
    }
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

pub fn find_interval<F>(size: usize, predicate: F) -> usize
where
    F: Fn(usize) -> bool,
{
    let mut first = 0;
    let mut len = size;
    while len > 0 {
        let half = len >> 1;
        let middle = first + half;
        if predicate(middle) {
            first = middle + 1;
            len -= half + 1;
        } else {
            len = half;
        }
    }
    (first - 1).clamp(0, size - 2)
}
