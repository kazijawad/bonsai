use crate::{
    base::{
        bxdf::BSDF_ALL, camera::Camera, film::SampledPixel, integrator::Integrator,
        interaction::Interaction, material::TransportMode, primitive::Primitive, sampler::Sampler,
        scene::Scene, spectrum::Spectrum,
    },
    geometries::{point2::Point2, ray::Ray, vec3::Vec3},
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
    utils::math::Float,
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

    fn specular_reflect(
        &self,
        ray: &Ray,
        si: &SurfaceInteraction,
        scene: &Scene,
        depth: u32,
    ) -> RGBSpectrum {
        todo!()
    }

    fn specular_transmit(
        &self,
        ray: &Ray,
        si: &SurfaceInteraction,
        scene: &Scene,
        depth: u32,
    ) -> RGBSpectrum {
        todo!()
    }
}

impl Integrator for WhittedIntegrator {
    fn preprocess(&self, scene: &Scene) {}

    fn li(&mut self, ray: &mut Ray, scene: &Scene, depth: u32) -> RGBSpectrum {
        let mut radiance = RGBSpectrum::default();

        // Find closest ray intersection or return background radiance.
        let mut si = SurfaceInteraction::default();
        if !scene.intersect(ray, &mut si) {
            for light in scene.lights.iter() {
                radiance += light.le(ray);
            }
            return radiance;
        }

        // Initialize common variables for integrator.
        let n = si.shading.n;
        let wo = si.base.wo;

        // Compute scattering functions for surface interaction.
        si.compute_scattering_functions(ray, scene, TransportMode::Radiance, false);
        if si.bsdf.is_none() {
            return self.li(&mut si.spawn_ray(&ray.direction), scene, depth);
        }

        // Compute emitted light if ray hit an area light source.
        radiance += si.le(&wo);

        // Add contribution of each light source.
        for light in scene.lights.iter() {
            let mut wi = Vec3::default();
            let mut pdf = 0.0;

            let (li, visibility) = light.sample_li(&si, &self.sampler.get_2d(), &mut wi, &mut pdf);
            if li.is_black() || pdf == 0.0 {
                continue;
            }

            let f = si.bsdf.as_ref().unwrap().f(&wo, &wi, BSDF_ALL);
            if !f.is_black() && visibility.is_unoccluded(scene) {
                radiance += f * li * wi.abs_dot(&Vec3::from(n)) / pdf;
            }
        }

        if depth + 1 < self.max_depth {
            // Trace rays for specular reflection and refraction.
            radiance += self.specular_reflect(&ray, &si, scene, depth);
            radiance += self.specular_transmit(&ray, &si, scene, depth);
        }

        radiance
    }

    fn render(&mut self, scene: &Scene) {
        let bounds = self.camera.film().bounds;

        for y in (bounds.min.y as usize)..(bounds.max.y as usize) {
            for x in (bounds.min.x as usize)..(bounds.max.x as usize) {
                let pixel = Point2::new(x as Float, y as Float);
                let mut sampled_pixel = SampledPixel::default();

                self.sampler.start_pixel(&pixel);

                loop {
                    // Initialize camera sample for current sample.
                    let camera_sample = self.sampler.get_camera_sample(&pixel);

                    // Generate camera ray for current sample.
                    let mut ray = Ray::default();
                    let ray_weight = self
                        .camera
                        .generate_ray_differential(&camera_sample, &mut ray);
                    ray.scale_differentials(
                        1.0 / (self.sampler.samples_per_pixel() as Float).sqrt(),
                    );

                    // Evaluate radiance along camera ray.
                    let mut radiance = if ray_weight > 0.0 {
                        self.li(&mut ray, scene, 0)
                    } else {
                        RGBSpectrum::default()
                    };

                    // Issue warning if unexpected radiance value returned.
                    if radiance.is_nan() {
                        eprintln!(
                            "NaN radiance value returned for pixel ({:?}, {:?}), sample {:?}. Setting to black.",
                            pixel.x,
                            pixel.y,
                            self.sampler.current_sample_number()
                        );
                        radiance = RGBSpectrum::new(1.0);
                    } else if radiance.y() < -1e-5 {
                        eprintln!(
                            "Negative luminance value, {:?}, returned for pixel ({:?}, {:?}), sample {:?}, Setting to black.",
                            radiance.y(),
                            pixel.x,
                            pixel.y,
                            self.sampler.current_sample_number()
                        );
                        radiance = RGBSpectrum::new(1.0);
                    } else if radiance.y().is_infinite() {
                        eprintln!(
                            "Infinite luminance returned for pixel ({:?}, {:?}), sample {:?}, Setting to black.",
                            pixel.x,
                            pixel.y,
                            self.sampler.current_sample_number()
                        );
                        radiance = RGBSpectrum::new(1.0);
                    }

                    // Add camera ray's contribution to image.
                    self.camera.film().add_sample(
                        &mut sampled_pixel,
                        &camera_sample.film_point,
                        radiance,
                        ray_weight,
                    );

                    if !self.sampler.start_next_sample() {
                        break;
                    }
                }

                self.camera.film().merge_samples(sampled_pixel, x, y);
            }
        }

        self.camera.film().write_image();
    }
}
