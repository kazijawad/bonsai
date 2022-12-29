use std::sync::Arc;

use pat::*;

fn main() {
    // Parse render options and scene description into settings struct.
    let settings = parser::parse();

    // Initialize rendering information.
    let camera = Camera::from(&settings);
    let mut renderer = Renderer::from(&settings);
    let mut film = Film::from(&settings);

    // Generate scene information.
    let center_transform = Transform::default_shared();
    let offset_transform = Arc::new(Transform::translate(&Vec3::new(1.5, 0.0, 0.0)));

    let material = TestMaterial::new();

    let center_sphere = Sphere::new(
        center_transform.clone(),
        center_transform.clone(),
        false,
        0.25,
        0.0,
        1.0,
        360.0,
    );

    let offset_sphere = Sphere::new(
        offset_transform.clone(),
        Arc::new(offset_transform.inverse()),
        false,
        0.25,
        0.0,
        1.0,
        180.0,
    );

    let center_mesh = GeometricPrimitive::new(
        center_sphere,
        Some(material.clone()),
        None,
        &MediumInterface,
    );

    let offset_mesh = GeometricPrimitive::new(
        offset_sphere,
        Some(material.clone()),
        None,
        &MediumInterface,
    );

    let scene = BVH::new(vec![center_mesh.clone(), offset_mesh.clone()], 4);

    // Render scene.
    renderer.render(&(scene as Box<dyn Aggregate>), &camera);

    // Write out image.
    film.write_image(renderer.samples);
}
