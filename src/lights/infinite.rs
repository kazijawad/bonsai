use crate::{
    base::{
        constants::{Float, PI},
        geometry::{spherical_phi, spherical_theta},
        interaction::Interaction,
        light::{Light, LightPointSample, VisibilityTester},
        mipmap::MIPMap,
        sampling::Distribution2D,
        scene::Scene,
        spectrum::Spectrum,
        transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2F, point3::Point3, ray::Ray, vec3::Vec3},
    interactions::base::BaseInteraction,
    io::image::{Image, ImageWrapMode},
    spectra::rgb::RGBSpectrum,
};

pub struct InfiniteAreaLight {
    light_to_world: Transform,
    world_to_light: Transform,
    mipmap: MIPMap,
    world_center: Point3,
    world_radius: Float,
    distribution: Distribution2D,
}

pub struct InfiniteAreaLightOptions<'a> {
    pub scene: &'a Scene<'a>,
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
            light_to_world,
            world_to_light,
            mipmap,
            world_center,
            world_radius,
            distribution: Distribution2D::new(func.as_slice(), width, height),
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
        let w = ray
            .direction
            .transform(&self.world_to_light, false)
            .0
            .normalize();
        let st = Point2F::new(
            spherical_phi(&w) * (1.0 / (2.0 * PI)),
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
            .transform(&self.light_to_world, false)
            .0;

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

    fn point_pdf(&self, _: &dyn Interaction, _dir: &Vec3) -> Float {
        todo!()
    }

    fn sample_ray(
        &self,
        u1: &Point2F,
        u2: &Point2F,
        time: Float,
    ) -> (RGBSpectrum, Ray, crate::Normal, Float, Float) {
        todo!()
    }

    fn ray_pdf(&self, ray: &Ray, n: &Normal) -> (Float, Float) {
        todo!()
    }

    fn is_infinite(&self) -> bool {
        true
    }
}
