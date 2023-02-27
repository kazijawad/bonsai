use std::sync::Arc;

use crate::{
    base::{
        spectrum::{CoefficientSpectrum, Spectrum},
        texture::{Texture, TextureMapping2D},
    },
    geometries::vec2::Vec2,
    interactions::surface::SurfaceInteraction,
    utils::math::Float,
};

pub enum AAMethod {
    None,
    ClosedForm,
}

pub struct Checkerboard2DTexture {
    mapping: Box<dyn TextureMapping2D>,
    tex1: Arc<dyn Texture<Spectrum>>,
    tex2: Arc<dyn Texture<Spectrum>>,
    aa_method: AAMethod,
}

impl Checkerboard2DTexture {
    pub fn new(
        mapping: Box<dyn TextureMapping2D>,
        tex1: Arc<dyn Texture<Spectrum>>,
        tex2: Arc<dyn Texture<Spectrum>>,
        aa_method: AAMethod,
    ) -> Self {
        Self {
            mapping,
            tex1,
            tex2,
            aa_method,
        }
    }
}

impl Texture<Spectrum> for Checkerboard2DTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        let mut dstdx = Vec2::default();
        let mut dstdy = Vec2::default();
        let st = self.mapping.map(si, &mut dstdx, &mut dstdy);

        if let AAMethod::None = self.aa_method {
            if (st.x.floor() as i32) + (st.y.floor() as i32) % 2 == 0 {
                self.tex1.evaluate(si)
            } else {
                self.tex2.evaluate(si)
            }
        } else {
            // Evaluate single check if filter is entirely inside one of them.
            let ds = dstdx.x.abs().max(dstdy.x.abs());
            let dt = dstdx.y.abs().max(dstdy.y.abs());
            let s0 = st.x - ds;
            let s1 = st.x + ds;
            let t0 = st.y - dt;
            let t1 = st.y + dt;
            if s0.floor() == s1.floor() && t0.floor() == t1.floor() {
                // Use point sample becasue the filter region is
                // inside a single check.
                if (st.x.floor() as i32) + (st.y.floor() as i32) % 2 == 0 {
                    return self.tex1.evaluate(si);
                }
                return self.tex2.evaluate(si);
            }

            // Apply box filter to checkerboard region.
            let bump_int = |x: Float| -> Float {
                (x / 2.0).floor() + 2.0 * (x / 2.0 - (x / 2.0).floor() - 0.5).max(0.0)
            };
            let s_int = (bump_int(s1) - bump_int(s0)) / (2.0 * ds);
            let t_int = (bump_int(t1) - bump_int(t0)) / (2.0 * dt);
            let mut area2 = s_int + t_int - 2.0 * s_int * t_int;
            if ds > 1.0 || dt > 1.0 {
                area2 = 0.5;
            }

            Spectrum::new(1.0 - area2) * self.tex1.evaluate(si)
                + Spectrum::new(area2) * self.tex2.evaluate(si)
        }
    }
}
