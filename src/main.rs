use std::{fs, process};

use clap::Parser;
use serde::Deserialize;

use pat::{
    point3::Point3, sphere::Sphere, vec3::Vec3, AggregatePrimitive, Camera, Float,
    GeometricPrimitive, Renderer, TestMaterial, Transform,
};

#[derive(Debug, Parser)]
struct Args {
    scene: String,
}

#[derive(Debug, Deserialize)]
struct SceneSettings {
    render: RenderSettings,
    film: FilmSettings,
    camera: CameraSettings,
}

#[derive(Debug, Deserialize)]
struct RenderSettings {
    max_sample_count: u32,
    max_depth: u32,
}

#[derive(Debug, Deserialize)]
struct FilmSettings {
    width: u32,
    height: u32,
    background: [Float; 3],
}

#[derive(Debug, Deserialize)]
struct CameraSettings {
    position: [Float; 3],
    look_at: [Float; 3],
    fov: Float,
    aperature: Float,
    focus_distance: Float,
}

fn main() {
    let args = Args::parse();

    let contents = match fs::read_to_string(&args.scene) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Failed to read file: {}", args.scene);
            process::exit(1);
        }
    };

    let settings: SceneSettings = match toml::from_str(&contents) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Failed to parse TOML file:\n{}", contents);
            process::exit(1);
        }
    };

    let transform = Transform::default();
    let shape = Sphere::new(&transform, &transform, false, 0.5, 0.0, 1.0, 360.0);
    let material = TestMaterial::new();
    let mesh = GeometricPrimitive::new(shape, material);

    let scene = AggregatePrimitive::new(vec![mesh]);

    let camera = Camera::new(
        Point3::from(settings.camera.position),
        Vec3::from(settings.camera.look_at),
        settings.camera.fov,
        settings.film.width as f32 / settings.film.height as f32,
        settings.camera.aperature,
        settings.camera.focus_distance,
    );

    let mut renderer = Renderer::new(
        settings.film.width,
        settings.film.height,
        Point3::from(settings.film.background),
        settings.render.max_sample_count,
        settings.render.max_depth,
        &camera,
        scene,
    );
    renderer.render();
}
