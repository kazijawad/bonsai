use crate::{base::camera::CameraSample, geometries::point2::Point2, utils::math::Float};

pub trait Sampler: Send + Sync {
    fn start_pixel(&mut self, pixel: &Point2);

    fn get_1d(&mut self) -> Float;
    fn get_2d(&mut self) -> Point2;

    fn get_camera_sample(&mut self, pixel: &Point2) -> CameraSample {
        CameraSample {
            film: pixel + &self.get_2d(),
            lens: self.get_2d(),
            time: self.get_1d(),
        }
    }

    fn request_1d_vec(&mut self, n: usize);
    fn request_2d_vec(&mut self, n: usize);

    fn get_1d_vec(&mut self, n: usize) -> Vec<Float>;
    fn get_2d_vec(&mut self, n: usize) -> Vec<Point2>;

    fn start_next_sample(&mut self) -> bool;
    fn set_sample_number(&mut self, sample_number: usize) -> bool;

    fn samples_per_pixel(&self) -> usize;
    fn current_sample_number(&self) -> usize;
}
