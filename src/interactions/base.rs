use crate::{
    base::interaction::Interaction,
    geometries::{normal::Normal, point3::Point3, ray::Ray, vec3::Vec3},
    utils::math::Float,
};

#[derive(Debug, Clone)]
pub struct BaseInteraction {
    pub p: Point3,
    pub p_error: Vec3,
    pub time: Float,
    pub wo: Vec3,
    pub n: Normal,
}

impl BaseInteraction {
    pub fn new(p: &Point3, time: Float) -> Self {
        Self {
            p: p.clone(),
            p_error: Vec3::default(),
            time,
            wo: Vec3::default(),
            n: Normal::default(),
        }
    }
}

impl Interaction for BaseInteraction {
    fn position(&self) -> Point3 {
        self.p
    }

    fn position_error(&self) -> Vec3 {
        self.p_error
    }

    fn normal(&self) -> Normal {
        self.n
    }

    fn time(&self) -> Float {
        self.time
    }

    fn spawn_ray(&self, direction: &Vec3) -> Ray {
        let origin = self.p.offset_ray_origin(&self.p_error, &self.n, direction);
        Ray::new(&origin, direction, 1.0 - 0.0001, self.time)
    }

    fn spawn_ray_to_point(&self, point: Point3) -> Ray {
        let direction = point - self.p;
        let origin = self.p.offset_ray_origin(&self.p_error, &self.n, &direction);
        Ray::new(&origin, &direction, 1.0 - 0.0001, self.time)
    }

    fn spawn_ray_to_it(&self, it: &dyn Interaction) -> Ray {
        let origin = self
            .p
            .offset_ray_origin(&self.p_error, &self.n, &(it.position() - self.p));
        let target = it.position().offset_ray_origin(
            &it.position_error(),
            &it.normal(),
            &(origin - it.position()),
        );
        let direction = target - origin;
        Ray::new(&origin, &direction, 1.0 - 0.0001, self.time)
    }
}
