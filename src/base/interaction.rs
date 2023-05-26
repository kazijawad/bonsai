use crate::{
    base::constants::Float,
    geometries::{normal::Normal, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::surface::SurfaceInteraction,
};

pub trait Interaction<'a>: Send + Sync {
    fn p(&self) -> Point3;

    fn p_error(&self) -> Vec3;

    fn time(&self) -> Float;

    fn wo(&self) -> Vec3;

    fn n(&self) -> Normal;

    fn spawn_ray(&self, dir: &Vec3) -> Ray {
        let origin = self.p().offset_ray_origin(&self.p_error(), &self.n(), dir);
        Ray::new(&origin, dir, 1.0 - 0.0001, self.time())
    }

    fn spawn_ray_to_point(&self, p: Point3) -> Ray {
        let direction = p - self.p();
        let origin = self
            .p()
            .offset_ray_origin(&self.p_error(), &self.n(), &direction);
        Ray::new(&origin, &direction, 1.0 - 0.0001, self.time())
    }

    fn spawn_ray_to_it(&self, it: &dyn Interaction) -> Ray {
        let origin = self
            .p()
            .offset_ray_origin(&self.p_error(), &self.n(), &(it.p() - self.p()));
        let target = it
            .p()
            .offset_ray_origin(&it.p_error(), &it.n(), &(origin - it.p()));
        let direction = target - origin;
        Ray::new(&origin, &direction, 1.0 - 0.0001, self.time())
    }

    fn surface_interaction(&self) -> Option<&SurfaceInteraction<'a>> {
        None
    }
}
