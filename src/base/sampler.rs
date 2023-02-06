use crate::{base::camera::CameraSample, geometries::point2::Point2, utils::math::Float};

pub trait Sampler: Clone {
    fn seed(seed: i32, sampler: Self) -> Self;

    fn start_pixel(&self, pixel: &Point2);

    fn get_1d(&self) -> Float;
    fn get_2d(&self) -> Point2;

    fn get_camera_sample(&self, raster_point: &Point2) -> CameraSample {
        let film_point = raster_point + &self.get_2d();
        let time = self.get_1d();
        let lens_point = self.get_2d();
        CameraSample {
            film_point,
            lens_point,
            time,
        }
    }

    fn request_1d_slice(&self, n: u32);
    fn request_2d_slice(&self, n: u32);

    fn round_count(&self, n: u32) -> u32 {
        n
    }

    fn get_1d_slice(&self, n: u32) -> [Float];
    fn get_2d_slice(&self, n: u32) -> [Point2];

    fn start_next_sample(&self) -> bool;

    fn set_sample_number(&self, sample_number: u64) -> bool;
}

pub trait GlobalSampler: Sampler {
    fn get_index_for_sample(&self, sample_number: u64) -> u64;

    fn sample_dimension(&self, index: u64, dimension: i32) -> Float;
}
