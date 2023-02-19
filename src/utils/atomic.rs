use std::sync::atomic::Ordering;

use crate::utils::math::{AtomicUInt, Float, UInt};

#[derive(Debug)]
pub struct AtomicFloat {
    bits: AtomicUInt,
}

impl AtomicFloat {
    pub fn new(v: Float) -> Self {
        Self {
            bits: AtomicUInt::new(Float::to_bits(v)),
        }
    }

    pub fn add(&self, v: Float) {
        let mut old_bits = self.bits.load(Ordering::SeqCst);
        let new_bits: UInt = 0;
        loop {
            let new_bits = Float::to_bits(Float::from_bits(old_bits) + v);
            match self.bits.compare_exchange_weak(
                old_bits,
                new_bits,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(x) => old_bits = x,
            }
        }
    }
}

impl Into<Float> for AtomicFloat {
    fn into(self) -> Float {
        Float::from_bits(self.bits.into_inner())
    }
}
