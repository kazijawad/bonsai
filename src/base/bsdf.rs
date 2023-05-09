use std::ptr;

use crate::{
    base::{
        bxdf::{BxDF, BxDFType, BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION},
        constants::Float,
    },
    geometries::{normal::Normal, point2::Point2, vec3::Vec3},
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
};

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

        Self {
            eta,
            ns,
            ng: si.base.n,
            ss,
            ts: Vec3::from(ns).cross(&ss),
            bxdfs: vec![],
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

        let reflect = wi_world.dot_normal(&self.ng) * wo_world.dot_normal(&self.ng) > 0.0;
        let mut f = RGBSpectrum::default();
        for bxdf in self.bxdfs.iter() {
            if bxdf.matches_flags(flags)
                && ((reflect && bxdf.bxdf_type() & BSDF_REFLECTION != 0)
                    || (!reflect && bxdf.bxdf_type() & BSDF_TRANSMISSION != 0))
            {
                f += bxdf.f(&wo, &wi);
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
        for bxdf in self.bxdfs.iter() {
            if bxdf.matches_flags(flags) {
                ret += bxdf.rho_hd(&wo, num_samples, samples);
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
        for bxdf in self.bxdfs.iter() {
            if bxdf.matches_flags(flags) {
                ret += bxdf.rho_hh(num_samples, u1, u2);
            }
        }
        ret
    }

    pub fn sample(
        &self,
        wo_world: &Vec3,
        u: &Point2,
        bxdf_type: BxDFType,
    ) -> (Vec3, RGBSpectrum, Float, BxDFType) {
        let matching_components = self.num_components(bxdf_type);
        if matching_components == 0 {
            return (Vec3::default(), RGBSpectrum::default(), 0.0, 0);
        }

        let component = ((u.x * matching_components as Float).floor() as u32)
            .min(matching_components as u32 - 1);

        let mut count = component;
        let mut bxdf = None;
        for b in self.bxdfs.iter() {
            if b.matches_flags(bxdf_type) && count == 0 {
                bxdf = Some(b);
                break;
            }
            count -= 1;
        }
        let bxdf = bxdf.expect("BSDF::sample BxDF was not initialized");

        // Remap BxDF sample to [0, 1].
        let u_remapped = Point2::new(
            (u[0] * matching_components as Float - component as Float).min(1.0 - Float::EPSILON),
            u[1],
        );

        // Sample chosen BxDF.
        let wo = self.world_to_local(&wo_world);
        if wo.z == 0.0 {
            return (Vec3::default(), RGBSpectrum::default(), 0.0, 0);
        }

        let (wi, mut f, mut pdf, bxdf_sampled_type) = bxdf.sample(&wo, &u_remapped);
        let sampled_type = if let Some(bxdf_type) = bxdf_sampled_type {
            bxdf_type
        } else {
            bxdf.bxdf_type()
        };

        if pdf == 0.0 {
            return (Vec3::default(), RGBSpectrum::default(), 0.0, 0);
        }
        let wi_world = self.local_to_world(&wi);

        // Compute overall PDF with all matching BxDFs.
        if bxdf.bxdf_type() & BSDF_SPECULAR == 0 && matching_components > 1 {
            for b in self.bxdfs.iter() {
                if !ptr::eq(b, bxdf) && b.matches_flags(bxdf_type) {
                    pdf += b.pdf(&wo, &wi);
                }
            }
        }
        if matching_components > 1 {
            pdf /= matching_components as Float;
        }

        // Compute value of BSDF for sampled direction.
        if bxdf.bxdf_type() & BSDF_SPECULAR == 0 {
            let reflect =
                wi_world.dot(&Vec3::from(self.ng)) * wo_world.dot(&Vec3::from(self.ng)) > 0.0;
            f = RGBSpectrum::default();
            for b in self.bxdfs.iter() {
                if b.matches_flags(bxdf_type)
                    && ((reflect && b.bxdf_type() & BSDF_REFLECTION != 0)
                        || (!reflect && b.bxdf_type() & BSDF_TRANSMISSION != 0))
                {
                    f += b.f(&wo, &wi);
                }
            }
        }

        (wi_world, f, pdf, sampled_type)
    }

    pub fn pdf(self, wo_world: &Vec3, wi_world: &Vec3, flags: BxDFType) -> Float {
        if self.bxdfs.len() == 0 {
            return 0.0;
        }

        let wo = self.world_to_local(wo_world);
        let wi = self.world_to_local(wi_world);
        if wo.z == 0.0 {
            return 0.0;
        }

        let mut ret = 0.0;
        let mut components = 0.0;
        for bxdf in self.bxdfs.iter() {
            if bxdf.matches_flags(flags) {
                components += 1.0;
                ret += bxdf.pdf(&wo, &wi);
            }
        }

        if components > 0.0 {
            ret / components
        } else {
            0.0
        }
    }
}
