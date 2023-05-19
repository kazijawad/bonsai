use crate::{
    base::{
        constants::Float,
        transform::{AnimatedTransform, Transform},
    },
    geometries::{point3::Point3, vec3::Vec3},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    pub t_max: Float,
    pub time: Float,
    pub differentials: Option<RayDifferentials>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayDifferentials {
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
            differentials: None,
        }
    }

    pub fn at(&self, t: Float) -> Point3 {
        self.origin + self.direction * t
    }

    pub fn scale_differentials(&mut self, scale: Float) {
        if let Some(diff) = self.differentials.as_mut() {
            diff.rx_origin = self.origin + (diff.rx_origin - self.origin) * scale;
            diff.ry_origin = self.origin + (diff.ry_origin - self.origin) * scale;
            diff.rx_direction = self.direction + (diff.rx_direction - self.direction) * scale;
            diff.ry_direction = self.direction + (diff.ry_direction - self.direction) * scale;
        }
    }

    pub fn transform(&self, t: &Transform) -> Self {
        let mut origin_error = Vec3::default();
        let mut origin = self.origin.transform_with_error(t, &mut origin_error);

        let direction = self.direction.transform(t);
        // Offset ray origin to edge of error bounds and compute max.
        let length_squared = direction.length_squared();
        let mut t_max = self.t_max;

        if length_squared > 0.0 {
            let dt = direction.abs().dot(&origin_error) / length_squared;
            origin += direction * dt;
            t_max -= dt;
        }

        let mut ray = Self::new(&origin, &direction, t_max, self.time);
        if let Some(diff) = self.differentials.as_ref() {
            ray.differentials = Some(RayDifferentials {
                rx_origin: diff.rx_origin.transform(t),
                ry_origin: diff.ry_origin.transform(t),
                rx_direction: diff.rx_direction.transform(t),
                ry_direction: diff.ry_direction.transform(t),
            });
        }

        ray
    }

    pub fn transform_with_error(
        &self,
        t: &Transform,
        o_error: &mut Vec3,
        d_error: &mut Vec3,
    ) -> Self {
        let mut origin = self.origin.transform_with_error(t, o_error);
        let direction = self.direction.transform_with_error(t, d_error);

        let t_max = self.t_max;
        let length_squared = direction.length_squared();

        if length_squared > 0.0 {
            let dt = direction.abs().dot(&o_error) / length_squared;
            origin += direction * dt;
        }

        let mut ray = Self::new(&origin, &direction, t_max, self.time);
        if let Some(diff) = self.differentials.as_ref() {
            ray.differentials = Some(RayDifferentials {
                rx_origin: diff.rx_origin.transform(t),
                ry_origin: diff.ry_origin.transform(t),
                rx_direction: diff.rx_direction.transform(t),
                ry_direction: diff.ry_direction.transform(t),
            });
        }

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
            differentials: None,
        }
    }
}
