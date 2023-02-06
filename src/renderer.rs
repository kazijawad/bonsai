use std::time::Instant;

use rand::{rngs::StdRng, Rng, SeedableRng};
use rayon::prelude::*;

use crate::{
    base::aggregate::Aggregate,
    camera::Camera,
    geometries::{point3::Point3, ray::Ray},
    interactions::surface::SurfaceInteraction,
    utils::math::Float,
};

pub struct Renderer {
    pub samples: Vec<Vec<Point3>>,
    width: u32,
    height: u32,
    background: Point3,
    max_sample_count: u32,
    max_depth: u32,
}

impl Renderer {
    pub fn new(
        width: u32,
        height: u32,
        background: Point3,
        max_sample_count: u32,
        max_depth: u32,
    ) -> Self {
        println!("Width: {}", width);
        println!("Height: {}", height);
        println!("Sample Count: {}", max_sample_count);
        println!("Bounce Count: {}", max_depth);

        Self {
            samples: vec![],
            width,
            height,
            background,
            max_sample_count,
            max_depth,
        }
    }

    pub fn render(&mut self, scene: &dyn Aggregate, camera: &Camera) {
        self.sample(scene, camera);
    }

    fn sample(&mut self, scene: &dyn Aggregate, camera: &Camera) {
        let render_time = Instant::now();

        let samples: Vec<_> = (0..self.height)
            .map(|y| {
                (0..self.width)
                    .into_par_iter()
                    .map(|x| {
                        let mut rng = StdRng::from_entropy();
                        let mut color = Point3::default();
                        for _ in 0..self.max_sample_count {
                            color += self.get_color(scene, camera, x, y, &mut rng);
                        }
                        color
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        println!("Render Time: {:.2?}", render_time.elapsed());
        self.samples = samples;
    }

    fn get_color(
        &self,
        scene: &dyn Aggregate,
        camera: &Camera,
        x: u32,
        y: u32,
        rng: &mut StdRng,
    ) -> Point3 {
        let width = self.width as Float;
        let height = self.height as Float;

        let u = ((x as Float) + rng.gen_range(0.0..1.0)) / (width - 1.0);
        let v = ((y as Float) + rng.gen_range(0.0..1.0)) / (height - 1.0);

        let ray = camera.at(u, v);
        self.trace_ray(scene, ray, self.max_depth)
    }

    fn trace_ray(&self, scene: &dyn Aggregate, mut ray: Ray, depth: u32) -> Point3 {
        if depth <= 0 {
            return Point3::default();
        }

        let mut interaction = SurfaceInteraction::default();
        if scene.intersect(&mut ray, &mut interaction) {
            Point3::new(1.0, 0.0, 0.0)
        } else {
            self.background
        }
    }
}
