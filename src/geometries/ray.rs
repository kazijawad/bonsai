use crate::{
    base::transform::{AnimatedTransform, Transform},
    geometries::{point3::Point3, vec3::Vec3},
    utils::math::Float,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    pub t_max: Float,
    pub time: Float,
    pub has_differentials: bool,
    pub rx_origin: Point3,
    pub ry_origin: Point3,
    pub rx_direction: Vec3,
    pub ry_direction: Vec3,
}

impl Ray {
    pub fn new(origin: &Point3, direction: &Vec3, t_max: Float, time: Float) -> Self {
        Self {
            origin: origin.clone(),
            direction: direction.clone(),
            t_max,
            time,
            has_differentials: false,
            rx_origin: Point3::default(),
            ry_origin: Point3::default(),
            rx_direction: Vec3::default(),
            ry_direction: Vec3::default(),
        }
    }

    pub fn at(&self, t: Float) -> Point3 {
        self.origin + self.direction * t
    }

    pub fn scale_differentials(&mut self, s: Float) {
        self.rx_origin = self.origin + (self.rx_origin - self.origin) * s;
        self.ry_origin = self.origin + (self.ry_origin - self.origin) * s;
        self.rx_direction = self.direction + (self.rx_direction - self.direction) * s;
        self.ry_direction = self.direction + (self.ry_direction - self.direction) * s;
    }

    pub fn transform(&self, t: &Transform) -> Self {
        let mut origin_error = Vec3::default();
        let mut origin = t.transform_point_with_error(&self.origin, &mut origin_error);

        let direction = t.transform_vec(&self.direction);
        // Offset ray origin to edge of error bounds and compute max.
        let length_squared = direction.length_squared();
        let mut t_max = self.t_max;

        if length_squared > 0.0 {
            let dt = direction.abs().dot(&origin_error) / length_squared;
            origin += direction * dt;
            t_max -= dt;
        }

        Self::new(&origin, &direction, t_max, self.time)
    }

    pub fn transform_with_error(
        &self,
        t: &Transform,
        origin_error: &mut Vec3,
        direction_error: &mut Vec3,
    ) -> Self {
        let mut origin = t.transform_point_with_error(&self.origin, origin_error);
        let direction = t.transform_vec_with_error(&self.direction, direction_error);

        let t_max = self.t_max;
        let length_squared = direction.length_squared();

        if length_squared > 0.0 {
            let dt = direction.abs().dot(&origin_error) / length_squared;
            origin += direction * dt;
        }

        Self::new(&origin, &direction, t_max, self.time)
    }

    pub fn transform_differential(&self, t: &Transform) -> Self {
        let mut ray = self.transform(t);
        ray.has_differentials = self.has_differentials;
        ray.rx_origin = self.rx_origin.transform(t);
        ray.ry_origin = self.ry_origin.transform(t);
        ray.rx_direction = t.transform_vec(&self.rx_direction);
        ray.ry_direction = t.transform_vec(&self.ry_direction);
        ray
    }

    pub fn animated_transform(&self, at: &AnimatedTransform) -> Self {
        if !at.is_animated || self.time <= at.start_time {
            self.transform(&at.start_transform)
        } else if self.time >= at.end_time {
            self.transform(&at.end_transform)
        } else {
            let mut t = Transform::default();
            at.interpolate(self.time, &mut t);
            self.transform(&t)
        }
    }

    pub fn animated_transform_differential(&self, at: &AnimatedTransform) -> Self {
        if !at.is_animated || self.time <= at.start_time {
            self.transform_differential(&at.start_transform)
        } else if self.time >= at.end_time {
            self.transform_differential(&at.end_transform)
        } else {
            let mut t = Transform::default();
            at.interpolate(self.time, &mut t);
            self.transform_differential(&t)
        }
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
            t_max: Float::INFINITY,
            time: 0.0,
            has_differentials: false,
            rx_origin: Point3::default(),
            ry_origin: Point3::default(),
            rx_direction: Vec3::default(),
            ry_direction: Vec3::default(),
        }
    }
}
