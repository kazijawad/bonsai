use crate::{
    base::{
        bxdf::{BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION},
        interaction::Interaction,
        sampler::Sampler,
        scene::Scene,
        spectrum::Spectrum,
    },
    geometries::{
        ray::{Ray, RayDifferentials},
        vec3::Vec3,
    },
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
};

pub trait Integrator: Send + Sync {
    fn render(&self, scene: &Scene);
}

pub trait SamplerIntegrator: Send + Sync + Integrator {
    fn radiance(
        &self,
        ray: &mut Ray,
        scene: &Scene,
        sampler: &mut Box<dyn Sampler>,
        depth: u32,
    ) -> RGBSpectrum;

    fn specular_reflect(
        &self,
        ray: &Ray,
        si: &SurfaceInteraction,
        scene: &Scene,
        sampler: &mut Box<dyn Sampler>,
        depth: u32,
    ) -> RGBSpectrum {
        // Compute specular reflection direction and BSDF.
        let wo = si.wo;

        let sample = si
            .bsdf
            .as_ref()
            .expect("Failed to find BSDF inside SurfaceInteraction")
            .sample(&wo, &sampler.get_2d(), BSDF_REFLECTION | BSDF_SPECULAR);

        // Return contribution of specular reflection.
        let ns = &si.shading.n;
        if sample.pdf > 0.0 && !sample.f.is_black() && sample.wi.abs_dot_normal(ns) != 0.0 {
            // Compute ray differential for specular reflection.
            let mut ray_diff = si.spawn_ray(&sample.wi);
            if let Some(diff) = ray.differentials.as_ref() {
                let rx_origin = si.p + si.dpdx;
                let ry_origin = si.p + si.dpdy;

                // Compute differential reflected directions.
                let dndx = si.shading.dndu * si.dudx + si.shading.dndv * si.dvdx;
                let dndy = si.shading.dndu * si.dudy + si.shading.dndv * si.dvdy;

                let dwodx = -diff.rx_direction - wo;
                let dwody = -diff.ry_direction - wo;

                let ddndx = dwodx.dot_normal(ns) + wo.dot_normal(&dndx);
                let ddndy = dwody.dot_normal(ns) + wo.dot_normal(&dndy);
                let rx_direction =
                    sample.wi - dwodx + 2.0 * Vec3::from(wo.dot_normal(ns) * dndx + ddndx * ns);
                let ry_direction =
                    sample.wi - dwody + 2.0 * Vec3::from(wo.dot_normal(ns) * dndy + ddndy * ns);

                ray_diff.differentials = Some(RayDifferentials {
                    rx_origin,
                    ry_origin,
                    rx_direction,
                    ry_direction,
                })
            }

            sample.f
                * self.radiance(&mut ray_diff, scene, sampler, depth + 1)
                * sample.wi.abs_dot_normal(ns)
                / sample.pdf
        } else {
            RGBSpectrum::default()
        }
    }

    fn specular_transmit(
        &self,
        ray: &Ray,
        si: &SurfaceInteraction,
        scene: &Scene,
        sampler: &mut Box<dyn Sampler>,
        depth: u32,
    ) -> RGBSpectrum {
        let p = si.p;
        let wo = si.wo;

        let bsdf = si
            .bsdf
            .as_ref()
            .expect("Failed to find BSDF inside SurfaceInteraction");

        let sample = bsdf.sample(&wo, &sampler.get_2d(), BSDF_TRANSMISSION | BSDF_SPECULAR);

        let mut result = RGBSpectrum::default();
        let mut ns = si.shading.n;
        if sample.pdf > 0.0 && !sample.f.is_black() && sample.wi.abs_dot_normal(&ns) != 0.0 {
            // Compute ray differential for specular reflection.
            let mut ray_diff = si.spawn_ray(&sample.wi);
            if let Some(diff) = ray.differentials.as_ref() {
                let rx_origin = p + si.dpdx;
                let ry_origin = p + si.dpdy;

                let mut dndx = si.shading.dndu * si.dudx + si.shading.dndv * si.dvdx;
                let mut dndy = si.shading.dndu * si.dudy + si.shading.dndv * si.dvdy;

                let mut eta = 1.0 / bsdf.eta;
                if wo.dot_normal(&ns) < 0.0 {
                    eta = 1.0 / eta;
                    ns = -ns;
                    dndx = -dndx;
                    dndy = -dndy;
                }

                let dwodx = -diff.rx_direction - wo;
                let dwody = -diff.ry_direction - wo;

                let ddndx = dwodx.dot_normal(&ns) + wo.dot_normal(&dndx);
                let ddndy = dwody.dot_normal(&ns) + wo.dot_normal(&dndy);

                let mu = eta * wo.dot_normal(&ns) - sample.wi.abs_dot_normal(&ns);
                let dmudx = (eta
                    - (eta * eta * wo.dot_normal(&ns)) / sample.wi.abs_dot_normal(&ns))
                    * ddndx;
                let dmudy = (eta
                    - (eta * eta * wo.dot_normal(&ns)) / sample.wi.abs_dot_normal(&ns))
                    * ddndy;

                let rx_direction = sample.wi - eta * dwodx + Vec3::from(mu * dndx + dmudx * ns);
                let ry_direction = sample.wi - eta * dwody + Vec3::from(mu * dndy + dmudy * ns);

                ray_diff.differentials = Some(RayDifferentials {
                    rx_origin,
                    ry_origin,
                    rx_direction,
                    ry_direction,
                })
            }

            result = sample.f
                * self.radiance(&mut ray_diff, scene, sampler, depth + 1)
                * sample.wi.abs_dot_normal(&ns)
                / sample.pdf;
        }

        result
    }
}
