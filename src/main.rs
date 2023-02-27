use std::sync::Arc;

use pat::*;

fn main() {
    Spectrum::init();

    let filter = GaussianFilter::create(&GaussianFilterDescriptior::default());

    let film = Film::create(&FilmDescriptor::default(), Box::new(filter));

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

    let kd = Arc::new(UVTexture::new(Box::new(UVMapping2D::new(
        1.0, 1.0, 0.0, 0.0,
    ))));
    let sigma = Arc::new(ConstantTexture::new(0.0));
    let material = MatteMaterial::new(kd, sigma);

    let primitive = GeometricPrimitive::new(&sphere, &material);
    let aggregate = BVH::new(vec![&primitive], 4);
    let scene = Scene::new(&aggregate, vec![]);

    let camera_transform = Transform::look_at(
        &Point3::new(0.0, 0.0, 3.0),
        &Point3::default(),
        &Vec3::new(0.0, 1.0, 0.0),
    );
    let camera_to_world = AnimatedTransform::new(&camera_transform, 0.0, &camera_transform, 1.0);

    let camera = PerspectiveCamera::new(&camera_to_world, 0.0, 1.0, 0.0, 1e6, 45.0, &film);

    let sampler = StratifiedSampler::new(4, 4, true, 4);

    let integrator = SamplerIntegrator::new(&camera, &sampler, camera.get_film().sample_bounds());

    integrator.render(&scene);
}
