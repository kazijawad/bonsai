use crate::{
    material::Material,
    math::{aabb::AABB, onb::OrthonormalBasis, vec3::Vec3},
    object::{HitRecord, Object},
    ray::Ray,
};

pub struct Sphere {
    center: Vec3,
    radius: f32,
    material: Box<dyn Material + Send + Sync>,
}

impl Sphere {
    pub fn new<T: Material + 'static>(center: &Vec3, radius: f32, material: T) -> Self {
        Self {
            center: center.clone(),
            radius,
            material: Box::new(material),
        }
    }

    pub fn get_sphere_uv(&self, point: &Vec3) -> (f32, f32) {
        let theta = (-point.y).acos();
        let phi = (-point.z).atan2(point.x) + std::f32::consts::PI;
        (
            phi / (2.0 * std::f32::consts::PI),
            theta / std::f32::consts::PI,
        )
    }
}

impl Object for Sphere {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let offset = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = Vec3::dot(&offset, &ray.direction);
        let c = offset.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        let position = ray.at(root);
        let outward_normal = (position - self.center) / self.radius;
        let uv = self.get_sphere_uv(&outward_normal);

        let mut hit_record = HitRecord::new(
            position,
            Vec3::zeros(),
            self.material.clone(),
            root,
            uv.0,
            uv.1,
            false,
        );
        hit_record.set_face_normal(ray, &outward_normal);

        Some(hit_record)
    }

    fn bounding_box(&self, _start_time: f32, _end_time: f32) -> Option<AABB> {
        Some(AABB::new(
            &(self.center - Vec3::new(self.radius, self.radius, self.radius)),
            &(self.center + Vec3::new(self.radius, self.radius, self.radius)),
        ))
    }

    fn pdf_value(&self, origin: &Vec3, direction: &Vec3) -> f32 {
        if let None = self.intersect(&Ray::new(origin, direction, 0.0), 0.001, f32::INFINITY) {
            return 0.0;
        }

        let cos_theta_max =
            (1.0 - self.radius * self.radius / (self.center - *origin).length_squared()).sqrt();
        let solid_angle = 2.0 * std::f32::consts::PI * (1.0 - cos_theta_max);

        1.0 / solid_angle
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        let direction = self.center - *origin;
        let distance_squared = direction.length_squared();
        let mut uvw = OrthonormalBasis::new();
        uvw.build_from_w(&direction);
        uvw.local(&Vec3::random_to_sphere(self.radius, distance_squared))
    }
}
