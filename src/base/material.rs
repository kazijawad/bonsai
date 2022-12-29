use std::sync::Arc;

use serde::Deserialize;

#[derive(Debug)]
pub enum TransportMode {
    Radiance,
    Importance,
}

#[derive(Debug, Deserialize)]
pub enum MaterialType {
    Test,
}

pub trait Material: Send + Sync {}

pub struct TestMaterial;

impl TestMaterial {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl Material for TestMaterial {}
