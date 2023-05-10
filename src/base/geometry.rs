use crate::{
    base::constants::{Float, PI},
    geometries::vec3::Vec3,
};

pub fn spherical_theta(v: &Vec3) -> Float {
    v.z.clamp(-1.0, 1.0).acos()
}

pub fn spherical_phi(v: &Vec3) -> Float {
    let p = v.y.atan2(v.x);
    if p < 0.0 {
        p + 2.0 * PI
    } else {
        p
    }
}
