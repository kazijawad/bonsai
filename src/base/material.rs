use std::sync::Arc;

pub enum TransportMode {
    Radiance,
    Importance,
}

pub trait Material: Send + Sync {}

pub struct TestMaterial;

impl TestMaterial {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl Material for TestMaterial {}
