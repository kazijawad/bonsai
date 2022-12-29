use std::sync::Arc;

use crate::{
    base::{
        material::{Material, MaterialType, TestMaterial},
        primitive::{Primitive, PrimitiveType},
        shape::{Shape, ShapeType},
    },
    geometries::vec3::Vec3,
    medium::MediumInterface,
    primitives::geometric::GeometricPrimitive,
    shapes::sphere::Sphere,
    transform::Transform,
    utils::parser::SceneSettings,
};

pub fn create_materials(settings: &SceneSettings) -> Vec<Arc<dyn Material>> {
    let material_settings = &settings.materials;

    let mut materials: Vec<Arc<dyn Material>> = Vec::with_capacity(material_settings.len());
    for material in material_settings.iter() {
        materials.push(match material.name {
            MaterialType::Test => TestMaterial::new(),
        });
    }

    materials
}

pub fn create_primitives<'a>(
    settings: &SceneSettings,
    materials: &Vec<Arc<dyn Material>>,
) -> Vec<Arc<dyn Primitive<'a> + 'a>> {
    // TODO: Define world space transformation programmatically.
    let default_transform = Arc::new(Transform::default());

    // Generate shapes.
    let shape_settings = &settings.shapes;
    let mut shapes: Vec<Arc<dyn Shape>> = Vec::with_capacity(shape_settings.len());
    for shape in shape_settings.iter() {
        match shape.name {
            ShapeType::Sphere => {
                // TODO: Setup transformation cache.
                // TODO: Handle different types of transformations.
                let (object_to_world, world_to_object) = if let Some(translation) = shape.translate
                {
                    let transform = Transform::translate(&Vec3::from(translation));
                    let inverted_transform = transform.inverse();
                    (Arc::new(transform), Arc::new(inverted_transform))
                } else {
                    (
                        Arc::clone(&default_transform),
                        Arc::clone(&default_transform),
                    )
                };

                let reverse_orientation = shape.reverse_orientation.unwrap_or(false);

                let (radius, z_min, z_max, phi_max) = if let Some(props) = &shape.properties {
                    (
                        props.radius.unwrap_or(1.0),
                        props.z_min.unwrap_or(0.0),
                        props.z_max.unwrap_or(1.0),
                        props.phi_max.unwrap_or(360.0),
                    )
                } else {
                    (1.0, 0.0, 1.0, 360.0)
                };

                shapes.push(Sphere::new(
                    object_to_world,
                    world_to_object,
                    reverse_orientation,
                    radius,
                    z_min,
                    z_max,
                    phi_max,
                ));
            }
            // TODO: Handle remaining shapes.
            _ => (),
        }
    }

    // Generate primitives.
    let primitive_settings = &settings.primitives;
    let mut primitives: Vec<Arc<dyn Primitive>> = Vec::with_capacity(primitive_settings.len());
    for primitive in primitive_settings.iter() {
        let shape = shapes[primitive.shape].clone();
        let material = materials[primitive.material].clone();
        match primitive.name {
            PrimitiveType::Geometric => primitives.push(GeometricPrimitive::new(
                shape,
                Some(material),
                None,
                &MediumInterface,
            )),
            // TODO: Handle remaining primitives.
            _ => (),
        }
    }

    primitives
}
