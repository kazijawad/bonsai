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

pub fn solve_linear_system(a: [[Float; 2]; 2], b: [Float; 2]) -> (bool, Float, Float) {
    let det = a[0][0] * a[1][1] - a[0][1] * a[1][0];
    if det.abs() < 1e-10 {
        return (false, 0.0, 0.0);
    }
    let x0 = (a[1][1] * b[0] - a[0][1] * b[1]) / det;
    let x1 = (a[0][0] * b[1] - a[1][0] * b[0]) / det;
    if x0.is_nan() || x1.is_nan() {
        return (false, 0.0, 0.0);
    }
    (true, x0, x1)
}
