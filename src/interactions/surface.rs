use std::sync::Arc;

use crate::{
    base::{
        bsdf::BSDF, constants::Float, interaction::Interaction, material::TransportMode,
        primitive::Primitive, transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    spectra::rgb::RGBSpectrum,
    utils::math::solve_linear_system_2x2,
};

#[derive(Clone)]
pub struct Shading {
    pub n: Normal,
    pub dpdu: Vec3,
    pub dpdv: Vec3,
    pub dndu: Normal,
    pub dndv: Normal,
}

#[derive(Clone)]
pub struct SurfaceInteraction {
    pub base: BaseInteraction,
    pub uv: Point2,
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

impl SurfaceInteraction {
    pub fn new(
        p: Point3,
        p_error: Vec3,
        uv: Point2,
        wo: Vec3,
        dpdu: Vec3,
        dpdv: Vec3,
        dndu: Normal,
        dndv: Normal,
        time: Float,
        reverse_orientation: bool,
        transform_swaps_handedness: bool,
    ) -> Self {
        let mut n = Normal::from(dpdu.cross(&dpdv).normalize());
        if reverse_orientation ^ transform_swaps_handedness {
            n *= -1.0;
        }

        Self {
            base: BaseInteraction {
                p,
                p_error,
                time,
                wo: wo.normalize(),
                n,
            },
            uv,
            dpdu,
            dpdv,
            dndu,
            dndv,
            // Initialize shading geometry from true geometry.
            shading: Shading {
                n,
                dpdu,
                dpdv,
                dndu,
                dndv,
            },
            bsdf: None,
            primitive: None,
            dpdx: Vec3::default(),
            dpdy: Vec3::default(),
            dudx: 0.0,
            dvdx: 0.0,
            dudy: 0.0,
            dvdy: 0.0,
        }
    }

    pub fn set_shading_geometry(
        &mut self,
        dpdus: &Vec3,
        dpdvs: &Vec3,
        dndus: &Normal,
        dndvs: &Normal,
        orientation_is_authoritative: bool,
    ) {
        // Compute shading normal.
        self.shading.n = Normal::from(dpdus.cross(dpdvs)).normalize();
        if orientation_is_authoritative {
            self.base.n = self.base.n.face_forward(&self.shading.n);
        } else {
            self.shading.n = self.shading.n.face_forward(&self.base.n);
        }

        // Initialize shading partial derivatives.
        self.shading.dpdu.clone_from(dpdus);
        self.shading.dpdv.clone_from(dpdvs);
        self.shading.dndu.clone_from(dndus);
        self.shading.dndv.clone_from(dndvs);
    }

    pub fn compute_scattering_functions(
        &mut self,
        ray: &Ray,
        mode: TransportMode,
        allow_multiple_lobes: bool,
    ) {
        self.compute_differentials(ray);
        self.primitive
            .as_ref()
            .unwrap()
            .clone()
            .compute_scattering_functions(self, mode, allow_multiple_lobes);
    }

    pub fn compute_differentials(&mut self, ray: &Ray) {
        let mut fail = || {
            self.dudx = 0.0;
            self.dvdx = 0.0;
            self.dudy = 0.0;
            self.dvdy = 0.0;
            self.dpdx = Vec3::default();
            self.dpdy = Vec3::default();
        };

        if ray.has_differentials {
            // Compute auxiliary intersection points with plane.
            let d = self.base.n.dot(&self.base.p.into());

            let tx = -(self.base.n.dot(&ray.rx_origin.into()) - d)
                / self.base.n.dot(&ray.rx_direction.into());
            if tx.is_infinite() || tx.is_nan() {
                fail();
                return;
            }
            let px = ray.rx_origin + tx * ray.rx_direction;

            let ty = -(self.base.n.dot(&ray.ry_origin.into()) - d)
                / self.base.n.dot(&ray.ry_direction.into());
            if ty.is_infinite() || ty.is_nan() {
                fail();
                return;
            }
            let py = ray.ry_origin + ty * ray.ry_direction;

            self.dpdx = px - self.base.p;
            self.dpdy = py - self.base.p;

            // Choose two dimensions to use for ray offset computation.
            let dim = if self.base.n.x.abs() > self.base.n.y.abs()
                && self.base.n.x.abs() > self.base.n.z.abs()
            {
                [1, 2]
            } else if self.base.n.y.abs() > self.base.n.z.abs() {
                [0, 2]
            } else {
                [0, 1]
            };

            // Initialize matrices for offset computation.
            let a = [
                [self.dpdu[dim[0]], self.dpdv[dim[0]]],
                [self.dpdu[dim[1]], self.dpdv[dim[1]]],
            ];
            let bx = [
                px[dim[0]] - self.base.p[dim[0]],
                px[dim[1]] - self.base.p[dim[1]],
            ];
            let by = [
                py[dim[0]] - self.base.p[dim[0]],
                py[dim[1]] - self.base.p[dim[1]],
            ];

            if !solve_linear_system_2x2(a, bx, &mut self.dudx, &mut self.dvdx) {
                self.dudx = 0.0;
                self.dvdx = 0.0;
            }
            if !solve_linear_system_2x2(a, by, &mut self.dudy, &mut self.dvdy) {
                self.dudy = 0.0;
                self.dvdy = 0.0;
            }
        } else {
            fail();
        }
    }

    pub fn le(&self, w: &Vec3) -> RGBSpectrum {
        RGBSpectrum::default()
    }

    pub fn transform(&self, t: &Transform) -> Self {
        let mut p_error = Vec3::default();
        let p = self
            .base
            .p
            .transform_with_point_error(t, &self.base.p_error, &mut p_error);

        let time = self.base.time;
        let wo = t.transform_vec(&self.base.wo);
        let n = t.transform_normal(&self.base.n).normalize();
        let uv = self.uv;
        let dpdu = t.transform_vec(&self.dpdu);
        let dpdv = t.transform_vec(&self.dpdv);
        let dndu = t.transform_normal(&self.dndu);
        let dndv = t.transform_normal(&self.dndv);
        let shading = Shading {
            n: t.transform_normal(&self.shading.n)
                .normalize()
                .face_forward(&n),
            dpdu: t.transform_vec(&self.shading.dpdu),
            dpdv: t.transform_vec(&self.shading.dpdv),
            dndu: t.transform_normal(&self.shading.dndu),
            dndv: t.transform_normal(&self.shading.dndv),
        };
        let bsdf = self.bsdf.clone();
        let primitive = self.primitive.clone();
        let dudx = self.dudx;
        let dvdx = self.dvdx;
        let dudy = self.dudy;
        let dvdy = self.dvdy;
        let dpdx = t.transform_vec(&self.dpdx);
        let dpdy = t.transform_vec(&self.dpdy);

        Self {
            base: BaseInteraction {
                p,
                p_error,
                time,
                wo,
                n,
            },
            uv,
            dpdu,
            dpdv,
            dndu,
            dndv,
            shading,
            bsdf,
            primitive,
            dpdx,
            dpdy,
            dudx,
            dvdx,
            dudy,
            dvdy,
        }
    }
}

impl Interaction for SurfaceInteraction {
    fn position(&self) -> Point3 {
        self.base.p
    }

    fn position_error(&self) -> Vec3 {
        self.base.p_error
    }

    fn normal(&self) -> Normal {
        self.base.n
    }

    fn time(&self) -> Float {
        self.base.time
    }

    fn spawn_ray(&self, direction: &Vec3) -> Ray {
        let origin = self
            .base
            .p
            .offset_ray_origin(&self.base.p_error, &self.base.n, direction);
        Ray::new(&origin, direction, Float::INFINITY, self.base.time)
    }

    fn spawn_ray_to_point(&self, point: Point3) -> Ray {
        let origin =
            self.base
                .p
                .offset_ray_origin(&self.base.p_error, &self.base.n, &(point - self.base.p));
        let direction = point - self.base.p;
        Ray::new(&origin, &direction, 1.0 - 0.0001, self.base.time)
    }

    fn spawn_ray_to_it(&self, it: &dyn Interaction) -> Ray {
        let origin = self.base.p.offset_ray_origin(
            &self.base.p_error,
            &self.base.n,
            &(it.position() - self.base.p),
        );
        let target = it.position().offset_ray_origin(
            &it.position_error(),
            &it.normal(),
            &(origin - it.position()),
        );
        let direction = target - origin;
        Ray::new(&origin, &direction, 1.0 - 0.0001, self.base.time)
    }
}

impl Default for SurfaceInteraction {
    fn default() -> Self {
        Self {
            base: BaseInteraction {
                p: Point3::default(),
                p_error: Vec3::default(),
                time: 0.0,
                wo: Vec3::default(),
                n: Normal::default(),
            },
            uv: Point2::default(),
            dpdu: Vec3::default(),
            dpdv: Vec3::default(),
            dndu: Normal::default(),
            dndv: Normal::default(),
            shading: Shading {
                n: Normal::default(),
                dpdu: Vec3::default(),
                dpdv: Vec3::default(),
                dndu: Normal::default(),
                dndv: Normal::default(),
            },
            bsdf: None,
            primitive: None,
            dpdx: Vec3::default(),
            dpdy: Vec3::default(),
            dudx: 0.0,
            dvdx: 0.0,
            dudy: 0.0,
            dvdy: 0.0,
        }
    }
}
