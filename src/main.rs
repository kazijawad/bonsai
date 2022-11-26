use std::rc::Rc;

use pat::*;

fn main() {
    // Image
    let aspect_ratio = 4.0 / 3.0;
    let width = 640;
    let height = ((width as f32) / aspect_ratio) as u32;

    // Scene
    let mut scene = Scene::new();

    let sphere = Sphere::new(
        &Vec3::zeros(),
        5.0,
        Rc::new(LambertianMaterial::new(&Vec3::new(0.8, 0.5, 0.5))),
    );

    scene.add(Rc::new(sphere));

    // Lights
    let lights = Scene::new();

    // Camera
    let camera = Camera::new(&CameraSettings {
        aspect_ratio,
        position: Vec3::new(0.0, 0.0, -20.0),
        look_at: Vec3::zeros(),
        fov: 40.0,
        aperature: 0.0,
        focus_distance: 10.0,
        start_time: 0.0,
        end_time: 1.0,
    });

    // Render
    let mut renderer = Renderer::new(
        &RenderSettings {
            width,
            height,
            background: Vec3::new(135.0 / 256.0, 206.0 / 256.0, 235.0 / 256.0),
            max_sample_count: 50,
            max_depth: 10,
        },
        &scene,
        &camera,
        &lights,
    );
    renderer.render();
}
