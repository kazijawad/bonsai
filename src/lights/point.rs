use crate::{
    base::{
        interaction::Interaction,
        light::{Light, LightFlag, VisibilityTester},
        transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
    utils::math::Float,
};

#[derive(Debug, Clone)]
pub struct PointLight {
    light_to_world: Transform,
    world_to_light: Transform,
    position: Point3,
    intensity: RGBSpectrum,
    flag: LightFlag,
}

#[derive(Debug)]
pub struct PointLightDescriptor {
    pub intensity: RGBSpectrum,
    pub scale: RGBSpectrum,
    pub from: Point3,
}

impl PointLight {
    pub fn create(desc: &PointLightDescriptor, light_to_world: Transform) -> Self {
        let light_to_world = Transform::translate(&Vec3::from(desc.from)) * light_to_world;
        Self::new(light_to_world, desc.intensity * desc.scale)
    }

    pub fn new(light_to_world: Transform, intensity: RGBSpectrum) -> Self {
        let world_to_light = light_to_world.inverse();
        let position = Point3::default().transform(&light_to_world);
        Self {
            light_to_world,
            world_to_light,
            position,
            intensity,
            flag: LightFlag::DeltaPosition,
        }
    }
}

impl Light for PointLight {
    fn power(&self) -> RGBSpectrum {
        todo!()
    }

    fn sample_li(
        &self,
        it: &dyn Interaction,
        wi: &mut Vec3,
        pdf: &mut Float,
        vis: &mut VisibilityTester,
    ) -> RGBSpectrum {
        todo!()
    }

    fn pdf_li(&self, it: &dyn Interaction, wi: &Vec3) -> Float {
        todo!()
    }

    fn sample_le(
        &self,
        u1: &Point2,
        u2: &Point2,
        time: Float,
        ray: &mut Ray,
        light_norm: Normal,
        pdf_pos: &mut Float,
        pdf_dir: &mut Float,
    ) -> RGBSpectrum {
        todo!()
    }

    fn pdf_le(&self, ray: &Ray, light_norm: Normal, pdf_pos: &mut Float, pdf_dir: &mut Float) {
        todo!()
    }

    fn flag(&self) -> LightFlag {
        self.flag
    }
}

impl Default for PointLightDescriptor {
    fn default() -> Self {
        Self {
            intensity: RGBSpectrum::new(1.0),
            scale: RGBSpectrum::new(1.0),
            from: Point3::new(0.0, 0.0, 0.0),
        }
    }
}
