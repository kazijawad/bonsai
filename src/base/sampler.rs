use rand::prelude::*;

use crate::{base::camera::CameraSample, geometries::point2::Point2, utils::math::Float};

pub trait Sampler {
    fn start_pixel(&mut self, pixel: &Point2);

    fn get_float(&mut self) -> Float;
    fn get_point(&mut self) -> Point2;

    fn get_camera_sample(&mut self, raster_point: &Point2) -> CameraSample {
        let film_point = raster_point + &self.get_point();
        let time = self.get_float();
        let lens_point = self.get_point();
        CameraSample {
            film_point,
            lens_point,
            time,
        }
    }

    fn request_float_batch(&mut self, n: usize);
    fn request_point_batch(&mut self, n: usize);

    fn round_count(&self, n: usize) -> usize {
        n
    }

    fn get_float_batch(&mut self, n: usize) -> Vec<Float>;
    fn get_point_batch(&mut self, n: usize) -> Vec<Point2>;

    fn start_next_sample(&mut self) -> bool;
    fn set_sample_number(&mut self, sample_number: usize) -> bool;

    fn shuffle<T>(sample: &mut [T], count: usize, num_dims: usize, rng: &mut ThreadRng) {
        for i in 0..count {
            let other = i + rng.gen_range(0..(count - i));
            for j in 0..num_dims {
                sample.swap(num_dims * i + j, num_dims * other + j);
            }
        }
    }
}
