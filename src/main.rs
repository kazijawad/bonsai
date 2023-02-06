use pat::*;

fn main() {
    let width = 1280;
    let height = 720;
    let sample_count = 50;
    let bounce_count = 5;

    Spectrum::init();

    let mut renderer = Renderer::new(
        width,
        height,
        Point3::new(0.53, 0.80, 0.92),
        sample_count,
        bounce_count,
    );
    let mut film = Film::new(width, height, sample_count);

    let object_to_world = Transform::default();
    let world_to_object = object_to_world.inverse();

    let sphere = Sphere::new(
        &object_to_world,
        &world_to_object,
        false,
        0.25,
        -1.0,
        1.0,
        360.0,
    );
    let material = TestMaterial::new();

    let primitive = GeometricPrimitive::new(&sphere, &material);
    let scene = BVH::new(vec![&primitive], 4);

    let camera = Camera::new(
        Point3::new(0.0, 0.0, 3.0),
        Vec3::default(),
        45.0,
        (width as Float) / (height as Float),
        0.0,
        10.0,
    );

    renderer.render(&scene, &camera);

    film.write_image(renderer.samples);
}
