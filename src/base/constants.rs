cfg_if::cfg_if! {
    if #[cfg(feature = "float-precision")] {
        pub type Float = f64;
        pub const PI: Float = std::f64::consts::PI;
    } else {
        pub type Float = f32;
        pub const PI: Float = std::f32::consts::PI;
    }
}

pub const PI_OVER_TWO: Float = PI / 2.0;
pub const INV_PI: Float = 1.0 / PI;
pub const INV_TWO_PI: Float = 1.0 / (2.0 * PI);

pub const ONE_MINUS_EPSILON: Float = 1.0 - Float::EPSILON;
pub const MACHINE_EPSILON: Float = Float::EPSILON * 0.5;
