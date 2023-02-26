use crate::{
    base::transform::Transform,
    geometries::{point2::Point2, point3::Point3, vec2::Vec2, vec3::Vec3},
    interactions::surface::SurfaceInteraction,
    utils::math::{Float, PI},
};

pub trait Texture<T: Copy + Send + Sync>: Send + Sync {
    fn evaluate(&self, si: &SurfaceInteraction) -> T;
}

pub trait TextureMapping2D: Send + Sync {
    fn map(&self, si: &SurfaceInteraction, dstdx: &mut Vec2, dstdy: &mut Vec2) -> Point2;
}

pub trait TextureMapping3D: Send + Sync {
    fn map(&self, si: &SurfaceInteraction, dpdx: &mut Vec3, dpdy: &mut Vec3) -> Point3;
}

pub struct UVMapping2D {
    su: Float,
    sv: Float,
    du: Float,
    dv: Float,
}

pub struct SphericalMapping2D {
    world_to_texture: Transform,
}

pub struct CylindricalMapping2D {
    world_to_texture: Transform,
}

pub struct PlanarMapping2D {
    vs: Vec3,
    vt: Vec3,
    ds: Float,
    dt: Float,
}

pub struct IdentityMapping3D {
    world_to_texture: Transform,
}

impl UVMapping2D {
    pub fn new(su: Float, sv: Float, du: Float, dv: Float) -> Self {
        Self { su, sv, du, dv }
    }
}

impl SphericalMapping2D {
    pub fn new(world_to_texture: Transform) -> Self {
        Self { world_to_texture }
    }

    fn sphere(&self, p: &Point3) -> Point2 {
        let v = (self.world_to_texture.transform_point(p) - Point3::default()).normalize();
        let theta = v.spherical_theta();
        let phi = v.spherical_phi();
        Point2::new(theta * (1.0 / PI), phi * (1.0 / PI))
    }
}

impl CylindricalMapping2D {
    pub fn new(world_to_texture: Transform) -> Self {
        Self { world_to_texture }
    }

    fn cylinder(&self, p: &Point3) -> Point2 {
        let v = (self.world_to_texture.transform_point(p) - Point3::default()).normalize();
        Point2::new((PI + v.y.atan2(v.x)) * (1.0 / (2.0 * PI)), v.z)
    }
}

impl PlanarMapping2D {
    pub fn new(vs: Vec3, vt: Vec3, ds: Float, dt: Float) -> Self {
        Self { vs, vt, ds, dt }
    }
}

impl IdentityMapping3D {
    pub fn new(world_to_texture: Transform) -> Self {
        Self { world_to_texture }
    }
}

impl TextureMapping2D for UVMapping2D {
    fn map(&self, si: &SurfaceInteraction, dstdx: &mut Vec2, dstdy: &mut Vec2) -> Point2 {
        *dstdx = Vec2::new(self.su * si.dudx, self.sv * si.dvdx);
        *dstdy = Vec2::new(self.su * si.dudy, self.sv * si.dvdy);
        Point2::new(self.su * si.uv.x + self.du, self.sv * si.uv.y + self.dv)
    }
}

impl TextureMapping2D for SphericalMapping2D {
    fn map(&self, si: &SurfaceInteraction, dstdx: &mut Vec2, dstdy: &mut Vec2) -> Point2 {
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
    fn map(&self, si: &SurfaceInteraction, dstdx: &mut Vec2, dstdy: &mut Vec2) -> Point2 {
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
    fn map(&self, si: &SurfaceInteraction, dstdx: &mut Vec2, dstdy: &mut Vec2) -> Point2 {
        let v = Vec3::from(si.p);
        *dstdx = Vec2::new(si.dpdx.dot(&self.vs), si.dpdx.dot(&self.vt));
        *dstdy = Vec2::new(si.dpdy.dot(&self.vs), si.dpdy.dot(&self.vt));
        Point2::new(self.ds + v.dot(&self.vs), self.dt + v.dot(&self.vt))
    }
}

impl TextureMapping3D for IdentityMapping3D {
    fn map(&self, si: &SurfaceInteraction, dpdx: &mut Vec3, dpdy: &mut Vec3) -> Point3 {
        *dpdx = self.world_to_texture.transform_vec(&si.dpdx);
        *dpdy = self.world_to_texture.transform_vec(&si.dpdy);
        self.world_to_texture.transform_point(&si.p)
    }
}
