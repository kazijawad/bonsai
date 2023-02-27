use crate::{
    geometries::{point3::Point3, vec3::Vec3},
    utils::math::{lerp, Float},
};

const PERMUTATION_SIZE: i32 = 256;

static PERMUTATION_LUT: [i32; (2 * PERMUTATION_SIZE) as usize] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180, 151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194,
    233, 7, 225, 140, 36, 103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234,
    75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174,
    20, 125, 136, 171, 168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83,
    111, 229, 122, 60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25,
    63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188,
    159, 86, 164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147,
    118, 126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170,
    213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253,
    19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193,
    238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31,
    181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93,
    222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
];

pub fn noise(x: Float, y: Float, z: Float) -> Float {
    let mut ix = x.floor() as i32;
    let mut iy = y.floor() as i32;
    let mut iz = z.floor() as i32;

    let dx = x - ix as Float;
    let dy = y - iy as Float;
    let dz = z - iz as Float;

    // Compute gradient weights.
    ix &= PERMUTATION_SIZE - 1;
    iy &= PERMUTATION_SIZE - 1;
    iz &= PERMUTATION_SIZE - 1;
    let w000 = grad(ix, iy, iz, dx, dy, dz);
    let w100 = grad(ix + 1, iy, iz, dx - 1.0, dy, dz);
    let w010 = grad(ix, iy + 1, iz, dx, dy - 1.0, dz);
    let w110 = grad(ix + 1, iy + 1, iz, dx - 1.0, dy - 1.0, dz);
    let w001 = grad(ix, iy, iz + 1, dx, dy, dz - 1.0);
    let w101 = grad(ix + 1, iy, iz + 1, dx - 1.0, dy, dz - 1.0);
    let w011 = grad(ix, iy + 1, iz + 1, dx, dy - 1.0, dz - 1.0);
    let w111 = grad(ix + 1, iy + 1, iz + 1, dx - 1.0, dy - 1.0, dz - 1.0);

    // Compute trilinear interpolation of weights.
    let wx = weight(dx);
    let wy = weight(dy);
    let wz = weight(dz);
    let x00 = lerp(wx, w000, w100);
    let x10 = lerp(wx, w010, w110);
    let x01 = lerp(wx, w001, w101);
    let x11 = lerp(wx, w011, w111);
    let y0 = lerp(wy, x00, x10);
    let y1 = lerp(wy, x01, x11);

    lerp(wz, y0, y1)
}

pub fn noise_point(p: &Point3) -> Float {
    noise(p.x, p.y, p.z)
}

pub fn fbm(p: &Point3, dpdx: &Vec3, dpdy: &Vec3, omega: Float, octaves: i32) -> Float {
    // Compute number of octaves for antialiased fBm.
    let len2 = dpdx.length_squared().max(dpdy.length_squared());
    let n = (-1.0 - 0.5 * len2.log2()).clamp(0.0, octaves as Float);
    let n_int = n.floor() as i32;

    // Compute sum of octaves of noise.
    let mut sum = 0.0;
    let mut lambda = 1.0;
    let mut o = 1.0;
    for i in 0..n_int {
        sum += o * noise_point(&(lambda * p));
        lambda *= 1.99;
        o *= omega as Float;
    }

    let n_partial = n - n_int as Float;
    sum += o * smoothstep(0.3, 0.7, n_partial) * noise_point(&(lambda * p));

    sum
}

pub fn turbulence(p: &Point3, dpdx: &Vec3, dpdy: &Vec3, omega: Float, octaves: i32) -> Float {
    // Compute number of octaves for antialiased fBm.
    let len2 = dpdx.length_squared().max(dpdy.length_squared());
    let n = (-1.0 - 0.5 * len2.log2()).clamp(0.0, octaves as Float);
    let n_int = n.floor() as i32;

    // Compute sum of octaves of noise for turbulence.
    let mut sum = 0.0;
    let mut lambda = 1.0;
    let mut o = 1.0;
    for i in 0..n_int {
        sum += o * noise_point(&(lambda * p)).abs();
        lambda *= 1.99;
        o *= omega as Float;
    }

    // Account for contributions of clamped octaves in turbulence.
    let n_partial = n - n_int as Float;
    sum += o * lerp(
        smoothstep(0.3, 0.7, n_partial),
        0.2,
        noise_point(&(lambda * p)).abs(),
    );
    for i in n_int..octaves {
        sum += o * 0.2;
        o *= omega;
    }

    sum
}

fn smoothstep(min: Float, max: Float, v: Float) -> Float {
    let v = ((v - min) / (max - min)).clamp(0.0, 1.0);
    v * v * (-2.0 * v + 3.0)
}

fn grad(x: i32, y: i32, z: i32, dx: Float, dy: Float, dz: Float) -> Float {
    let mut h =
        PERMUTATION_LUT[(PERMUTATION_LUT[(PERMUTATION_LUT[x as usize] + y) as usize] + z) as usize];
    h &= 15;
    let mut u = if h < 8 || h == 12 || h == 13 { dx } else { dy };
    let mut v = if h < 4 || h == 12 || h == 13 { dy } else { dz };
    if h & 1 != 0 {
        u = -u
    }
    if h & 2 != 0 {
        v = -v
    }
    u + v
}

fn weight(t: Float) -> Float {
    let t3 = t * t * t;
    let t4 = t3 * t;
    6.0 * t4 * t - 15.0 * t4 + 10.0 * t3
}
