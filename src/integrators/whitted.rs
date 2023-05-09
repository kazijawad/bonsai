use rayon::prelude::*;

use crate::{
    base::{
        bxdf::{BSDF_ALL, BSDF_REFLECTION, BSDF_SPECULAR, BSDF_TRANSMISSION},
        camera::Camera,
        constants::Float,
        film::SampledPixel,
        integrator::Integrator,
        interaction::Interaction,
        material::TransportMode,
        sampler::Sampler,
        scene::Scene,
        spectrum::Spectrum,
    },
    geometries::{point2::Point2F, ray::Ray, vec3::Vec3},
    interactions::surface::SurfaceInteraction,
    spectra::rgb::RGBSpectrum,
};

pub struct WhittedIntegrator {
    camera: Box<dyn Camera>,
    sampler: Box<dyn Sampler>,
    max_depth: u32,
}

impl<'a> WhittedIntegrator {
    pub fn new(camera: Box<dyn Camera>, sampler: Box<dyn Sampler>, max_depth: u32) -> Self {
        Self {
            camera,
            sampler,
            max_depth,
        }
    }

    fn specular_reflect(
        &self,
        sampler: &mut Box<dyn Sampler>,
        ray: &Ray,
        si: &SurfaceInteraction,
        scene: &Scene<'a>,
        depth: u32,
    ) -> RGBSpectrum {
        // Compute specular reflection direction and BSDF.
        let wo = si.base.wo;
        let (wi, f, pdf, _) = if let Some(bsdf) = si.bsdf.as_ref() {
            bsdf.sample(&wo, &sampler.get_2d(), BSDF_REFLECTION | BSDF_SPECULAR)
        } else {
            panic!("WhittedIntegrator::specular_reflect BSDF is None in SurfaceInteraction")
        };

        // Return contribution of specular reflection.
        let ns = si.shading.n;
        if pdf > 0.0 && !f.is_black() && wi.abs_dot_normal(&ns) != 0.0 {
            // Compute ray differential for specular reflection.
            let mut ray_diff = si.spawn_ray(&wi);
            if ray.has_differentials {
                ray_diff.has_differentials = true;
                ray_diff.rx_origin = si.base.p + si.dpdx;
                ray_diff.ry_origin = si.base.p + si.dpdy;

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

            f * self.radiance(sampler, &mut ray_diff, scene, depth + 1) * wi.abs_dot_normal(&ns)
                / pdf
        } else {
            RGBSpectrum::default()
        }
    }

    fn specular_transmit(
        &self,
        sampler: &mut Box<dyn Sampler>,
        ray: &Ray,
        si: &SurfaceInteraction,
        scene: &Scene<'a>,
        depth: u32,
    ) -> RGBSpectrum {
        let p = si.base.p;
        let wo = si.base.wo;
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
                * self.radiance(sampler, &mut ray_diff, scene, depth + 1)
                * wi.abs_dot_normal(&ns)
                / pdf;
        }

        result
    }
}

impl<'a> Integrator<'a> for WhittedIntegrator {
    fn radiance(
        &self,
        sampler: &mut Box<dyn Sampler>,
        ray: &mut Ray,
        scene: &Scene<'a>,
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
        let wo = si.base.wo;

        // Compute scattering functions for surface interaction.
        si.compute_scattering_functions(ray, TransportMode::Radiance, false);
        if si.bsdf.is_none() {
            return self.radiance(sampler, &mut si.spawn_ray(&ray.direction), scene, depth);
        }

        // Compute emitted light if ray hit an area light source.
        result += si.emitted_radiance(&wo);

        // Add contribution of each light source.
        for light in scene.lights.iter() {
            let (emission, wi, pdf, visibility) = light.sample_point(&si, &sampler.get_2d());
            if emission.is_black() || pdf == 0.0 {
                continue;
            }

            if let Some(bsdf) = si.bsdf.as_ref() {
                let f = bsdf.f(&wo, &wi, BSDF_ALL);
                if !f.is_black() && visibility.is_unoccluded(scene) {
                    result += f * emission * wi.abs_dot_normal(&n) / pdf;
                }
            }
        }

        if depth + 1 < self.max_depth {
            // Trace rays for specular reflection and refraction.
            result += self.specular_reflect(sampler, &ray, &si, scene, depth);
            result += self.specular_transmit(sampler, &ray, &si, scene, depth);
        }

        result
    }

    fn render(&mut self, scene: &Scene<'a>) {
        let bounds = self.camera.film().bounds;
        let width = bounds.max.x - bounds.min.x;

        for y in (bounds.min.y as usize)..(bounds.max.y as usize) {
            ((bounds.min.x as usize)..(bounds.max.x as usize)).into_par_iter().for_each(|x| {
                let pixel = Point2F::new(x as Float, y as Float);
                let mut sampled_pixel = SampledPixel::default();

                let mut sampler = self.sampler.clone();
                sampler.seed((pixel.y * width + pixel.x) as u64);

                sampler.start_pixel(&pixel);

                loop {
                    // Initialize camera sample for current sample.
                    let camera_sample = sampler.get_camera_sample(&pixel);

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
                        self.radiance(&mut sampler, &mut ray, scene, 0)
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
                        &camera_sample.film,
                        radiance,
                        ray_weight,
                    );

                    if !sampler.start_next_sample() {
                        break;
                    }
                }

                self.camera.film().merge_samples(sampled_pixel, x, y);
            });
        }

        self.camera.film().write_image();
    }
}
