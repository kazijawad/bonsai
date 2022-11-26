use std::rc::Rc;

use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    math::{onb::OrthonormalBasis, vec3::Vec3},
    object::Object,
};

pub trait PDF {
    fn value(&self, direction: &Vec3) -> f32;
    fn generate(&self) -> Vec3;
}

pub struct CosinePDF {
    uvw: OrthonormalBasis,
}

pub struct HittablePDF {
    origin: Vec3,
    reference: Rc<dyn Object>,
}

pub struct MixturePDF {
    pdfs: Vec<Rc<dyn PDF>>,
}

impl CosinePDF {
    pub fn new(w: &Vec3) -> Self {
        let mut uvw = OrthonormalBasis::new();
        uvw.build_from_w(w);
        Self { uvw }
    }
}

impl HittablePDF {
    pub fn new(object: Rc<dyn Object>, origin: &Vec3) -> Self {
        Self {
            reference: Rc::clone(&object),
            origin: origin.clone(),
        }
    }
}

impl MixturePDF {
    pub fn new(pdf_a: Rc<dyn PDF>, pdf_b: Rc<dyn PDF>) -> Self {
        Self {
            pdfs: vec![Rc::clone(&pdf_a), Rc::clone(&pdf_b)],
        }
    }
}

impl PDF for CosinePDF {
    fn value(&self, direction: &Vec3) -> f32 {
        let cosine = Vec3::dot(&Vec3::normalize(direction), &self.uvw.w());
        if cosine <= 0.0 {
            0.0
        } else {
            cosine / std::f32::consts::PI
        }
    }

    fn generate(&self) -> Vec3 {
        self.uvw.local(&Vec3::random_cosine_direction())
    }
}

impl PDF for HittablePDF {
    fn value(&self, direction: &Vec3) -> f32 {
        self.reference.pdf_value(&self.origin, direction)
    }

    fn generate(&self) -> Vec3 {
        self.reference.random(&self.origin)
    }
}

impl PDF for MixturePDF {
    fn value(&self, direction: &Vec3) -> f32 {
        0.5 * self.pdfs[0].value(direction) + 0.5 * self.pdfs[1].value(direction)
    }

    fn generate(&self) -> Vec3 {
        let mut rng = StdRng::from_entropy();
        if rng.gen_range(0.0..1.0) < 0.5 {
            return self.pdfs[0].generate();
        }
        self.pdfs[1].generate()
    }
}
