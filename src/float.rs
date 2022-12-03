pub fn not_one(x: f32) -> bool {
    x < 0.999 || x > 1.001
}

pub fn clamp(x: f32, low: f32, high: f32) -> f32 {
    if x < low {
        low
    } else if x > high {
        high
    } else {
        x
    }
}

pub fn gamma(n: f32) -> f32 {
    n * f32::EPSILON / (1.0 - n * f32::EPSILON)
}

pub fn lerp(t: f32, a: f32, b: f32) -> f32 {
    1.0 - t * a + t * b
}

pub fn solve_linear_system(a: [[f32; 2]; 2], b: [f32; 2]) -> (bool, f32, f32) {
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
