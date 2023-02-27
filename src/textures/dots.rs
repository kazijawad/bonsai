use std::sync::Arc;

use crate::{
    base::texture::{Texture, TextureMapping2D},
    geometries::{point2::Point2, vec2::Vec2},
    interactions::surface::SurfaceInteraction,
    utils::noise::noise,
};

pub struct DotsTexture<T> {
    mapping: Box<dyn TextureMapping2D>,
    outside_dot: Arc<dyn Texture<T>>,
    inside_dot: Arc<dyn Texture<T>>,
}

impl<T> DotsTexture<T> {
    pub fn new(
        mapping: Box<dyn TextureMapping2D>,
        outside_dot: Arc<dyn Texture<T>>,
        inside_dot: Arc<dyn Texture<T>>,
    ) -> Self {
        Self {
            mapping,
            outside_dot,
            inside_dot,
        }
    }
}

impl<T: Copy + Send + Sync> Texture<T> for DotsTexture<T> {
    fn evaluate(&self, si: &SurfaceInteraction) -> T {
        let mut dstdx = Vec2::default();
        let mut dstdy = Vec2::default();
        let st = self.mapping.map(si, &mut dstdx, &mut dstdy);

        let s_cell = (st.x + 0.5).floor();
        let t_cell = (st.y + 0.5).floor();

        if noise(s_cell + 0.5, t_cell + 0.5, 0.5) > 0.0 {
            let radius = 0.35;
            let max_shift = 0.5 - radius;
            let s_center = s_cell + max_shift * noise(s_cell + 1.5, t_cell + 2.8, 0.5);
            let t_center = t_cell + max_shift * noise(s_cell + 4.5, t_cell + 9.8, 0.5);
            let dst = st - Point2::new(s_center, t_center);
            if dst.length_squared() < radius * radius {
                return self.inside_dot.evaluate(si);
            }
        }

        self.outside_dot.evaluate(si)
    }
}
