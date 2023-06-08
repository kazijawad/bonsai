use std::ptr;

use crate::{
    base::{
        bxdf::{BxDF, BxDFType, BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION},
        constants::{Float, ONE_MINUS_EPSILON},
        interaction::Interaction,
    },
    geometries::{normal::Normal, point2::Point2F, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
};

// A heuristic to preallocate the bxdfs parameters.
const BSDF_CAPACITY: usize = 8;

pub struct BSDF {
    pub eta: Float,
    shading_normal: Normal,
    geometric_normal: Normal,
    s_shading: Vec3,
    t_shading: Vec3,
    bxdfs: Vec<Box<dyn BxDF>>,
}

pub struct BSDFSample {
    pub wi: Vec3,
    pub f: RGBSpectrum,
    pub pdf: Float,
    pub sampled_type: BxDFType,
}

impl BSDF {
    pub fn new(it: &Interaction, eta: Float) -> Self {
        let si = it.surface.as_ref().unwrap();

        let shading_normal = si.shading.normal;
        let geometric_normal = it.normal;

        let s_shading = si.shading.dpdu.normalize();
        let t_shading = Vec3::from(shading_normal).cross(&s_shading);

        let bxdfs = Vec::with_capacity(BSDF_CAPACITY);

        Self {
            eta,
            shading_normal,
            geometric_normal,
            s_shading,
            t_shading,
            bxdfs,
        }
    }

    pub fn add(&mut self, bxdf: Box<dyn BxDF>) {
        self.bxdfs.push(bxdf);
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
        Vec3::new(
            v.dot(&self.s_shading),
            v.dot(&self.t_shading),
            v.dot(&Vec3::from(self.shading_normal)),
        )
    }

    pub fn local_to_world(&self, v: &Vec3) -> Vec3 {
        Vec3::new(
            self.s_shading.x * v.x + self.t_shading.x * v.y + self.shading_normal.x * v.z,
            self.s_shading.y * v.x + self.t_shading.y * v.y + self.shading_normal.y * v.z,
            self.s_shading.z * v.x + self.t_shading.z * v.y + self.shading_normal.z * v.z,
        )
    }

    pub fn f(&self, wo_world: &Vec3, wi_world: &Vec3, flags: BxDFType) -> RGBSpectrum {
        let wo = self.world_to_local(wo_world);
        let wi = self.world_to_local(wi_world);
        if wo.z == 0.0 {
            return RGBSpectrum::default();
        }

        let normal = Vec3::from(self.geometric_normal);
        let reflect = wi_world.dot(&normal) * wo_world.dot(&normal) > 0.0;

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
        samples: &[Point2F],
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
        u1: &[Point2F],
        u2: &[Point2F],
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

    pub fn sample(&self, wo_world: &Vec3, u: &Point2F, bxdf_type: BxDFType) -> BSDFSample {
        let matches = self.num_components(bxdf_type);
        if matches == 0 {
            return BSDFSample {
                wi: Vec3::default(),
                f: RGBSpectrum::default(),
                pdf: 0.0,
                sampled_type: 0,
            };
        }

        let component = ((u[0] * matches as Float).floor() as u32).min(matches as u32 - 1);

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
        let u_remapped = Point2F::new(
            (u[0] * matches as Float - component as Float).min(ONE_MINUS_EPSILON),
            u[1],
        );

        // Sample chosen BxDF.
        let wo = self.world_to_local(&wo_world);
        if wo.z == 0.0 {
            return BSDFSample {
                wi: Vec3::default(),
                f: RGBSpectrum::default(),
                pdf: 0.0,
                sampled_type: 0,
            };
        }

        let mut sample = bxdf.sample(&wo, &u_remapped);
        let sampled_type = if let Some(bxdf_type) = sample.sampled_type {
            bxdf_type
        } else {
            bxdf.bxdf_type()
        };

        if sample.pdf == 0.0 {
            return BSDFSample {
                wi: Vec3::default(),
                f: RGBSpectrum::default(),
                pdf: 0.0,
                sampled_type: 0,
            };
        }
        let wi_world = self.local_to_world(&sample.wi);

        // Compute overall PDF with all matching BxDFs.
        if bxdf.bxdf_type() & BSDF_SPECULAR == 0 && matches > 1 {
            for b in self.bxdfs.iter() {
                if !ptr::eq(b, bxdf) && b.matches_flags(bxdf_type) {
                    sample.pdf += b.pdf(&wo, &sample.wi);
                }
            }
        }
        if matches > 1 {
            sample.pdf /= matches as Float;
        }

        // Compute value of BSDF for sampled direction.
        if bxdf.bxdf_type() & BSDF_SPECULAR == 0 {
            let normal = Vec3::from(self.geometric_normal);
            let reflect = wi_world.dot(&normal) * wo_world.dot(&normal) > 0.0;

            sample.f = RGBSpectrum::default();
            for b in self.bxdfs.iter() {
                if b.matches_flags(bxdf_type)
                    && ((reflect && b.bxdf_type() & BSDF_REFLECTION != 0)
                        || (!reflect && b.bxdf_type() & BSDF_TRANSMISSION != 0))
                {
                    sample.f += b.f(&wo, &sample.wi);
                }
            }
        }

        BSDFSample {
            wi: wi_world,
            f: sample.f,
            pdf: sample.pdf,
            sampled_type,
        }
    }

    pub fn pdf(&self, wo_world: &Vec3, wi_world: &Vec3, flags: BxDFType) -> Float {
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
