use crate::{
    base::{
        camera::Camera,
        integrator::{uniform_sample_all_lights, uniform_sample_one_light, SamplerIntegrator},
        interaction::Interaction,
        material::TransportMode,
        sampler::Sampler,
        scene::Scene,
    },
    geometries::ray::Ray,
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
};

pub enum LightStrategy {
    UniformSampleAll,
    UniformSampleOne,
}

pub struct DirectLightingIntegrator {
    camera: Box<dyn Camera>,
    sampler: Box<dyn Sampler>,
    max_depth: u32,
    strategy: LightStrategy,
    light_sample_counts: Vec<usize>,
}

impl DirectLightingIntegrator {
    pub fn new(
        camera: Box<dyn Camera>,
        sampler: Box<dyn Sampler>,
        scene: &Scene,
        max_depth: u32,
        strategy: LightStrategy,
    ) -> Self {
        // Compute number of samples to use for each light.
        let light_sample_counts = if let LightStrategy::UniformSampleAll = strategy {
            scene
                .lights
                .iter()
                .map(|light| sampler.round_count(light.num_samples()))
                .collect()
        } else {
            Vec::new()
        };

        Self {
            camera,
            sampler,
            max_depth,
            strategy,
            light_sample_counts,
        }
    }
}

impl SamplerIntegrator for DirectLightingIntegrator {
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
        depth: u32,
    ) -> crate::RGBSpectrum {
        let mut output = RGBSpectrum::default();

        // Find closest ray intersection or return background radiance.
        let mut si = SurfaceInteraction::default();
        if !scene.intersect(ray, &mut si) {
            for light in scene.lights.iter() {
                output += light.radiance(&ray)
            }
            return output;
        }

        // Compute scattering functions for surface interaction.
        si.compute_scattering_functions(ray, TransportMode::Radiance, false);
        if si.bsdf.is_none() {
            return self.radiance(&mut si.spawn_ray(&ray.direction), scene, sampler, depth);
        }

        // Compute emitted radiance from area light intersection.
        output += si.emitted_radiance(&si.wo);

        // Compute direct lighting based on sampling strategy.
        if !scene.lights.is_empty() {
            if let LightStrategy::UniformSampleAll = self.strategy {
                output +=
                    uniform_sample_all_lights(&si, &scene, sampler, &self.light_sample_counts);
            } else {
                output += uniform_sample_one_light(&si, &scene, sampler, None);
            }
        }

        if depth + 1 < self.max_depth {
            // Trace rays for specular reflection and refraction.
            output += self.specular_reflect(ray, &si, scene, sampler, depth);
            output += self.specular_transmit(ray, &si, scene, sampler, depth);
        }

        output
    }
}
