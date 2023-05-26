use crate::{
    base::{constants::Float, interaction::Interaction},
    geometries::{normal::Normal, point3::Point3, vec3::Vec3},
};

#[derive(Debug, Clone)]
pub struct BaseInteraction {
    pub p: Point3,
    pub p_error: Vec3,
    pub time: Float,
    pub wo: Vec3,
    pub n: Normal,
}

impl<'a> Interaction<'a> for BaseInteraction {
    fn p(&self) -> Point3 {
        self.p
    }

    fn p_error(&self) -> Vec3 {
        self.p_error
    }

    fn time(&self) -> Float {
        self.time
    }

    fn wo(&self) -> Vec3 {
        self.wo
    }

    fn n(&self) -> Normal {
        self.n
    }
}

impl<'a> From<&dyn Interaction<'a>> for BaseInteraction {
    fn from(it: &dyn Interaction) -> Self {
        Self {
            p: it.p(),
            p_error: it.p_error(),
            time: it.time(),
            wo: it.wo(),
            n: it.n(),
        }
    }
}
