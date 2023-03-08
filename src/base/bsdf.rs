use crate::{
    base::bxdf::{BxDF, BxDFType, BSDF_REFLECTION, BSDF_TRANSMISSION},
    geometries::{normal::Normal, point2::Point2, vec3::Vec3},
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
    utils::math::Float,
};

// This is a heuristic assumption that a BSDF will
// typically not need more than MAX_BXDFS. This will
// let us preallocate the vector of BxDFs.
const MAX_BXDFS: usize = 8;

#[derive(Debug, Clone)]
pub struct BSDF {
    pub eta: Float,
    ns: Normal,
    ng: Normal,
    ss: Vec3,
    ts: Vec3,
    bxdfs: Vec<Box<dyn BxDF>>,
}

impl BSDF {
    pub fn new(si: &SurfaceInteraction, eta: Float) -> Self {
        let ns = si.shading.n;
        let ss = si.shading.dpdu.normalize();

        let mut bxdfs: Vec<Box<dyn BxDF>> = vec![];
        bxdfs.reserve(MAX_BXDFS);

        Self {
            eta,
            ns,
            ng: si.base.n,
            ss,
            ts: Vec3::from(ns).cross(&ss),
            bxdfs,
        }
    }

    pub fn add(&mut self, b: Box<dyn BxDF>) {
        self.bxdfs.push(b);
    }

    pub fn num_components(&self, flags: BxDFType) -> usize {
        let mut num = 0;
        for b in self.bxdfs.iter() {
            if b.matches_flags(flags) {
                num += 1;
            }
        }
        num
    }

    pub fn world_to_local(&self, v: &Vec3) -> Vec3 {
        Vec3::new(v.dot(&self.ss), v.dot(&self.ts), v.dot(&self.ns.into()))
    }

    pub fn local_to_world(&self, v: &Vec3) -> Vec3 {
        Vec3::new(
            self.ss.x * v.x + self.ts.x * v.y + self.ns.x * v.z,
            self.ss.y * v.x + self.ts.y * v.y + self.ns.y * v.z,
            self.ss.z * v.x + self.ts.z * v.y + self.ns.z * v.z,
        )
    }

    pub fn f(&self, wo_world: &Vec3, wi_world: &Vec3, flags: BxDFType) -> RGBSpectrum {
        let wo = self.world_to_local(wo_world);
        let wi = self.world_to_local(wi_world);
        if wo.z == 0.0 {
            return RGBSpectrum::default();
        }

        let to_reflect = wi_world.dot(&self.ng.into()) * wo_world.dot(&self.ng.into()) > 0.0;
        let mut f = RGBSpectrum::default();
        for b in self.bxdfs.iter() {
            if b.matches_flags(flags)
                && ((to_reflect && b.matches_flags(BSDF_REFLECTION))
                    || (!to_reflect && b.matches_flags(BSDF_TRANSMISSION)))
            {
                f += b.f(&wo, &wi);
            }
        }

        f
    }

    pub fn rho_hd(
        &self,
        wo: &Vec3,
        num_samples: usize,
        samples: &[Point2],
        flags: BxDFType,
    ) -> RGBSpectrum {
        let wo = self.world_to_local(wo);
        let mut ret = RGBSpectrum::default();
        for b in self.bxdfs.iter() {
            if b.matches_flags(flags) {
                ret += b.rho_hd(&wo, num_samples, samples);
            }
        }
        ret
    }

    pub fn rho_hh(
        &self,
        num_samples: usize,
        u1: &[Point2],
        u2: &[Point2],
        flags: BxDFType,
    ) -> RGBSpectrum {
        let mut ret = RGBSpectrum::default();
        for b in self.bxdfs.iter() {
            if b.matches_flags(flags) {
                ret += b.rho_hh(num_samples, u1, u2);
            }
        }
        ret
    }
}
