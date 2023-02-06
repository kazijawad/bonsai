use crate::{
    geometries::{point3::Point3, vec3::Vec3},
    utils::math::Float,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    pub t_max: Float,
    pub time: Float,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayDifferential {
    pub ray: Ray,
    pub rx_origin: Point3,
    pub ry_origin: Point3,
    pub rx_direction: Vec3,
    pub ry_direction: Vec3,
    pub has_differentials: bool,
}

impl Ray {
    pub fn new(origin: &Point3, direction: &Vec3, t_max: Float, time: Float) -> Self {
        Self {
            origin: origin.clone(),
            direction: direction.clone(),
            t_max,
            time,
        }
    }

    pub fn at(&self, t: Float) -> Point3 {
        self.origin + self.direction * t
    }

    pub fn is_nan(&self) -> bool {
        self.origin.is_nan() || self.direction.is_nan() || self.t_max.is_nan()
    }
}

impl RayDifferential {
    pub fn new(origin: &Point3, direction: &Vec3, t_max: Float, time: Float) -> Self {
        Self {
            ray: Ray::new(origin, direction, t_max, time),
            rx_origin: Point3::default(),
            ry_origin: Point3::default(),
            rx_direction: Vec3::default(),
            ry_direction: Vec3::default(),
            has_differentials: false,
        }
    }

    pub fn at(&self, t: Float) -> Point3 {
        self.ray.origin + self.ray.direction * t
    }

    pub fn scale_differentials(&mut self, s: Float) {
        self.rx_origin = self.ray.origin + (self.rx_origin - self.ray.origin) * s;
        self.ry_origin = self.ray.origin + (self.ry_origin - self.ray.origin) * s;
        self.rx_direction = self.ray.direction + (self.rx_direction - self.ray.direction) * s;
        self.ry_direction = self.ray.direction + (self.ry_direction - self.ray.direction) * s;
    }

    pub fn is_nan(&self) -> bool {
        self.ray.origin.is_nan() || self.ray.direction.is_nan() || self.ray.t_max.is_nan()
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            origin: Point3::default(),
            direction: Vec3::default(),
            t_max: Float::INFINITY,
            time: 0.0,
        }
    }
}

impl Default for RayDifferential {
    fn default() -> Self {
        Self {
            ray: Ray::default(),
            rx_origin: Point3::default(),
            ry_origin: Point3::default(),
            rx_direction: Vec3::default(),
            ry_direction: Vec3::default(),
            has_differentials: false,
        }
    }
}

// TYPE CONVERSION

impl From<RayDifferential> for Ray {
    fn from(diff: RayDifferential) -> Self {
        Self {
            origin: diff.ray.origin,
            direction: diff.ray.direction,
            t_max: diff.ray.t_max,
            time: diff.ray.time,
        }
    }
}

impl From<Ray> for RayDifferential {
    fn from(r: Ray) -> Self {
        Self {
            ray: r,
            rx_origin: Point3::default(),
            ry_origin: Point3::default(),
            rx_direction: Vec3::default(),
            ry_direction: Vec3::default(),
            has_differentials: false,
        }
    }
}
