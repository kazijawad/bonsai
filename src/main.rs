use std::sync::Arc;

use bonsai::*;

fn main() {
    let material = Arc::new(MatteMaterial {
        kd: Box::new(ConstantTexture {
            value: RGBSpectrum::new(0.5),
        }),
        sigma: Box::new(ConstantTexture { value: 0.0 }),
    });

    let sphere = Arc::new(Sphere::new(SphereOptions {
        transform: Transform::default(),
        reverse_orientation: false,
        radius: 1.0,
        z_min: -1.0,
        z_max: 1.0,
        phi_max: 360.0,
    }));

    let disk = Arc::new(Disk::new(DiskOptions {
        transform: Transform::translate(&Vec3::new(0.0, 0.0, 4.0)),
        reverse_orientation: false,
        height: 0.0,
        radius: 5.0,
        inner_radius: 0.0,
        phi_max: 360.0,
    }));

    let scene = Scene::new(
        Box::new(BVH::new(
            vec![Arc::new(GeometricPrimitive {
                shape: sphere,
                material,
                area_light: None,
            })],
            4,
        )),
        vec![
            Box::new(DiffuseAreaLight::new(DiffuseAreaLightOptions {
                intensity: RGBSpectrum::new(10.0),
                shape: disk,
                double_sided: false,
            })),
            Box::new(SpotLight::new(SpotLightOptions {
                transform: Transform::default(),
                from: Point3::new(3.0, 0.0, 0.0),
                to: Point3::new(0.0, 0.0, 0.0),
                intensity: RGBSpectrum::new(1.0),
                cone_angle: 90.0,
                cone_delta_angle: 0.0,
            })),
        ],
    );

    let film = Film::new(FilmOptions {
        resolution: Point2::new(1024.0, 1024.0),
        crop_window: Bounds2::new(&Point2::new(0.0, 0.0), &Point2::new(1.0, 1.0)),
        filter: Box::new(BoxFilter::new(Vec2::new(0.5, 0.5))),
        filename: String::from("result.exr"),
        scale: 1.0,
        max_sample_luminance: Float::INFINITY,
    });

    let camera_transform = Transform::look_at(
        &Point3::new(10.0, 0.0, 0.0),
        &Point3::new(0.0, 0.0, 0.0),
        &Vec3::new(0.0, 0.0, 1.0),
    );
    let camera = Box::new(PerspectiveCamera::new(PerspectiveCameraOptions {
        animated_transform: AnimatedTransform::new(
            camera_transform.clone(),
            0.0,
            camera_transform,
            1.0,
        ),
        shutter_open: 0.0,
        shutter_close: 1.0,
        lens_radius: 0.0,
        focal_distance: 1e6,
        fov: 45.0,
        near: 0.1,
        far: 1000.0,
        film,
    }));

    let sampler = Box::new(StratifiedSampler::new(StratifiedSamplerOptions {
        x_pixel_samples: 4,
        y_pixel_samples: 4,
        dimensions: 4,
        jitter_samples: true,
    }));

    let mut integrator = WhittedIntegrator::new(camera, sampler, 5);
    integrator.render(&scene);
}
