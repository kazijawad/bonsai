use crate::{
    base::{
        constants::{Float, PI},
        transform::Transform,
    },
    geometries::{point2::Point2F, point3::Point3, vec2::Vec2F, vec3::Vec3},
    interactions::surface::SurfaceInteraction,
};

pub trait Texture<T: Send + Sync>: Send + Sync {
    fn evaluate(&self, si: &SurfaceInteraction) -> T;
}

pub trait TextureMapping2D: Send + Sync {
    fn map(&self, si: &SurfaceInteraction, dstdx: &mut Vec2F, dstdy: &mut Vec2F) -> Point2F;
}

pub trait TextureMapping3D: Send + Sync {
    fn map(&self, si: &SurfaceInteraction, dpdx: &mut Vec3, dpdy: &mut Vec3) -> Point3;
}

pub struct UVMapping2D {
    pub u_scale: Float,
    pub v_scale: Float,
    pub u_delta: Float,
    pub v_delta: Float,
}

pub struct SphericalMapping2D {
    pub world_to_texture: Transform,
}

pub struct CylindricalMapping2D {
    pub world_to_texture: Transform,
}

pub struct PlanarMapping2D {
    pub vs: Vec3,
    pub vt: Vec3,
    pub ds: Float,
    pub dt: Float,
}

pub struct IdentityMapping3D {
    world_to_texture: Transform,
}

impl SphericalMapping2D {
    fn sphere(&self, p: &Point3) -> Point2F {
        let v = (p.transform(&self.world_to_texture) - Point3::default()).normalize();
        let theta = v.spherical_theta();
        let phi = v.spherical_phi();
        Point2F::new(theta * (1.0 / PI), phi * (1.0 / PI))
    }
}

impl CylindricalMapping2D {
    fn cylinder(&self, p: &Point3) -> Point2F {
        let v = (p.transform(&self.world_to_texture) - Point3::default()).normalize();
        Point2F::new((PI + v.y.atan2(v.x)) * (1.0 / (2.0 * PI)), v.z)
    }
}

impl TextureMapping2D for UVMapping2D {
    fn map(&self, si: &SurfaceInteraction, dstdx: &mut Vec2F, dstdy: &mut Vec2F) -> Point2F {
        *dstdx = Vec2F::new(self.u_scale * si.dudx, self.v_scale * si.dvdx);
        *dstdy = Vec2F::new(self.u_scale * si.dudy, self.v_scale * si.dvdy);
        Point2F::new(
            self.u_scale * si.uv[0] + self.u_delta,
            self.v_scale * si.uv[1] + self.v_delta,
        )
    }
}

impl TextureMapping2D for SphericalMapping2D {
    fn map(&self, si: &SurfaceInteraction, dstdx: &mut Vec2F, dstdy: &mut Vec2F) -> Point2F {
        let st = self.sphere(&si.p);

        // Compute texture coordinate differentials for (u,v) mapping.
        const DELTA: Float = 0.1;
        let st_delta_x = self.sphere(&(si.p + DELTA * si.dpdx));
        *dstdx = (st_delta_x - st) / DELTA;
        let st_delta_y = self.sphere(&(si.p + DELTA * si.dpdy));
        *dstdy = (st_delta_y - st) / DELTA;

        // Handle mapping discontinuity for coordinate differentials.
        if dstdx.y > 0.5 {
            dstdx.y = 1.0 - dstdx.y;
        } else if dstdx.y < -0.5 {
            dstdx.y = -(dstdx.y + 1.0);
        }
        if dstdy.y > 0.5 {
            dstdy.y = 1.0 - dstdy.y;
        } else if dstdy.y < -0.5 {
            dstdy.y = -(dstdy.y + 1.0);
        }

        st
    }
}

impl TextureMapping2D for CylindricalMapping2D {
    fn map(&self, si: &SurfaceInteraction, dstdx: &mut Vec2F, dstdy: &mut Vec2F) -> Point2F {
        let st = self.cylinder(&si.p);

        // Compute texture coordinate differentials for (u,v) mapping.
        const DELTA: Float = 0.1;
        let st_delta_x = self.cylinder(&(si.p + DELTA * si.dpdx));
        *dstdx = (st_delta_x - st) / DELTA;
        let st_delta_y = self.cylinder(&(si.p + DELTA * si.dpdy));
        *dstdy = (st_delta_y - st) / DELTA;

        // Handle mapping discontinuity for coordinate differentials.
        if dstdx.y > 0.5 {
            dstdx.y = 1.0 - dstdx.y;
        } else if dstdx.y < -0.5 {
            dstdx.y = -(dstdx.y + 1.0);
        }
        if dstdy.y > 0.5 {
            dstdy.y = 1.0 - dstdy.y;
        } else if dstdy.y < -0.5 {
            dstdy.y = -(dstdy.y + 1.0);
        }

        st
    }
}

impl TextureMapping2D for PlanarMapping2D {
    fn map(&self, si: &SurfaceInteraction, dstdx: &mut Vec2F, dstdy: &mut Vec2F) -> Point2F {
        let v = Vec3::from(si.p);
        *dstdx = Vec2F::new(si.dpdx.dot(&self.vs), si.dpdx.dot(&self.vt));
        *dstdy = Vec2F::new(si.dpdy.dot(&self.vs), si.dpdy.dot(&self.vt));
        Point2F::new(self.ds + v.dot(&self.vs), self.dt + v.dot(&self.vt))
    }
}

impl TextureMapping3D for IdentityMapping3D {
    fn map(&self, si: &SurfaceInteraction, dpdx: &mut Vec3, dpdy: &mut Vec3) -> Point3 {
        *dpdx = si.dpdx.transform(&self.world_to_texture);
        *dpdy = si.dpdy.transform(&self.world_to_texture);
        si.p.transform(&self.world_to_texture)
    }
}

impl Default for UVMapping2D {
    fn default() -> Self {
        Self {
            u_scale: 1.0,
            v_scale: 1.0,
            u_delta: 0.0,
            v_delta: 0.0,
        }
    }
}
