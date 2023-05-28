use std::debug_assert;

use crate::{
    base::{
        bxdf::{BSDF_ALL, BSDF_SPECULAR, BSDF_TRANSMISSION},
        camera::Camera,
        constants::Float,
        integrator::{uniform_sample_one_light, SamplerIntegrator},
        interaction::Interaction,
        material::TransportMode,
        sampler::Sampler,
        scene::Scene,
        spectrum::Spectrum,
    },
    geometries::ray::Ray,
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
};

pub struct PathIntegrator {
    camera: Box<dyn Camera>,
    sampler: Box<dyn Sampler>,
    max_depth: u32,
    rr_threshold: Float,
}

impl PathIntegrator {
    pub fn new(
        camera: Box<dyn Camera>,
        sampler: Box<dyn Sampler>,
        max_depth: u32,
        rr_threshold: Float,
    ) -> Self {
        Self {
            camera,
            sampler,
            max_depth,
            rr_threshold,
        }
    }
}

impl SamplerIntegrator for PathIntegrator {
    fn camera(&self) -> &dyn Camera {
        self.camera.as_ref()
    }

    fn sampler(&self) -> &dyn Sampler {
        self.sampler.as_ref()
    }

    fn radiance(
        &self,
        ray: &mut Ray,
        scene: &Scene,
        sampler: &mut dyn Sampler,
        _: u32,
    ) -> RGBSpectrum {
        let mut output = RGBSpectrum::default();
        let mut beta = RGBSpectrum::new(1.0);

        let mut ray = ray.clone();
        let mut specular_bounce = false;

        // Tracks the accumulated effect of radiance scaling due
        // to rays passing through refractive boundaries.
        let mut eta_scale = 1.0;

        let mut bounces = 0;
        // Find next path vertex and accumulate contribution.
        loop {
            // Intersect ray with scene.
            let mut si = SurfaceInteraction::default();
            let si_intersection = scene.intersect(&mut ray, &mut si);

            // Add intersection emission if it is the first intersection
            // from camera ray or the prior path segment included a
            // specular BSDF component.
            if bounces == 0 || specular_bounce {
                if si_intersection {
                    output += beta * si.emitted_radiance(&-ray.direction);
                } else {
                    for light in scene.lights.iter() {
                        output += beta * light.radiance(&ray);
                    }
                }
            }

            // Terminate path if there was no intersection or
            // max_depth is reached.
            if !si_intersection || bounces >= self.max_depth {
                break;
            }

            // Compute scattering functions and skip over medium boundaries.
            si.compute_scattering_functions(&ray, TransportMode::Radiance, true);
            if si.bsdf.is_none() {
                ray = si.spawn_ray(&ray.direction);
                bounces -= 1;
                continue;
            }

            // Sample illumination from lights to find path contribution,
            // ignoring specular BSDFs.
            let bsdf = si.bsdf.as_ref().unwrap();
            if bsdf.num_components(BSDF_ALL & !BSDF_SPECULAR) > 0 {
                output += beta * uniform_sample_one_light(&si, scene, sampler);
            }

            // Sample BSDF to get new path direction.
            let wo = -ray.direction;
            let bsdf_sample = bsdf.sample(&wo, &sampler.get_2d(), BSDF_ALL);
            if bsdf_sample.f.is_black() || bsdf_sample.pdf == 0.0 {
                break;
            }
            beta *= bsdf_sample.f * bsdf_sample.wi.abs_dot_normal(&si.shading.n) / bsdf_sample.pdf;
            specular_bounce = (bsdf_sample.sampled_type & BSDF_SPECULAR) != 0;
            if (bsdf_sample.sampled_type & BSDF_SPECULAR) != 0
                && (bsdf_sample.sampled_type & BSDF_TRANSMISSION) != 0
            {
                let eta = bsdf.eta;
                eta_scale *= if wo.dot_normal(&si.n) > 0.0 {
                    eta * eta
                } else {
                    1.0 / (eta * eta)
                };
            }

            // Terminate path with russian roulette.
            let rr_beta = beta * eta_scale;
            if rr_beta.max_component_value() < self.rr_threshold && bounces > 3 {
                let q = (1.0 - rr_beta.max_component_value()).max(0.5);
                if sampler.get_1d() < q {
                    break;
                }
                beta /= 1.0 - q;
                debug_assert!(beta.y().is_finite());
            }

            bounces += 1;
        }

        output
    }
}
