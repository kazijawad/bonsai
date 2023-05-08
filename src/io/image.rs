use crate::{
    base::{constants::Float, spectrum::Spectrum},
    geometries::point2::Point2,
    spectra::rgb::RGBSpectrum,
};

pub fn read_image(path: &str) -> (Point2, Vec<RGBSpectrum>) {
    let image = image::open(path).unwrap().into_rgb32f();
    let (width, height) = image.dimensions();

    let mut spectra = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y).0;
            spectra.push(RGBSpectrum::from_rgb(&pixel));
        }
    }

    (Point2::new(width as Float, height as Float), spectra)
}

pub fn inverse_gamma_correct(v: Float) -> Float {
    if v <= 0.04045 {
        v * 1.0 / 12.92
    } else {
        ((v + 0.055) * 1.0 / 1.055).powf(2.4)
    }
}
