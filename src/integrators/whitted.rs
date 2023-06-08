use crate::{
    base::{
        bxdf::BSDF_ALL, camera::Camera, integrator::SamplerIntegrator, interaction::Interaction,
        material::TransportMode, sampler::Sampler, scene::Scene, spectrum::Spectrum,
    },
    geometries::{ray::Ray, vec3::Vec3},
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
        let mut it = Interaction::default();
        if !scene.intersect(ray, &mut it) {
            for light in scene.lights.iter() {
                output += light.radiance(ray);
            }
            return output;
        }

        // Compute scattering functions for surface interaction.
        it.compute_scattering_functions(ray, TransportMode::Radiance, false);

        let si = it.surface.as_ref().unwrap();
        if si.bsdf.is_none() {
            return self.radiance(&mut it.spawn_ray(&ray.direction), scene, sampler, depth);
        }

        // Compute emitted light if ray hit an area light source.
        output += it.emitted_radiance(&it.direction);

        // Add contribution of each light source.
        let normal = Vec3::from(si.shading.normal);
        for light in scene.lights.iter() {
            let sample = light.sample_point(&it, &sampler.get_2d());
            if sample.radiance.is_black() || sample.pdf == 0.0 {
                continue;
            }

            if let Some(bsdf) = si.bsdf.as_ref() {
                let f = bsdf.f(&it.direction, &sample.wi, BSDF_ALL);
                if let Some(visibility) = sample.visibility {
                    if !f.is_black() && visibility.is_unoccluded(scene) {
                        output += f * sample.radiance * sample.wi.abs_dot(&normal) / sample.pdf;
                    }
                }
            }
        }

        if depth + 1 < self.max_depth {
            // Trace rays for specular reflection and refraction.
            output += self.specular_reflect(&ray, &it, scene, sampler, depth);
            output += self.specular_transmit(&ray, &it, scene, sampler, depth);
        }

        output
    }
}
