#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Mat4 {
    pub fn new(
        t00: f32,
        t01: f32,
        t02: f32,
        t03: f32,
        t10: f32,
        t11: f32,
        t12: f32,
        t13: f32,
        t20: f32,
        t21: f32,
        t22: f32,
        t23: f32,
        t30: f32,
        t31: f32,
        t32: f32,
        t33: f32,
    ) -> Self {
        Self {
            m: [
                [t00, t01, t02, t03],
                [t10, t11, t12, t13],
                [t20, t21, t22, t23],
                [t30, t31, t32, t33],
            ],
        }
    }

    pub fn mul(m1: &Self, m2: &Self) -> Self {
        let mut r = Self::default();
        for i in 0..4 {
            for j in 0..4 {
                r.m[i][j] = m1.m[i][0] * m2.m[0][j]
                    + m1.m[i][1] * m2.m[1][j]
                    + m1.m[i][2] * m2.m[2][j]
                    + m1.m[i][3] * m2.m[3][j];
            }
        }
        r
    }

    pub fn transpose(&self) -> Self {
        Self::new(
            self.m[0][0],
            self.m[1][0],
            self.m[2][0],
            self.m[3][0],
            self.m[0][1],
            self.m[1][1],
            self.m[2][1],
            self.m[3][1],
            self.m[0][2],
            self.m[1][2],
            self.m[2][2],
            self.m[3][2],
            self.m[0][3],
            self.m[1][3],
            self.m[2][3],
            self.m[3][3],
        )
    }

    pub fn inverse(&self) -> Self {
        let mut m_inverse = self.m.clone();

        let mut index_c: [i32; 4] = [0; 4];
        let mut index_r: [i32; 4] = [0; 4];
        let mut pivot: [i32; 4] = [0; 4];

        for i in 0..4 {
            let mut row = 0;
            let mut col = 0;
            let mut big = 0.0;

            for j in 0..4 {
                if pivot[j] != 1 {
                    for k in 0..4 {
                        if pivot[k] == 0 {
                            if m_inverse[j][k].abs() >= big {
                                big = m_inverse[j][k].abs() as f32;
                                row = j;
                                col = k;
                            }
                        } else if pivot[k] > 1 {
                            panic!("Singular matrix in matrix inverse");
                        }
                    }
                }
            }

            pivot[col] += 1;
            if row != col {
                for k in 0..4 {
                    let temp = m_inverse[row][k];
                    m_inverse[row][k] = m_inverse[col][k];
                    m_inverse[col][k] = temp;
                }
            }

            index_r[i] = row as i32;
            index_c[i] = col as i32;
            if m_inverse[col][col] == 0.0 {
                panic!("Singular matrix in matrix inverse");
            }

            let pivot_inverse = 1.0 / m_inverse[col][col];
            m_inverse[col][col] = 1.0;
            for j in 0..4 {
                m_inverse[col][j] *= pivot_inverse;
            }

            for j in 0..4 {
                if j != col {
                    let save = m_inverse[j][col];
                    m_inverse[j][col] = 0.0;
                    for k in 0..4 {
                        m_inverse[j][k] -= m_inverse[col][k] * save;
                    }
                }
            }
        }

        for j in (0..=3).rev() {
            if index_r[j] != index_c[j] {
                for k in 0..4 {
                    let r = index_r[j] as usize;
                    let c = index_c[j] as usize;
                    let temp = m_inverse[k][r];
                    m_inverse[k][r] = m_inverse[k][c];
                    m_inverse[k][c] = temp;
                }
            }
        }

        Self::from(m_inverse)
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl From<[[f32; 4]; 4]> for Mat4 {
    fn from(m: [[f32; 4]; 4]) -> Self {
        Self { m }
    }
}