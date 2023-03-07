use crate::{
    base::{
        camera::Camera, integrator::Integrator, sampler::Sampler, scene::Scene, spectrum::Spectrum,
    },
    geometries::{bounds2::Bounds2, point2::Point2, ray::RayDifferential},
    utils::math::Float,
};

pub trait SamplerIntegrated: Send + Sync {
    fn preprocess(scene: &Scene, sampler: &dyn Sampler) {}

    fn li(ray: &RayDifferential, scene: &Scene, sampler: &dyn Sampler, depth: i32) -> Spectrum;
}

pub struct SamplerIntegrator<'a> {
    camera: &'a dyn Camera,
    sampler: &'a dyn Sampler,
    pixel_bounds: Bounds2,
}

impl<'a> SamplerIntegrator<'a> {
    pub fn new(camera: &'a dyn Camera, sampler: &'a dyn Sampler, pixel_bounds: Bounds2) -> Self {
        Self {
            camera,
            sampler,
            pixel_bounds,
        }
    }
}

impl<'a> Integrator for SamplerIntegrator<'a> {
    fn render(&self, scene: &Scene) {
        // Compute number of tiles to use for parallel rendering.
        let sample_bounds = self.camera.get_film().sample_bounds();
        let sample_extent = sample_bounds.diagonal();

        let tile_size = 16;
        let x_tiles = (sample_extent.x as i32 + tile_size - 1) / tile_size;
        let y_tiles = (sample_extent.y as i32 + tile_size - 1) / tile_size;

        for x in 0..x_tiles {
            for y in 0..y_tiles {
                // Compute sample bounds for tile.
                let x0 = sample_bounds.min.x as i32 + x * tile_size;
                let x1 = (x0 + tile_size).min(sample_bounds.max.x as i32);
                let y0 = sample_bounds.min.y as i32 + y * tile_size;
                let y1 = (y0 + tile_size).min(sample_bounds.max.y as i32);

                let tile_bounds = Bounds2::new(
                    &Point2::new(x0 as Float, y0 as Float),
                    &Point2::new(x1 as Float, y1 as Float),
                );

                println!("{:?}", tile_bounds);
            }
        }

        // Save final image after rendering.
        self.camera.get_film().write_image(1.0);
    }
}
