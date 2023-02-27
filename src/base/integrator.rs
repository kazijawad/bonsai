use crate::base::scene::Scene;

pub trait Integrator: Send + Sync {
    fn render(&self, scene: &Scene);
}
