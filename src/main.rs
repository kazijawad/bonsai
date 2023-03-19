use std::sync::Arc;

use bonsai::*;

fn main() {
    let filter = Box::new(BoxFilter::new(Vec2::new(0.5, 0.5)));

    let film = Film::new(
        &Point2::new(1024.0, 1024.0),
        &Bounds2::new(&Point2::new(0.0, 0.0), &Point2::new(1.0, 1.0)),
        filter,
        String::from("result.exr"),
        1.0,
        Float::INFINITY,
    );

    let object_to_world = Arc::new(Transform::default());
    let world_to_object = Arc::new(object_to_world.inverse());

    let sphere = Arc::new(Sphere::new(
        object_to_world,
        world_to_object,
        false,
        1.0,
        -1.0,
        1.0,
        360.0,
    ));

    let kd = Arc::new(ConstantTexture::new(RGBSpectrum::new(0.5)));
    let sigma = Arc::new(ConstantTexture::new(0.0));
    let material = Arc::new(MatteMaterial::new(kd, sigma));

    let spot_from = Point3::new(2.0, 0.0, 0.0);
    let spot_dir = (Point3::default() - spot_from).normalize();
    let (spot_du, spot_dv) = Vec3::coordinate_system(&spot_dir);
    let spot_transform = Transform::translate(&spot_from.into())
        * Transform::from(Mat4::new(
            spot_du.x, spot_du.y, spot_du.z, 0.0, spot_dv.x, spot_dv.y, spot_dv.z, 0.0, spot_dir.x,
            spot_dir.y, spot_dir.z, 0.0, 0.0, 0.0, 0.0, 1.0,
        ))
        .inverse();
    let spot_light = Box::new(SpotLight::new(
        spot_transform,
        RGBSpectrum::new(1.0),
        30.0,
        25.0,
    ));

    let scene = Scene::new(
        Box::new(BVH::new(
            vec![Arc::new(GeometricPrimitive::new(sphere, material))],
            4,
        )),
        vec![spot_light],
    );

    let camera_transform = Arc::new(Transform::look_at(
        &Point3::new(3.0, 0.0, 0.0),
        &Point3::new(0.0, 0.0, 0.0),
        &Vec3::new(0.0, 0.0, 1.0),
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
        0.1,
        1000.0,
        film,
    ));

    let sampler = Box::new(StratifiedSampler::new(4, 4, true, 4));

    let mut integrator = WhittedIntegrator::new(camera, sampler, 5);
    integrator.render(&scene);
}
