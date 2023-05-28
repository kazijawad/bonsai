use std::sync::Arc;

use bonsai::*;

fn main() {
    let material = Arc::new(MatteMaterial {
        kd: Box::new(ConstantTexture {
            value: RGBSpectrum::new(1.0),
        }),
        sigma: Box::new(ConstantTexture { value: 0.0 }),
    });

    let triangles = OBJ::read(
        "assets/meshes/bunny.obj",
        Transform::translate(&Vec3::new(0.0, 0.0, -0.1))
            * Transform::rotate(180.0, &Vec3::new(0.0, 0.0, 1.0))
            * Transform::rotate(90.0, &Vec3::new(1.0, 0.0, 0.0)),
    );

    let mut primitives: Vec<Arc<dyn Primitive>> = Vec::with_capacity(triangles.len());
    for triangle in triangles {
        primitives.push(Arc::new(GeometricPrimitive {
            shape: Arc::new(triangle),
            material: material.clone(),
            area_light: None,
        }))
    }

    let aggregate = Box::new(BVH::new(primitives));

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
        &Point3::new(0.5, 0.0, 0.0),
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

    let integrator =
        DirectLightingIntegrator::new(camera, sampler, &scene, 5, LightStrategy::UniformSampleAll);
    integrator.render(&scene);
}
