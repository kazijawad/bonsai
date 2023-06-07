use rayon::prelude::*;

use crate::{
    base::{
        bxdf::{BSDF_ALL, BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION},
        camera::Camera,
        constants::Float,
        interaction::Interaction,
        light::{is_delta_light, Light},
        sampler::Sampler,
        sampling::power_heuristic,
        scene::Scene,
        spectrum::Spectrum,
    },
    geometries::{
        bounds2::Bounds2I,
        point2::{Point2F, Point2I},
        ray::{Ray, RayDifferentials},
        vec3::Vec3,
    },
    spectra::rgb::RGBSpectrum,
};

const TILE_SIZE: i32 = 16;

pub trait Integrator: Send + Sync {
    fn render(&self, scene: &Scene);
}

pub trait SamplerIntegrator: Send + Sync {
    fn camera(&self) -> &dyn Camera;

    fn sampler(&self) -> &dyn Sampler;

    fn render(&self, scene: &Scene) {
        // Compute number of tiles to use for parallel rendering.
        let sample_bounds = self.camera().film().sample_bounds();
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
                    let seed = (tile.y * num_tiles.x + tile.x) as u64;
                    let mut sampler = self.sampler().seed(seed);

                    // Compute sample bounds for tile.
                    let x0 = sample_bounds.min.x + tile.x * TILE_SIZE;
                    let x1 = (x0 + TILE_SIZE).min(sample_bounds.max.x);

                    let y0 = sample_bounds.min.y + tile.y * TILE_SIZE;
                    let y1 = (y0 + TILE_SIZE).min(sample_bounds.max.y);

                    let tile_bounds = Bounds2I::new(&Point2I::new(x0, y0), &Point2I::new(x1, y1));

                    let mut film_tile = self.camera().film().get_film_tile(&tile_bounds);

                    tile_bounds.traverse(|pixel| {
                        sampler.start_pixel_sample(&pixel);

                        if !self.camera().film().cropped_pixel_bounds.inside_exclusive(&pixel) {
                            return;
                        }

                        loop {
                            let camera_sample = sampler.get_camera_sample(&pixel);

                            // Generate camera ray for current sample.
                            let mut ray = Ray::default();
                            let ray_weight = self.camera()
                                .generate_ray(&camera_sample, &mut ray);
                            ray.scale_differentials(
                                1.0 / (sampler.samples_per_pixel() as Float).sqrt(),
                            );

                            // Evaluate radiance along camera ray.
                            let mut radiance = if ray_weight > 0.0 {
                                self.radiance(&mut ray, scene, sampler.as_mut(), 0)
                            } else {
                                RGBSpectrum::default()
                            };

                            // Issue warning if unexpected radiance value returned.
                            if radiance.is_nan() {
                                eprintln!(
                                    "NaN radiance value returned for pixel ({:?}, {:?}), sample {:?}. Setting to black.",
                                    pixel.x,
                                    pixel.y,
                                    sampler.current_sample_index()
                                );
                                radiance = RGBSpectrum::default();
                            } else if radiance.y() < -1e-5 {
                                eprintln!(
                                    "Negative luminance value, {:?}, returned for pixel ({:?}, {:?}), sample {:?}, Setting to black.",
                                    radiance.y(),
                                    pixel.x,
                                    pixel.y,
                                    sampler.current_sample_index()
                                );
                                radiance = RGBSpectrum::default();
                            } else if radiance.y().is_infinite() {
                                eprintln!(
                                    "Infinite luminance returned for pixel ({:?}, {:?}), sample {:?}, Setting to black.",
                                    pixel.x,
                                    pixel.y,
                                    sampler.current_sample_index()
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

                    self.camera().film().merge_film_tile(film_tile);
                }
            });

        self.camera().film().write_image(1.0);
    }

    fn radiance(
        &self,
        ray: &mut Ray,
        scene: &Scene,
        sampler: &mut dyn Sampler,
        depth: u32,
    ) -> RGBSpectrum;

    fn specular_reflect(
        &self,
        ray: &Ray,
        it: &Interaction,
        scene: &Scene,
        sampler: &mut dyn Sampler,
        depth: u32,
    ) -> RGBSpectrum {
        // Compute specular reflection direction and BSDF.
        let wo = it.direction;
        let si = it.surface.as_ref().unwrap();

        let sample = si
            .bsdf
            .as_ref()
            .expect("Failed to find BSDF inside SurfaceInteraction")
            .sample(&wo, &sampler.get_2d(), BSDF_REFLECTION | BSDF_SPECULAR);

        // Return contribution of specular reflection.
        let ns = &si.shading.normal;
        if sample.pdf > 0.0 && !sample.f.is_black() && sample.wi.abs_dot_normal(ns) != 0.0 {
            // Compute ray differential for specular reflection.
            let mut ray_diff = it.spawn_ray(&sample.wi);
            if let Some(diff) = ray.differentials.as_ref() {
                let rx_origin = it.point + si.dpdx;
                let ry_origin = it.point + si.dpdy;

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
        it: &Interaction,
        scene: &Scene,
        sampler: &mut dyn Sampler,
        depth: u32,
    ) -> RGBSpectrum {
        let p = it.point;
        let wo = it.direction;
        let si = it.surface.as_ref().unwrap();

        let bsdf = si
            .bsdf
            .as_ref()
            .expect("Failed to find BSDF inside SurfaceInteraction");

        let sample = bsdf.sample(&wo, &sampler.get_2d(), BSDF_TRANSMISSION | BSDF_SPECULAR);

        let mut result = RGBSpectrum::default();
        let mut ns = si.shading.normal;
        if sample.pdf > 0.0 && !sample.f.is_black() && sample.wi.abs_dot_normal(&ns) != 0.0 {
            // Compute ray differential for specular reflection.
            let mut ray_diff = it.spawn_ray(&sample.wi);
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

pub fn uniform_sample_all_lights(
    it: &Interaction,
    scene: &Scene,
    sampler: &mut dyn Sampler,
    light_sample_counts: &[usize],
) -> RGBSpectrum {
    let mut output = RGBSpectrum::default();

    for i in 0..scene.lights.len() {
        let light = &scene.lights[i];
        let sample_count = light_sample_counts[i];

        let u_light_batch = (0..sample_count)
            .map(|_| sampler.get_2d())
            .collect::<Vec<Point2F>>();
        let u_scattering_batch = (0..sample_count)
            .map(|_| sampler.get_2d())
            .collect::<Vec<Point2F>>();

        let mut direct_output = RGBSpectrum::default();
        for j in 0..sample_count {
            direct_output += estimate_direct(
                it,
                scene,
                light.as_ref(),
                &u_scattering_batch[j],
                &u_light_batch[j],
            );
        }
        output += direct_output / sample_count as Float;
    }

    output
}

pub fn uniform_sample_one_light(
    it: &Interaction,
    scene: &Scene,
    sampler: &mut dyn Sampler,
) -> RGBSpectrum {
    if scene.lights.is_empty() {
        return RGBSpectrum::default();
    }

    // Randomly choose a single light to sample from.
    let light_count = scene.lights.len() as Float;
    let light_index = (sampler.get_1d() * light_count).min(light_count - 1.0) as usize;
    let light_pdf = 1.0 / light_count;

    let light = &scene.lights[light_index];
    let u_light = sampler.get_2d();
    let u_scattering = sampler.get_2d();

    estimate_direct(it, scene, light.as_ref(), &u_scattering, &u_light) / light_pdf
}

fn estimate_direct(
    it: &Interaction,
    scene: &Scene,
    light: &dyn Light,
    u_scattering: &Point2F,
    u_light: &Point2F,
) -> RGBSpectrum {
    let bsdf_flags = BSDF_ALL & !BSDF_SPECULAR;
    let mut output = RGBSpectrum::default();

    // Sample light source with multiple importance sampling.
    let mut light_sample = light.sample_point(it, u_light);
    let mut scattering_pdf = 0.0;
    if light_sample.pdf > 0.0 && !light_sample.radiance.is_black() {
        let mut f = RGBSpectrum::default();

        if let Some(si) = &it.surface {
            // Evaluate BSDF for light sampling strategy.
            let bsdf = si
                .bsdf
                .as_ref()
                .expect("Failed to find BSDF inside SurfaceInteraction");
            f = bsdf.f(&it.direction, &light_sample.wi, bsdf_flags)
                * light_sample.wi.abs_dot_normal(&si.shading.normal);
            scattering_pdf = bsdf.pdf(&it.direction, &light_sample.wi, bsdf_flags);
        }

        if !f.is_black() {
            // Compute effect of visibility for light sample.
            if !light_sample
                .visibility
                .expect("Failed to find VisibilityTester on LightPointSample")
                .is_unoccluded(scene)
            {
                light_sample.radiance = RGBSpectrum::default();
            }

            // Add light's contribution to reflected radiance.
            if !light_sample.radiance.is_black() {
                if is_delta_light(light.flag()) {
                    output += f * light_sample.radiance / light_sample.pdf;
                } else {
                    let weight = power_heuristic(1.0, light_sample.pdf, 1.0, scattering_pdf);
                    output += f * light_sample.radiance * weight / light_sample.pdf;
                }
            }
        }
    }

    // Sample BSDF with multiple importance sampling.
    if !is_delta_light(light.flag()) {
        let mut f = RGBSpectrum::default();
        let mut sampled_specular = false;

        if let Some(si) = &it.surface {
            // Sample scattered direction for surface interactions.
            let bsdf_sample = si
                .bsdf
                .as_ref()
                .expect("Failed to find BSDF inside SurfaceInteraction")
                .sample(&it.direction, u_scattering, bsdf_flags);

            f = bsdf_sample.f * bsdf_sample.wi.abs_dot_normal(&si.shading.normal);
            sampled_specular = (bsdf_sample.sampled_type & BSDF_SPECULAR) != 0;
        }

        if !f.is_black() && scattering_pdf > 0.0 {
            // Account for light contributions along sampled direction.
            let mut weight = 1.0;
            if !sampled_specular {
                light_sample.pdf = light.point_pdf(it, &light_sample.wi);
                if light_sample.pdf == 0.0 {
                    return output;
                }

                weight = power_heuristic(1.0, scattering_pdf, 1.0, light_sample.pdf);
            }

            // Find intersection.
            let mut light_it = Interaction::default();
            let mut ray = it.spawn_ray(&light_sample.wi);
            let surface_intersection = scene.intersect(&mut ray, &mut light_it);

            // Add light contribution from material sampling.
            let mut radiance = RGBSpectrum::default();
            if surface_intersection {
                let si = light_it.surface.as_ref().unwrap();

                let primitive = si
                    .primitive
                    .as_ref()
                    .expect("Failed to find primitive on SurfaceInteraction");

                if primitive.area_light().is_some() {
                    radiance = light_it.emitted_radiance(&-light_sample.wi);
                }
            } else {
                radiance = light.radiance(&ray);
            }

            if !radiance.is_black() {
                output += f * radiance * weight / scattering_pdf;
            }
        }
    }

    output
}
