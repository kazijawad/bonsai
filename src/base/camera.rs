use crate::{
    base::{constants::Float, film::Film, interaction::Interaction, light::VisibilityTester},
    geometries::{point2::Point2F, ray::Ray, vec3::Vec3},
    spectra::rgb::RGBSpectrum,
};

pub struct CameraRaySample {
    pub film: Point2F,
    pub lens: Point2F,
    pub time: Float,
}

pub struct CameraLensSample {
    pub radiance: RGBSpectrum,
    pub wi: Vec3,
    pub pdf: Float,
    pub visibility: VisibilityTester,
}

pub trait Camera: Send + Sync {
    fn generate_ray(&self, sample: &CameraRaySample, ray: &mut Ray) -> Float;

    fn importance_emission(&self, ray: &Ray, raster_position: Option<&mut Point2F>) -> RGBSpectrum;

    fn importance_pdf(&self, ray: &Ray, position_pdf: &mut Float, direction_pdf: &mut Float);

    fn importance_sample(
        &self,
        it: &Interaction,
        u: &Point2F,
        raster_point: &mut Point2F,
    ) -> CameraLensSample;

    fn film(&self) -> &Film;
}
