use crate::{
    base::{
        constants::{Float, INV_TWO_PI, ONE_MINUS_EPSILON, PI, PI_OVER_TWO},
        math::find_interval,
        rng::RNG,
    },
    geometries::{point2::Point2F, vec2::Vec2, vec3::Vec3},
};

pub struct Distribution1D {
    func: Vec<Float>,
    cdf: Vec<Float>,
    func_int: Float,
}

pub struct Distribution2D {
    p_cond_v: Vec<Distribution1D>,
    p_marginal: Distribution1D,
}

impl Distribution1D {
    pub fn new(func: Vec<Float>, n: usize) -> Self {
        // Compute integral of step function at xi.
        let mut cdf = Vec::with_capacity(n + 1);
        cdf.push(0.0);
        for i in 1..(n + 1) {
            cdf.push(cdf[i - 1] + func[i - 1] / n as Float);
        }

        // Transform step function integral into CDF.
        let func_int = cdf[n];
        if func_int == 0.0 {
            for i in 1..(n + 1) {
                cdf[i] = i as Float / n as Float;
            }
        } else {
            for i in 1..(n + 1) {
                cdf[i] /= func_int;
            }
        }

        Self {
            func,
            cdf,
            func_int,
        }
    }

    pub fn count(&self) -> usize {
        self.func.len()
    }

    pub fn sample_continuous(&self, u: Float, pdf: &mut Float, off: Option<&mut usize>) -> Float {
        // Find surrounding CDF segments and offset.
        let offset = find_interval(self.cdf.len(), |index| self.cdf[index] <= u);
        if let Some(o) = off {
            *o = offset;
        }

        // Compute offset along CDF segment.
        let mut du = u - self.cdf[offset];
        if self.cdf[offset + 1] - self.cdf[offset] > 0.0 {
            debug_assert!(self.cdf[offset + 1] > self.cdf[offset]);
            du /= self.cdf[offset + 1] - self.cdf[offset]
        }
        debug_assert!(!du.is_nan());

        // Compute PDF for sampled offset.
        *pdf = if self.func_int > 0.0 {
            self.func[offset] / self.func_int
        } else {
            0.0
        };

        (offset as Float + du) / self.count() as Float
    }
}

impl Distribution2D {
    pub fn new(func: Vec<Float>, nu: usize, nv: usize) -> Self {
        let mut p_cond_v = Vec::with_capacity(nv);
        for v in 0..nv {
            // Compute conditional sampling distribution for v.
            p_cond_v.push(Distribution1D::new(func[v * nu..].to_vec(), nu))
        }

        // Compute marginal sampling distribution.
        let mut marginal_func = Vec::with_capacity(nv);
        for v in 0..nv {
            marginal_func.push(p_cond_v[v].func_int)
        }

        Self {
            p_cond_v,
            p_marginal: Distribution1D::new(marginal_func, nv),
        }
    }

    pub fn sample_continuous(&self, u: &Point2F, pdf: &mut Float) -> Point2F {
        let mut pdfs = [0.0, 0.0];
        let mut v = 0;

        let d1 = self
            .p_marginal
            .sample_continuous(u[1], &mut pdfs[1], Some(&mut v));
        let d0 = self.p_cond_v[v].sample_continuous(u[0], &mut pdfs[0], None);

        *pdf = pdfs[0] * pdfs[1];

        Point2F::new(d0, d1)
    }

    pub fn pdf(&self, p: &Point2F) -> Float {
        let u = (p[0] as usize * self.p_cond_v[0].count()).clamp(0, self.p_cond_v[0].count() - 1);
        let v = (p[1] as usize * self.p_marginal.count()).clamp(0, self.p_marginal.count() - 1);
        self.p_cond_v[v].func[u] / self.p_marginal.func_int
    }
}

pub fn shuffle<T>(rng: &mut RNG, samples: &mut [T], dimensions: usize) {
    let sample_count = samples.len();

    for i in 0..sample_count {
        let other = i + rng.uniform_discrete_range(0, sample_count - i);
        for j in 0..dimensions {
            samples.swap(dimensions * i + j, dimensions * other + j);
        }
    }
}

pub fn stratified_sample_1d(rng: &mut RNG, samples: &mut [Float], jitter_samples: bool) {
    let sample_count = samples.len();

    let inv_sample_count = 1.0 / sample_count as Float;

    for i in 0..sample_count {
        let delta = if jitter_samples {
            rng.uniform_continuous()
        } else {
            0.5
        };

        samples[i] = ((i as Float + delta) * inv_sample_count).min(ONE_MINUS_EPSILON);
    }
}

pub fn stratified_sample_2d(
    rng: &mut RNG,
    samples: &mut [Point2F],
    nx: usize,
    ny: usize,
    jitter_samples: bool,
) {
    let delta_x = 1.0 / nx as Float;
    let delta_y = 1.0 / ny as Float;

    let mut i = 0;
    for y in 0..ny {
        for x in 0..nx {
            let jitter_x = if jitter_samples {
                rng.uniform_continuous()
            } else {
                0.5
            };

            let jitter_y = if jitter_samples {
                rng.uniform_continuous()
            } else {
                0.5
            };

            let sample = &mut samples[i];
            sample.x = Float::min((x as Float + jitter_x) * delta_x, ONE_MINUS_EPSILON);
            sample.y = Float::min((y as Float + jitter_y) * delta_y, ONE_MINUS_EPSILON);

            i += 1;
        }
    }
}

pub fn concentric_sample_disk(u: &Point2F) -> Point2F {
    // Map uniform values to [-1, 1].
    let offset = 2.0 * u - Vec2::new(1.0, 1.0);

    // Handle degeneracy at the origin.
    if offset.x == 0.0 && offset.y == 0.0 {
        return Point2F::default();
    }

    // Apply concentric mapping to point.
    let (theta, radius) = if offset.x.abs() > offset.y.abs() {
        ((PI / 4.0) * (offset.y / offset.x), offset.x)
    } else {
        (PI_OVER_TWO - (PI / 4.0) * (offset.x / offset.y), offset.y)
    };

    radius * Point2F::new(theta.cos(), theta.sin())
}

pub fn uniform_sample_hemisphere(u: &Point2F) -> Vec3 {
    let z = u[0];
    let r = (1.0 - z * z).max(0.0).sqrt();
    let phi = 2.0 * PI * u[1];
    Vec3::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn uniform_hemisphere_pdf() -> Float {
    INV_TWO_PI
}

pub fn uniform_sample_sphere(u: &Point2F) -> Vec3 {
    let z = 1.0 - 2.0 * u[0];
    let r = (1.0 - z * z).max(0.0).sqrt();
    let phi = 2.0 * PI * u[1];
    Vec3::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn uniform_sphere_pdf() -> Float {
    1.0 / (4.0 * PI)
}

pub fn uniform_sample_cone(u: &Point2F, cos_theta_max: Float) -> Vec3 {
    let cos_theta = (1.0 - u[0]) + u[0] * cos_theta_max;
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    let phi = u[1] * 2.0 * PI;
    Vec3::new(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
}

pub fn uniform_cone_pdf(cos_theta_max: Float) -> Float {
    1.0 / (2.0 * PI * (1.0 - cos_theta_max))
}

pub fn uniform_sample_triangle(u: &Point2F) -> Point2F {
    let sqrt0 = u[0].sqrt();
    Point2F::new(1.0 - sqrt0, u[1] * sqrt0)
}

pub fn cosine_sample_hemisphere(u: &Point2F) -> Vec3 {
    let d = concentric_sample_disk(u);
    let z = (1.0 - d.x * d.x - d.y * d.y).max(0.0).sqrt();
    Vec3::new(d.x, d.y, z)
}

pub fn cosine_hemisphere_pdf(cos_theta: Float) -> Float {
    cos_theta * (1.0 / PI)
}

pub fn power_heuristic(nf: Float, pdf_f: Float, ng: Float, pdf_g: Float) -> Float {
    let f = nf * pdf_f;
    let g = ng * pdf_g;
    (f * f) / (f * f + g * g)
}
