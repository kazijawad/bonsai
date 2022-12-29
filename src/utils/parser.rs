use std::{fs, process};

use clap::Parser;
use serde::Deserialize;

use crate::{
    base::{material::MaterialType, primitive::PrimitiveType, shape::ShapeType},
    utils::math::Float,
};

#[derive(Debug, Parser)]
struct Args {
    scene: String,
}

#[derive(Debug, Deserialize)]
pub struct SceneSettings {
    pub render: RenderSettings,
    pub film: FilmSettings,
    pub camera: CameraSettings,
    pub materials: Vec<MaterialSettings>,
    pub shapes: Vec<ShapeSettings>,
    pub primitives: Vec<PrimitiveSettings>,
}

#[derive(Debug, Deserialize)]
pub struct RenderSettings {
    pub max_sample_count: u32,
    pub max_depth: u32,
}

#[derive(Debug, Deserialize)]
pub struct FilmSettings {
    pub width: u32,
    pub height: u32,
    pub background: [Float; 3],
}

#[derive(Debug, Deserialize)]
pub struct CameraSettings {
    pub position: [Float; 3],
    pub look_at: [Float; 3],
    pub fov: Float,
    pub aperature: Float,
    pub focus_distance: Float,
}

#[derive(Debug, Deserialize)]
pub struct MaterialSettings {
    pub name: MaterialType,
}

#[derive(Debug, Deserialize)]
pub struct ShapeSettings {
    pub name: ShapeType,
    pub reverse_orientation: Option<bool>,
    pub translate: Option<[Float; 3]>,
    pub rotate: Option<[Float; 3]>,
    pub scale: Option<[Float; 3]>,
    pub properties: Option<PropertySettings>,
}

#[derive(Debug, Deserialize)]
pub struct PrimitiveSettings {
    pub name: PrimitiveType,
    pub shape: usize,
    pub material: usize,
}

#[derive(Debug, Deserialize)]
pub struct PropertySettings {
    pub radius: Option<Float>,
    pub z_min: Option<Float>,
    pub z_max: Option<Float>,
    pub phi_max: Option<Float>,
}

pub fn parse() -> SceneSettings {
    let args = Args::parse();

    let contents = match fs::read_to_string(&args.scene) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Failed to read file: {}", args.scene);
            process::exit(1);
        }
    };

    match toml::from_str(&contents) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse TOML file: {}\n{}", e, contents);
            process::exit(1);
        }
    }
}
