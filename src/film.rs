use std::fs;

use image::{imageops, ImageBuffer, Rgb, RgbImage};

use crate::geometries::vec3::Vec3;

pub struct Film {
    width: u32,
    height: u32,
    sample_count: u32,
    samples: Vec<Vec3>,
    buffer: RgbImage,
}

impl Film {
    pub fn new(width: u32, height: u32, sample_count: u32) -> Self {
        Self {
            width,
            height,
            sample_count,
            samples: Vec::with_capacity((width * height) as usize),
            buffer: ImageBuffer::new(width, height) as RgbImage,
        }
    }

    pub fn add_sample(&mut self, x: u32, y: u32, sample: Vec3) {
        assert!(
            self.samples.capacity() == ((self.width * self.height) as usize),
            "Invalid sample amount"
        );

        self.samples.push(sample);

        let color = self.get_srgb_color(&sample);
        self.buffer.put_pixel(x, y, Rgb(color));
    }

    pub fn add_samples(&mut self, samples: Vec<Vec<Vec3>>) {
        for (y, row) in samples.iter().enumerate() {
            for (x, color) in row.iter().enumerate() {
                self.add_sample(x as u32, y as u32, *color);
            }
        }
    }

    pub fn write_image(&mut self) {
        // TODO: Find better way to handle this.
        self.buffer = imageops::rotate180(&self.buffer);

        fs::remove_dir_all("./dist").ok();
        fs::create_dir("./dist").expect("Failed to create output directory");

        self.buffer
            .save("dist/output.png")
            .expect("Failed to save PNG file");

        println!("Image Location: dist/output.png");
    }

    pub fn get_srgb_color(&self, sample: &Vec3) -> [u8; 3] {
        let mut r = sample.x;
        let mut g = sample.y;
        let mut b = sample.z;

        // Gamma Correction
        let scale = 1.0 / (self.sample_count as f32);
        r = (scale * r).sqrt();
        g = (scale * g).sqrt();
        b = (scale * b).sqrt();

        [
            (256.0 * r.max(0.0).min(1.0)) as u8,
            (256.0 * g.max(0.0).min(1.0)) as u8,
            (256.0 * b.max(0.0).min(1.0)) as u8,
        ]
    }
}
