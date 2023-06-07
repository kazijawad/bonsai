use std::sync::Arc;

use bonsai::*;

fn main() {
    let material = Arc::new(MatteMaterial {
        kd: Box::new(UVTexture {
            mapping: Box::new(UVMapping2D::default()),
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

    let primitive = Arc::new(GeometricPrimitive {
        shape: sphere,
        material,
        area_light: None,
    });

    let aggregate = Box::new(BVH::new(vec![primitive]));

    let infinite_light = Box::new(InfiniteAreaLight::new(InfiniteAreaLightOptions {
        bounds: aggregate.bounds(),
        transform: Transform::default(),
        intensity: RGBSpectrum::new(1.0),
        filename: "assets/maps/environment.exr",
    }));

    let scene = Scene::new(aggregate, vec![infinite_light]);

    let film = Film::new(FilmOptions {
        resolution: Point2F::new(1024.0, 1024.0),
        crop_window: Bounds2F::new(&Point2F::new(0.0, 0.0), &Point2F::new(1.0, 1.0)),
        filter: Box::new(BoxFilter::new(Vec2::new(0.5, 0.5))),
        filename: "dist/result.exr",
        scale: 1.0,
        max_sample_luminance: Float::INFINITY,
    });

    let camera_transform = Transform::look_at(
        &Point3::new(5.0, 0.0, 0.0),
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
        near: 0.01,
        far: 1000.0,
        film,
    }));

    let sampler = Box::new(StratifiedSampler::new(StratifiedSamplerOptions {
        x_pixel_samples: 4,
        y_pixel_samples: 4,
        sampled_dimensions: 4,
        jitter_samples: true,
    }));

    let integrator = PathIntegrator::new(camera, sampler, 5, 1.0);
    integrator.render(&scene);
}
