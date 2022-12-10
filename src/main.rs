use pat::{
    AggregatePrimitive, Camera, Float, GeometricPrimitive, Renderer, Sphere, TestMaterial,
    Transform, Vec3,
};

fn main() {
    // Image
    let aspect_ratio = 1.0;
    let width = 640;
    let height = ((width as Float) / aspect_ratio) as u32;

    // Scene
    let background = Vec3::new(135.0 / 256.0, 206.0 / 256.0, 235.0 / 256.0);
    let mut scene = AggregatePrimitive::new();

    let transform = Transform::default();
    let shape = Sphere::new(
        &transform,
        &transform,
        false,
        1.0,
        Float::NEG_INFINITY,
        Float::INFINITY,
        360.0,
    );
    let material = TestMaterial::new();
    let mesh = GeometricPrimitive::new(shape, material);

    scene.add(mesh);

    // Camera
    let position = Vec3::new(0.0, 0.0, 3.0);
    let look_at = Vec3::default();
    let fov = 45.0;
    let camera = Camera::new(position, look_at, fov, aspect_ratio, 0.0, 10.0, 0.0, 1.0);

    // Render
    let mut renderer = Renderer::new(width, height, background, 50, 10, &camera, &scene);
    renderer.render();
}
