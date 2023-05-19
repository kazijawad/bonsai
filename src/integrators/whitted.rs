use rayon::prelude::*;

use crate::{
    base::{
        bxdf::BSDF_ALL,
        camera::Camera,
        constants::Float,
        integrator::{Integrator, SamplerIntegrator},
        interaction::Interaction,
        material::TransportMode,
        sampler::Sampler,
        scene::Scene,
        spectrum::Spectrum,
    },
    geometries::{bounds2::Bounds2I, point2::Point2I, ray::Ray},
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
};

const TILE_SIZE: i32 = 16;

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

impl Integrator for WhittedIntegrator {
    fn render(&self, scene: &Scene) {
        // Compute number of tiles to use for parallel rendering.
        let sample_bounds = self.camera.film().sample_bounds();
        let sample_extent = sample_bounds.diagonal();
        let num_tiles = Point2I::new(
            (sample_extent.x + TILE_SIZE - 1) / TILE_SIZE,
            (sample_extent.y + TILE_SIZE - 1) / TILE_SIZE,
        );

        (0..num_tiles.y)
            .collect::<Vec<i32>>()
            .par_iter()
            .for_each(|y| {
                for x in 0..num_tiles.x {
                    let tile = Point2I::new(x, *y);

                    // Get sampler instance for tile.
                    let seed = tile.y * num_tiles.x + tile.x;
                    let mut sampler = self.sampler.seed(seed);

                    // Compute sample bounds for tile.
                    let x0 = sample_bounds.min.x + tile.x * TILE_SIZE;
                    let x1 = (x0 + TILE_SIZE).min(sample_bounds.max.x);

                    let y0 = sample_bounds.min.y + tile.y * TILE_SIZE;
                    let y1 = (y0 + TILE_SIZE).min(sample_bounds.max.y);

                    let tile_bounds = Bounds2I::new(&Point2I::new(x0, y0), &Point2I::new(x1, y1));

                    let mut film_tile = self.camera.film().get_film_tile(&tile_bounds);

                    tile_bounds.traverse(|pixel| {
                        sampler.start_pixel(&pixel);

                        if !self.camera.film().cropped_pixel_bounds.inside_exclusive(&pixel) {
                            return;
                        }

                        loop {
                            let camera_sample = sampler.get_camera_sample(&pixel);

                            // Generate camera ray for current sample.
                            let mut ray = Ray::default();
                            let ray_weight = self.camera
                                .generate_ray(&camera_sample, &mut ray);
                            ray.scale_differentials(
                                1.0 / (sampler.samples_per_pixel() as Float).sqrt(),
                            );

                            // Evaluate radiance along camera ray.
                            let mut radiance = if ray_weight > 0.0 {
                                self.radiance(&mut ray, scene, &mut sampler, 0)
                            } else {
                                RGBSpectrum::default()
                            };

                            // Issue warning if unexpected radiance value returned.
                            if radiance.is_nan() {
                                eprintln!(
                                    "NaN radiance value returned for pixel ({:?}, {:?}), sample {:?}. Setting to black.",
                                    pixel.x,
                                    pixel.y,
                                    sampler.current_sample_number()
                                );
                                radiance = RGBSpectrum::default();
                            } else if radiance.y() < -1e-5 {
                                eprintln!(
                                    "Negative luminance value, {:?}, returned for pixel ({:?}, {:?}), sample {:?}, Setting to black.",
                                    radiance.y(),
                                    pixel.x,
                                    pixel.y,
                                    sampler.current_sample_number()
                                );
                                radiance = RGBSpectrum::default();
                            } else if radiance.y().is_infinite() {
                                eprintln!(
                                    "Infinite luminance returned for pixel ({:?}, {:?}), sample {:?}, Setting to black.",
                                    pixel.x,
                                    pixel.y,
                                    sampler.current_sample_number()
                                );
                                radiance = RGBSpectrum::default();
                            }

                            // Add camera ray's contribution to image.
                            film_tile.add_sample(camera_sample.film, radiance, ray_weight);

                            if !sampler.start_next_sample() {
                                break;
                            }
                        }
                    });

                    self.camera.film().merge_film_tile(film_tile);
                }
            });

        self.camera.film().write_image(1.0);
    }
}

impl SamplerIntegrator for WhittedIntegrator {
    fn radiance(
        &self,
        ray: &mut Ray,
        scene: &Scene,
        sampler: &mut Box<dyn Sampler>,
        depth: u32,
    ) -> RGBSpectrum {
        let mut result = RGBSpectrum::default();

        // Find closest ray intersection or return background radiance.
        let mut si = SurfaceInteraction::default();
        if !scene.intersect(ray, &mut si) {
            for light in scene.lights.iter() {
                result += light.radiance(ray);
            }
            return result;
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
        result += si.emitted_radiance(&wo);

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
                        result += f * sample.radiance * sample.wi.abs_dot_normal(&n) / sample.pdf;
                    }
                }
            }
        }

        if depth + 1 < self.max_depth {
            // Trace rays for specular reflection and refraction.
            result += self.specular_reflect(&ray, &si, scene, sampler, depth);
            result += self.specular_transmit(&ray, &si, scene, sampler, depth);
        }

        result
    }
}
