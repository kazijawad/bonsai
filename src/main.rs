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

    let material = Arc::new(MatteMaterial::new(
        Arc::new(ConstantTexture::new(RGBSpectrum::new(0.5))),
        Arc::new(ConstantTexture::new(0.0)),
    ));

    let sphere_transform = Arc::new(Transform::default());
    let sphere = Arc::new(Sphere::new(
        sphere_transform.clone(),
        sphere_transform,
        false,
        1.0,
        -1.0,
        1.0,
        360.0,
    ));

    let disk_transform = Arc::new(Transform::translate(&Vec3::new(0.0, 0.0, 2.0)));
    let disk = Arc::new(Disk::new(
        disk_transform.clone(),
        Arc::new(disk_transform.inverse()),
        false,
        0.0,
        1.0,
        0.0,
        360.0,
    ));

    let spot_from = Point3::new(2.0, 0.0, 0.0);
    let spot_dir = (Point3::new(0.0, 0.0, 2.0) - spot_from).normalize();
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
            vec![
                Arc::new(GeometricPrimitive::new(sphere, material.clone(), None)),
                Arc::new(GeometricPrimitive::new(disk.clone(), material, None)),
            ],
            4,
        )),
        vec![
            spot_light,
            Box::new(PointLight::new(
                Transform::translate(&Vec3::new(2.0, 0.0, 0.0)),
                RGBSpectrum::new(1.0),
            )),
            Box::new(DiffuseAreaLight::new(RGBSpectrum::new(1.0), disk, false)),
        ],
    );

    let camera_transform = Arc::new(Transform::look_at(
        &Point3::new(10.0, 0.0, 0.0),
        &Point3::new(0.0, 0.0, 0.0),
        &Vec3::new(0.0, 0.0, 1.0),
    ));
    let camera = Box::new(PerspectiveCamera::new(
        AnimatedTransform::new(camera_transform.clone(), 0.0, camera_transform, 1.0),
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
