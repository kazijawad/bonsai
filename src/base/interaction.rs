use std::sync::Arc;

use crate::{
    base::{
        bsdf::BSDF, constants::Float, material::TransportMode, math::solve_linear_system_2x2,
        primitive::Primitive, transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
};

pub struct Shading {
    pub normal: Normal,
    pub dpdu: Vec3,
    pub dpdv: Vec3,
    pub dndu: Normal,
    pub dndv: Normal,
}

pub struct SurfaceOptions {
    pub uv: Point2F,
    pub dpdu: Vec3,
    pub dpdv: Vec3,
    pub dndu: Normal,
    pub dndv: Normal,
    pub reverse_orientation: bool,
    pub transform_swaps_handedness: bool,
}

pub struct SurfaceInteraction {
    pub uv: Point2F,
    pub dpdu: Vec3,
    pub dpdv: Vec3,
    pub dndu: Normal,
    pub dndv: Normal,
    pub shading: Shading,
    pub bsdf: Option<BSDF>,
    pub primitive: Option<Arc<dyn Primitive>>,
    pub dpdx: Vec3,
    pub dpdy: Vec3,
    pub dudx: Float,
    pub dvdx: Float,
    pub dudy: Float,
    pub dvdy: Float,
}

pub struct Interaction {
    pub point: Point3,
    pub point_error: Vec3,
    pub time: Float,
    pub direction: Vec3,
    pub normal: Normal,
    pub surface: Option<SurfaceInteraction>,
}

impl Interaction {
    pub fn new(
        point: Point3,
        point_error: Vec3,
        time: Float,
        direction: Vec3,
        normal: Option<Normal>,
        surface: Option<SurfaceOptions>,
    ) -> Self {
        if let Some(si) = surface {
            let mut normal = Normal::from(si.dpdu.cross(&si.dpdv).normalize());
            if si.reverse_orientation ^ si.transform_swaps_handedness {
                normal *= -1.0;
            }

            Self {
                point,
                point_error,
                time,
                direction,
                normal,
                surface: Some(SurfaceInteraction {
                    uv: si.uv,
                    dpdu: si.dpdu,
                    dpdv: si.dpdv,
                    dndu: si.dndu,
                    dndv: si.dndv,
                    // Initialize shading geometry from true geometry.
                    shading: Shading {
                        normal,
                        dpdu: si.dpdu,
                        dpdv: si.dpdv,
                        dndu: si.dndu,
                        dndv: si.dndv,
                    },
                    bsdf: None,
                    primitive: None,
                    dpdx: Vec3::default(),
                    dpdy: Vec3::default(),
                    dudx: 0.0,
                    dvdx: 0.0,
                    dudy: 0.0,
                    dvdy: 0.0,
                }),
            }
        } else {
            let normal = normal.unwrap();

            Self {
                point,
                point_error,
                time,
                direction,
                normal,
                surface: None,
            }
        }
    }

    pub fn spawn_ray(&self, direction: &Vec3) -> Ray {
        let origin = self
            .point
            .offset_ray_origin(&self.point_error, &self.normal, direction);

        Ray::new(&origin, direction, 1.0 - 0.0001, self.time)
    }

    pub fn spawn_ray_to_point(&self, p: Point3) -> Ray {
        let direction = p - self.point;
        let origin = self
            .point
            .offset_ray_origin(&self.point_error, &self.normal, &direction);

        Ray::new(&origin, &direction, 1.0 - 0.0001, self.time)
    }

    pub fn spawn_ray_to_it(&self, it: &Self) -> Ray {
        let origin =
            self.point
                .offset_ray_origin(&self.point_error, &self.normal, &(it.point - self.point));
        let target = it
            .point
            .offset_ray_origin(&it.point_error, &it.normal, &(origin - it.point));

        let direction = target - origin;

        Ray::new(&origin, &direction, 1.0 - 0.0001, self.time)
    }

    pub fn set_shading_geometry(
        &mut self,
        dpdus: &Vec3,
        dpdvs: &Vec3,
        dndus: &Normal,
        dndvs: &Normal,
        orientation_is_authoritative: bool,
    ) {
        let si = self.surface.as_mut().unwrap();

        // Compute shading normal.
        si.shading.normal = Normal::from(dpdus.cross(dpdvs)).normalize();
        if orientation_is_authoritative {
            self.normal = self.normal.face_forward(&si.shading.normal);
        } else {
            si.shading.normal = si.shading.normal.face_forward(&self.normal);
        }

        // Initialize shading partial derivatives.
        si.shading.dpdu.clone_from(dpdus);
        si.shading.dpdv.clone_from(dpdvs);
        si.shading.dndu.clone_from(dndus);
        si.shading.dndv.clone_from(dndvs);
    }

    pub fn compute_scattering_functions(
        &mut self,
        ray: &Ray,
        mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        self.compute_differentials(ray);

        let si = self.surface.as_mut().unwrap();
        if let Some(primitive) = si.primitive.clone() {
            primitive.compute_scattering_functions(self, mode, allow_multiple_lobes);
        }
    }

    pub fn compute_differentials(&mut self, ray: &Ray) {
        let si = self.surface.as_mut().unwrap();

        let mut fail = || {
            si.dudx = 0.0;
            si.dvdx = 0.0;
            si.dudy = 0.0;
            si.dvdy = 0.0;
            si.dpdx = Vec3::default();
            si.dpdy = Vec3::default();
        };

        if ray.differentials.is_none() {
            return fail();
        }

        let diff = ray.differentials.as_ref().unwrap();

        // Compute auxiliary intersection points with plane.
        let d = self.normal.dot_point(&self.point);

        let tx = -(self.normal.dot_point(&diff.rx_origin) - d)
            / self.normal.dot(&Normal::from(diff.rx_direction));
        if tx.is_infinite() || tx.is_nan() {
            return fail();
        }
        let px = diff.rx_origin + tx * diff.rx_direction;

        let ty = -(self.normal.dot_point(&diff.ry_origin) - d)
            / self.normal.dot(&Normal::from(diff.ry_direction));
        if ty.is_infinite() || ty.is_nan() {
            return fail();
        }
        let py = diff.ry_origin + ty * diff.ry_direction;

        si.dpdx = px - self.point;
        si.dpdy = py - self.point;

        // Choose two dimensions to use for ray offset computation.
        let dim = if self.normal.x.abs() > self.normal.y.abs()
            && self.normal.x.abs() > self.normal.z.abs()
        {
            [1, 2]
        } else if self.normal.y.abs() > self.normal.z.abs() {
            [0, 2]
        } else {
            [0, 1]
        };

        // Initialize matrices for offset computation.
        let a = [
            [si.dpdu[dim[0]], si.dpdv[dim[0]]],
            [si.dpdu[dim[1]], si.dpdv[dim[1]]],
        ];
        let bx = [
            px[dim[0]] - self.point[dim[0]],
            px[dim[1]] - self.point[dim[1]],
        ];
        let by = [
            py[dim[0]] - self.point[dim[0]],
            py[dim[1]] - self.point[dim[1]],
        ];

        if !solve_linear_system_2x2(a, bx, &mut si.dudx, &mut si.dvdx) {
            si.dudx = 0.0;
            si.dvdx = 0.0;
        }
        if !solve_linear_system_2x2(a, by, &mut si.dudy, &mut si.dvdy) {
            si.dudy = 0.0;
            si.dvdy = 0.0;
        }
    }

    pub fn emitted_radiance(&self, direction: &Vec3) -> RGBSpectrum {
        let so = self.surface.as_ref().unwrap();
        if let Some(primitive) = so.primitive.as_ref() {
            if let Some(area_light) = primitive.area_light() {
                return area_light.emission(self, direction);
            }
        }

        RGBSpectrum::default()
    }

    pub fn transform(&mut self, t: &Transform) {
        let mut abs_error = Vec3::default();
        self.point = self
            .point
            .transform_with_point_error(t, &self.point_error, &mut abs_error);
        self.point_error = abs_error;

        self.direction = self.direction.transform(t);
        self.normal = self.normal.transform(t).normalize();

        if let Some(surface) = self.surface.as_mut() {
            surface.dpdu = surface.dpdu.transform(t);
            surface.dpdv = surface.dpdv.transform(t);
            surface.dndu = surface.dndu.transform(t);
            surface.dndv = surface.dndv.transform(t);

            surface.shading = Shading {
                normal: surface
                    .shading
                    .normal
                    .transform(t)
                    .normalize()
                    .face_forward(&self.normal),
                dpdu: surface.shading.dpdu.transform(t),
                dpdv: surface.shading.dpdv.transform(t),
                dndu: surface.shading.dndu.transform(t),
                dndv: surface.shading.dndv.transform(t),
            };

            surface.dpdx = surface.dpdx.transform(t);
            surface.dpdy = surface.dpdy.transform(t);
        }
    }
}

impl Default for Interaction {
    fn default() -> Self {
        Self {
            point: Point3::default(),
            point_error: Vec3::default(),
            time: 0.0,
            direction: Vec3::default(),
            normal: Normal::default(),
            surface: None,
        }
    }
}
