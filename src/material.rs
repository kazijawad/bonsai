pub trait Material: Send + Sync {}

pub struct TestMaterial;

impl TestMaterial {
    pub fn new() -> Self {
        Self
    }
}

impl Material for TestMaterial {}