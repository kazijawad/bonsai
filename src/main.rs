use pat::*;

fn main() {
    // Parse render options and scene description into settings struct.
    let settings = parser::parse();

    // Initialize rendering information.
    let camera = Camera::from(&settings);
    let mut renderer = Renderer::from(&settings);
    let mut film = Film::from(&settings);

    // Generate scene information.
    let materials = create_materials(&settings);
    let primitives = create_primitives(&settings, &materials);
    let scene = BVH::new(primitives, 4);

    // Render scene.
    renderer.render(&(scene as Box<dyn Aggregate>), &camera);

    // Write out image.
    film.write_image(renderer.samples);
}
