use crate::base::constants::{Float, PRIMES};

pub fn radical_inverse(base_index: usize, mut a: u64) -> Float {
    let base = PRIMES[base_index];
    // We have to stop once reversed_digits is >= limit otherwise the
    // next digit of |a| may cause reversed_digits to overflow.
    let limit: u64 = !0 / base - base;
    let inverse_base = 1.0 / base as Float;
    let mut inverse_base_m = 1.0;
    let mut reversed_digits: u64 = 0;
    while a != 0 && reversed_digits < limit {
        // Extract least significant digit.
        let next = a / base;
        let digit = a - next * base;
        reversed_digits = reversed_digits * base + digit;
        inverse_base_m *= inverse_base;
        a = next;
    }
    Float::min(
        reversed_digits as Float * inverse_base_m,
        1.0 - Float::EPSILON,
    )
}
