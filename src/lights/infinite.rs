use crate::{
    base::{
        constants::{Float, INV_PI, INV_TWO_PI, PI},
        geometry::{spherical_phi, spherical_theta},
        interaction::Interaction,
        light::{Light, LightPointSample, LightRaySample, VisibilityTester},
        mipmap::MIPMap,
        primitive::Primitive,
        sampling::{concentric_sample_disk, Distribution2D},
        spectrum::Spectrum,
        transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    io::image::{Image, ImageWrapMode},
    spectra::rgb::RGBSpectrum,
};

pub struct InfiniteAreaLight {
    pub mipmap: MIPMap,
    light_to_world: Transform,
    world_to_light: Transform,
    world_center: Point3,
    world_radius: Float,
    distribution: Distribution2D,
}

pub struct InfiniteAreaLightOptions<'a> {
    pub scene: &'a (dyn Primitive<'a> + 'a),
    pub transform: Transform,
    pub intensity: RGBSpectrum,
    pub filename: &'a str,
}

impl InfiniteAreaLight {
    pub fn new(opts: InfiniteAreaLightOptions) -> Self {
        let mut image = Image::read(opts.filename);
        image.scale(&opts.intensity);

        let mipmap = MIPMap::new(image, ImageWrapMode::Repeat);

        // Compute scalar-valued image from environment map.
        let width = 2 * mipmap.width();
        let height = 2 * mipmap.height();

        let mut func = vec![0.0; width * height];
        let fwidth = 0.5 / width.min(height) as Float;
        for v in 0..height {
            let vp = (v as Float + 0.5) / height as Float;
            let sin_theta = (PI * (v as Float + 0.5) / height as Float).sin();
            for u in 0..width {
                let up = (u as Float + 0.5) / width as Float;
                func[v * width + u] =
                    mipmap.trilinear_filter(&Point2F::new(up, vp), fwidth).y() * sin_theta;
            }
        }

        let light_to_world = opts.transform;
        let world_to_light = light_to_world.inverse();

        let mut world_center = Point3::default();
        let mut world_radius = 0.0;
        opts.scene
            .world_bound()
            .bounding_sphere(&mut world_center, &mut world_radius);

        Self {
            mipmap,
            light_to_world,
            world_to_light,
            world_center,
            world_radius,
            distribution: Distribution2D::new(func, width, height),
        }
    }
}

impl Light for InfiniteAreaLight {
    fn power(&self) -> RGBSpectrum {
        PI * self.world_radius
            * self.world_radius
            * self.mipmap.trilinear_filter(&Point2F::new(0.5, 0.5), 0.5)
    }

    fn radiance(&self, ray: &Ray) -> RGBSpectrum {
        let w = ray.direction.transform(&self.world_to_light).normalize();
        let st = Point2F::new(
            spherical_phi(&w) * INV_TWO_PI,
            spherical_theta(&w) * (1.0 / PI),
        );

        self.mipmap.trilinear_filter(&st, 0.0)
    }

    fn sample_point(&self, it: &dyn Interaction, u: &Point2F) -> LightPointSample {
        // Find (u,v) sample coordinates in infinite light texture.
        let mut map_pdf = 0.0;
        let uv = self.distribution.sample_continuous(u, &mut map_pdf);
        if map_pdf == 0.0 {
            return LightPointSample {
                radiance: RGBSpectrum::default(),
                wi: Vec3::default(),
                pdf: map_pdf,
                visibility: None,
            };
        }

        // Convert infinite light sample point to direction.
        let theta = uv[1] * PI;
        let phi = uv[0] * 2.0 * PI;

        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        let wi = Vec3::new(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta)
            .transform(&self.light_to_world);

        // Compute PDF for sampled infinite light direction.
        let mut pdf = map_pdf / (2.0 * PI * PI * sin_theta);
        if sin_theta == 0.0 {
            pdf = 0.0;
        }

        // Return radiance value for infinite light direction.
        let visibility = Some(VisibilityTester::new(
            BaseInteraction::from(it),
            BaseInteraction {
                p: it.p() + wi * (2.0 * self.world_radius),
                p_error: Vec3::default(),
                time: it.time(),
                wo: Vec3::default(),
                n: Normal::default(),
            },
        ));

        LightPointSample {
            radiance: self.mipmap.trilinear_filter(&uv, 0.0),
            wi,
            pdf,
            visibility,
        }
    }

    fn point_pdf(&self, _: &dyn Interaction, w: &Vec3) -> Float {
        let wi = w.transform(&self.world_to_light);

        let theta = spherical_theta(&wi);
        let phi = spherical_phi(&wi);

        let sin_theta = theta.sin();
        if sin_theta == 0.0 {
            return 0.0;
        }

        self.distribution
            .pdf(&Point2F::new(phi * INV_TWO_PI, theta * INV_PI))
            / (2.0 * PI * PI * sin_theta)
    }

    fn sample_ray(&self, u1: &Point2F, u2: &Point2F, time: Float) -> LightRaySample {
        // Compute direction for infinite light sample ray.
        let u = u1;

        // Find UV sample coordinates in infinite light texture.
        let mut map_pdf = 0.0;
        let uv = self.distribution.sample_continuous(&u, &mut map_pdf);
        if map_pdf == 0.0 {
            return LightRaySample {
                radiance: RGBSpectrum::default(),
                ray: Ray::default(),
                light_normal: Normal::default(),
                position_pdf: 0.0,
                direction_pdf: 0.0,
            };
        }

        let theta = uv[1] * PI;
        let phi = uv[0] * 2.0 * PI;

        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        let d = -Vec3::new(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta)
            .transform(&self.light_to_world);

        // Compute origin for infinite light sample ray.
        let (v1, v2) = Vec3::coordinate_system(&-d);
        let cd = concentric_sample_disk(u2);
        let disk_point = self.world_center + self.world_radius * (cd.x * v1 + cd.y * v2);

        let direction_pdf = if sin_theta == 0.0 {
            0.0
        } else {
            map_pdf / (2.0 * PI * PI * sin_theta)
        };

        LightRaySample {
            radiance: self.mipmap.trilinear_filter(&uv, 0.0),
            ray: Ray::new(
                &(disk_point + self.world_radius * -d),
                &d,
                Float::INFINITY,
                time,
            ),
            light_normal: Normal::from(d),
            position_pdf: 1.0 / (PI * self.world_radius * self.world_radius),
            direction_pdf,
        }
    }

    fn ray_pdf(&self, ray: &Ray, _: &Normal) -> (Float, Float) {
        let d = -ray.direction.transform(&self.world_to_light);

        let theta = spherical_theta(&d);
        let phi = spherical_phi(&d);

        let uv = Point2F::new(phi * INV_TWO_PI, theta * INV_PI);

        let map_pdf = self.distribution.pdf(&uv);
        (
            1.0 / (PI * self.world_radius * self.world_radius),
            map_pdf / (2.0 * PI * PI * theta.sin()),
        )
    }

    fn is_infinite(&self) -> bool {
        true
    }
}
