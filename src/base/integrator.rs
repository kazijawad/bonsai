use crate::{
    base::{
        bxdf::{BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION},
        interaction::Interaction,
        sampler::Sampler,
        scene::Scene,
        spectrum::Spectrum,
    },
    geometries::{ray::Ray, vec3::Vec3},
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
        let (wi, f, pdf, _) = if let Some(bsdf) = si.bsdf.as_ref() {
            bsdf.sample(&wo, &sampler.get_2d(), BSDF_REFLECTION | BSDF_SPECULAR)
        } else {
            panic!("SamplerIntegrator::specular_reflect BSDF is None in SurfaceInteraction")
        };

        // Return contribution of specular reflection.
        let ns = si.shading.n;
        if pdf > 0.0 && !f.is_black() && wi.abs_dot_normal(&ns) != 0.0 {
            // Compute ray differential for specular reflection.
            let mut ray_diff = si.spawn_ray(&wi);
            if ray.has_differentials {
                ray_diff.has_differentials = true;
                ray_diff.rx_origin = si.p + si.dpdx;
                ray_diff.ry_origin = si.p + si.dpdy;

                // Compute differential reflected directions.
                let dndx = si.shading.dndu * si.dudx + si.shading.dndv * si.dvdx;
                let dndy = si.shading.dndu * si.dudy + si.shading.dndv * si.dvdy;

                let dwodx = -ray.rx_direction - wo;
                let dwody = -ray.ry_direction - wo;

                let ddndx = dwodx.dot_normal(&ns) + wo.dot_normal(&dndx);
                let ddndy = dwody.dot_normal(&ns) + wo.dot_normal(&dndy);
                ray_diff.rx_direction =
                    wi - dwodx + 2.0 * Vec3::from(wo.dot_normal(&ns) * dndx + ddndx * ns);
                ray_diff.ry_direction =
                    wi - dwody + 2.0 * Vec3::from(wo.dot_normal(&ns) * dndy + ddndy * ns);
            }

            f * self.radiance(&mut ray_diff, scene, sampler, depth + 1) * wi.abs_dot_normal(&ns)
                / pdf
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
        let bsdf = si.bsdf.as_ref().unwrap();

        let (wi, f, pdf, _) =
            bsdf.sample(&wo, &sampler.get_2d(), BSDF_TRANSMISSION | BSDF_SPECULAR);

        let mut result = RGBSpectrum::default();
        let mut ns = si.shading.n;
        if pdf > 0.0 && !f.is_black() && wi.abs_dot_normal(&ns) != 0.0 {
            // Compute ray differential for specular reflection.
            let mut ray_diff = si.spawn_ray(&wi);
            if ray.has_differentials {
                ray_diff.has_differentials = true;

                ray_diff.rx_origin = p + si.dpdx;
                ray_diff.ry_origin = p + si.dpdy;

                let mut dndx = si.shading.dndu * si.dudx + si.shading.dndv * si.dvdx;
                let mut dndy = si.shading.dndu * si.dudy + si.shading.dndv * si.dvdy;

                let mut eta = 1.0 / bsdf.eta;
                if wo.dot_normal(&ns) < 0.0 {
                    eta = 1.0 / eta;
                    ns = -ns;
                    dndx = -dndx;
                    dndy = -dndy;
                }

                let dwodx = -ray.rx_direction - wo;
                let dwody = -ray.ry_direction - wo;

                let ddndx = dwodx.dot_normal(&ns) + wo.dot_normal(&dndx);
                let ddndy = dwody.dot_normal(&ns) + wo.dot_normal(&dndy);

                let mu = eta * wo.dot_normal(&ns) - wi.abs_dot_normal(&ns);
                let dmudx =
                    (eta - (eta * eta * wo.dot_normal(&ns)) / wi.abs_dot_normal(&ns)) * ddndx;
                let dmudy =
                    (eta - (eta * eta * wo.dot_normal(&ns)) / wi.abs_dot_normal(&ns)) * ddndy;

                ray_diff.rx_direction = wi - eta * dwodx + Vec3::from(mu * dndx + dmudx * ns);
                ray_diff.ry_direction = wi - eta * dwody + Vec3::from(mu * dndy + dmudy * ns);
            }

            result = f
                * self.radiance(&mut ray_diff, scene, sampler, depth + 1)
                * wi.abs_dot_normal(&ns)
                / pdf;
        }

        result
    }
}
