use crate::{
    base::{
        bxdf::BSDF_ALL, camera::Camera, integrator::SamplerIntegrator, interaction::Interaction,
        material::TransportMode, sampler::Sampler, scene::Scene, spectrum::Spectrum,
    },
    geometries::ray::Ray,
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
};

pub struct WhittedIntegrator {
    camera: Box<dyn Camera>,
    sampler: Box<dyn Sampler>,
    max_depth: u32,
}

impl WhittedIntegrator {
    pub fn new(camera: Box<dyn Camera>, sampler: Box<dyn Sampler>, max_depth: u32) -> Self {
        Self {
            camera,
            sampler,
            max_depth,
        }
    }
}

impl SamplerIntegrator for WhittedIntegrator {
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
    ) -> RGBSpectrum {
        let mut output = RGBSpectrum::default();

        // Find closest ray intersection or return background radiance.
        let mut si = SurfaceInteraction::default();
        if !scene.intersect(ray, &mut si) {
            for light in scene.lights.iter() {
                output += light.radiance(ray);
            }
            return output;
        }

        // Initialize common variables for integrator.
        let n = si.shading.n;
        let wo = si.wo;

        // Compute scattering functions for surface interaction.
        si.compute_scattering_functions(ray, TransportMode::Radiance, false);
        if si.bsdf.is_none() {
            return self.radiance(&mut si.spawn_ray(&ray.direction), scene, sampler, depth);
        }

        // Compute emitted light if ray hit an area light source.
        output += si.emitted_radiance(&wo);

        // Add contribution of each light source.
        for light in scene.lights.iter() {
            let sample = light.sample_point(&si, &sampler.get_2d());
            if sample.radiance.is_black() || sample.pdf == 0.0 {
                continue;
            }

            if let Some(bsdf) = si.bsdf.as_ref() {
                let f = bsdf.f(&wo, &sample.wi, BSDF_ALL);
                if let Some(visibility) = sample.visibility {
                    if !f.is_black() && visibility.is_unoccluded(scene) {
                        output += f * sample.radiance * sample.wi.abs_dot_normal(&n) / sample.pdf;
                    }
                }
            }
        }

        if depth + 1 < self.max_depth {
            // Trace rays for specular reflection and refraction.
            output += self.specular_reflect(&ray, &si, scene, sampler, depth);
            output += self.specular_transmit(&ray, &si, scene, sampler, depth);
        }

        output
    }
}
