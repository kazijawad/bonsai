use std::sync::Arc;

use pat::*;

fn main() {
    let filter = Box::new(GaussianFilter::create(&GaussianFilterDescriptior::default()));

    let film = Film::new(
        &Point2::new(1280.0, 720.0),
        &Bounds2::new(&Point2::new(0.0, 0.0), &Point2::new(1.0, 1.0)),
        filter,
        String::from("result.exr"),
        1.0,
        Float::INFINITY,
    );

    let object_to_world = Arc::new(Transform::default());
    let world_to_object = Arc::new(object_to_world.inverse());

    let sphere = Sphere::new(
        object_to_world,
        world_to_object,
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

    let point_light = Arc::new(PointLight::create(
        &PointLightDescriptor::default(),
        Transform::default(),
    ));

    let scene = Scene::new(&aggregate, vec![point_light]);

    let camera_transform = Arc::new(Transform::look_at(
        &Point3::new(0.0, 0.0, 3.0),
        &Point3::default(),
        &Vec3::new(0.0, 1.0, 0.0),
    ));
    let camera_to_world =
        AnimatedTransform::new(camera_transform.clone(), 0.0, camera_transform, 1.0);
    let camera = Box::new(PerspectiveCamera::new(
        camera_to_world,
        0.0,
        1.0,
        0.0,
        1e6,
        45.0,
        film,
    ));

    let sampler = Box::new(StratifiedSampler::new(4, 4, true, 4));

    let mut integrator = WhittedIntegrator::new(camera, sampler, 5);
    integrator.render(&scene);
}
