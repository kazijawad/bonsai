use crate::{
    base::{camera::CameraRaySample, constants::Float},
    geometries::point2::{Point2F, Point2I},
};

pub trait Sampler: Send + Sync {
    fn seed(&self, seed: u64) -> Box<dyn Sampler>;

    fn start_pixel_sample(&mut self, p: &Point2I);

    fn get_1d(&mut self) -> Float;
    fn get_2d(&mut self) -> Point2F;

    fn get_camera_sample(&mut self, pixel: &Point2I) -> CameraRaySample {
        CameraRaySample {
            film: Point2F::from(pixel.clone()) + self.get_2d(),
            lens: self.get_2d(),
            time: self.get_1d(),
        }
    }

    fn start_next_sample(&mut self) -> bool;

    fn current_sample_index(&self) -> usize;

    fn round_count(&self, n: usize) -> usize {
        n
    }

    fn samples_per_pixel(&self) -> usize;
}
