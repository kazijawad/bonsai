use crate::{
    base::{
        interaction::Interaction,
        light::{Light, LightFlag, VisibilityTester},
        spectrum::Spectrum,
        transform::Transform,
    },
    geometries::{normal::Normal, point2::Point2, point3::Point3, ray::Ray, vec3::Vec3},
    utils::math::Float,
};

#[derive(Debug, Clone)]
pub struct PointLight {
    light_to_world: Transform,
    world_to_light: Transform,
    position: Point3,
    intensity: Spectrum,
    flag: LightFlag,
}

impl PointLight {
    pub fn new(light_to_world: Transform, intensity: Spectrum) -> Self {
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
    fn power(&self) -> Spectrum {
        todo!()
    }

    fn sample_li(
        &self,
        it: &dyn Interaction,
        wi: &mut Vec3,
        pdf: &mut Float,
        vis: &mut VisibilityTester,
    ) -> Spectrum {
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
    ) -> Spectrum {
        todo!()
    }

    fn pdf_le(&self, ray: &Ray, light_norm: Normal, pdf_pos: &mut Float, pdf_dir: &mut Float) {
        todo!()
    }

    fn flag(&self) -> LightFlag {
        self.flag
    }
}
