use crate::{
    geometries::{point3::Point3, vec3::Vec3},
    medium::Medium,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    pub t_max: f32,
    pub time: f32,
    pub medium: Option<Medium>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayDifferential {
    pub origin: Point3,
    pub direction: Vec3,
    pub t_max: f32,
    pub time: f32,
    pub medium: Option<Medium>,
    pub rx_origin: Point3,
    pub ry_origin: Point3,
    pub rx_direction: Vec3,
    pub ry_direction: Vec3,
    pub has_differentials: bool,
}

impl Ray {
    pub fn new(
        origin: &Point3,
        direction: &Vec3,
        t_max: f32,
        time: f32,
        medium: Option<Medium>,
    ) -> Self {
        Self {
            origin: origin.clone(),
            direction: direction.clone(),
            t_max,
            time,
            medium,
        }
    }

    pub fn at(&self, t: f32) -> Point3 {
        self.origin + self.direction * t
    }

    pub fn is_nan(&self) -> bool {
        self.origin.is_nan() || self.direction.is_nan() || self.t_max.is_nan()
    }
}

impl RayDifferential {
    pub fn new(
        origin: &Point3,
        direction: &Vec3,
        t_max: f32,
        time: f32,
        medium: Option<Medium>,
    ) -> Self {
        Self {
            origin: origin.clone(),
            direction: direction.clone(),
            t_max,
            time,
            medium,
            rx_origin: Point3::default(),
            ry_origin: Point3::default(),
            rx_direction: Vec3::default(),
            ry_direction: Vec3::default(),
            has_differentials: false,
        }
    }

    pub fn at(&self, t: f32) -> Point3 {
        self.origin + self.direction * t
    }

    pub fn scale_differentials(&mut self, s: f32) {
        self.rx_origin = self.origin + (self.rx_origin - self.origin) * s;
        self.ry_origin = self.origin + (self.ry_origin - self.origin) * s;
        self.rx_direction = self.direction + (self.rx_direction - self.direction) * s;
        self.ry_direction = self.direction + (self.ry_direction - self.direction) * s;
    }

    pub fn is_nan(&self) -> bool {
        self.origin.is_nan() || self.direction.is_nan() || self.t_max.is_nan()
    }
}

impl Default for Ray {
    fn default() -> Self {
        Self {
            origin: Point3::default(),
            direction: Vec3::default(),
            t_max: f32::INFINITY,
            time: 0.0,
            medium: None,
        }
    }
}

impl Default for RayDifferential {
    fn default() -> Self {
        Self {
            origin: Point3::default(),
            direction: Vec3::default(),
            t_max: f32::INFINITY,
            time: 0.0,
            medium: None,
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
    fn from(r: RayDifferential) -> Self {
        Self {
            origin: r.origin,
            direction: r.direction,
            t_max: r.t_max,
            time: r.time,
            medium: r.medium,
        }
    }
}

impl From<Ray> for RayDifferential {
    fn from(r: Ray) -> Self {
        Self {
            origin: r.origin,
            direction: r.direction,
            t_max: r.t_max,
            time: r.time,
            medium: r.medium,
            rx_origin: Point3::default(),
            ry_origin: Point3::default(),
            rx_direction: Vec3::default(),
            ry_direction: Vec3::default(),
            has_differentials: false,
        }
    }
}
