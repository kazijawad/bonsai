use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{camera::Camera, film::Film, math::vec3::Vec3, object::Object, ray::Ray, Scene};

#[derive(Debug, Clone, Copy)]
pub struct RenderSettings {
    pub width: u32,
    pub height: u32,
    pub background: Vec3,
    pub max_sample_count: u32,
    pub max_depth: u32,
}

pub struct Renderer<'a> {
    pub settings: RenderSettings,
    pub scene: &'a Scene,
    pub camera: &'a Camera,
    pub lights: &'a Scene,

    film: Film,
}

impl<'a> Renderer<'a> {
    pub fn new(
        settings: &RenderSettings,
        scene: &'a Scene,
        camera: &'a Camera,
        lights: &'a Scene,
    ) -> Self {
        println!("Initializing Renderer...");
        println!("Render Settings:");
        println!("Aspect Ratio: {}", camera.settings.aspect_ratio);
        println!("Width: {}", settings.width);
        println!("Height: {}", settings.height);
        println!("Sample Count: {}", settings.max_sample_count);
        println!("Bounce Count: {}", settings.max_depth);

        Self {
            settings: settings.clone(),
            scene,
            camera,
            lights,

            film: Film::new(settings.width, settings.height, settings.max_sample_count),
        }
    }

    pub fn render(&mut self) {
        self.sample();
        self.film.write_image();
    }

    fn sample(&mut self) {
        let mut rng = StdRng::from_entropy();

        for y in (0..self.settings.height).rev() {
            for x in 0..self.settings.width {
                let mut color = Vec3::zeros();
                for _ in 0..self.settings.max_sample_count {
                    color += self.get_color(x, y, &mut rng);
                }
                self.film.add_sample(x, y, color);
            }
        }

        println!("Finished rendering...");
    }

    fn get_color(&self, x: u32, y: u32, mut rng: &mut StdRng) -> Vec3 {
        let width = self.settings.width as f32;
        let height = self.settings.height as f32;

        let u = ((x as f32) + rng.gen_range(0.0..1.0)) / (width - 1.0);
        let v = ((y as f32) + rng.gen_range(0.0..1.0)) / (height - 1.0);

        let ray = self.camera.get_ray(u, v, &mut rng);
        self.trace_ray(ray, self.settings.max_depth)
    }

    fn trace_ray(&self, ray: Ray, depth: u32) -> Vec3 {
        if depth <= 0 {
            return Vec3::zeros();
        }

        match self.scene.hit(&ray, 0.0, f32::INFINITY) {
            Some(hit_record) => {
                let emitted = hit_record.material.emitted(&ray, &hit_record);

                match hit_record.material.scatter(&ray, &hit_record) {
                    Some(scatter_record) => {
                        if scatter_record.is_specular {
                            return scatter_record.attenuation
                                * self.trace_ray(scatter_record.specular, depth - 1);
                        }

                        // let mixture_pdf = MixturePDF::new(
                        //     Rc::new(HittablePDF::new(Rc::new(self.lights), &hit_record.position)),
                        //     scatter_record.distribution,
                        // );

                        let scattered = Ray::new(
                            &hit_record.position,
                            &scatter_record.distribution.generate(),
                            ray.time,
                        );
                        let pdf_value = scatter_record.distribution.value(&scattered.direction);

                        emitted
                            + scatter_record.attenuation
                                * hit_record
                                    .material
                                    .scattering_pdf(&ray, &scattered, &hit_record)
                                * self.trace_ray(scattered, depth - 1)
                                / pdf_value
                    }
                    None => {
                        return emitted;
                    }
                }
            }
            None => {
                return self.settings.background;
            }
        }
    }
}
