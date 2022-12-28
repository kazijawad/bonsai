use std::sync::Arc;

use pat::*;

fn main() {
    let settings = parser::parse();

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

    let bvh = BVH::new(vec![center_mesh.clone(), offset_mesh.clone()], 4);

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
        camera,
        bvh,
    );
    renderer.render();
}
