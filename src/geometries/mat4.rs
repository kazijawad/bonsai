use crate::base::constants::Float;

#[derive(Debug, Clone, PartialEq)]
pub struct Mat4 {
    pub m: [[Float; 4]; 4],
}

impl Mat4 {
    pub fn new(
        t00: Float,
        t01: Float,
        t02: Float,
        t03: Float,
        t10: Float,
        t11: Float,
        t12: Float,
        t13: Float,
        t20: Float,
        t21: Float,
        t22: Float,
        t23: Float,
        t30: Float,
        t31: Float,
        t32: Float,
        t33: Float,
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
        let mut m_inv = self.m.clone();

        let mut ic = [0; 4];
        let mut ir = [0; 4];
        let mut pivot = [0; 4];

        for i in 0..4 {
            let mut row = 0;
            let mut col = 0;
            let mut big = 0.0;

            // Choose pivot.
            for j in 0..4 {
                if pivot[j] != 1 {
                    for k in 0..4 {
                        if pivot[k] == 0 {
                            if m_inv[j][k].abs() >= big {
                                big = m_inv[j][k].abs();
                                row = j;
                                col = k;
                            }
                        } else if pivot[k] > 1 {
                            eprintln!("Mat4::inverse produced a singular matrix");
                        }
                    }
                }
            }

            pivot[col] += 1;
            if row != col {
                for k in 0..4 {
                    let temp = m_inv[row][k];
                    m_inv[row][k] = m_inv[col][k];
                    m_inv[col][k] = temp;
                }
            }

            ir[i] = row;
            ic[i] = col;
            if m_inv[col][col] == 0.0 {
                eprintln!("Mat4::inverse produced a singular matrix");
            }

            let pivot_inv = 1.0 / m_inv[col][col];
            m_inv[col][col] = 1.0;
            for j in 0..4 {
                m_inv[col][j] *= pivot_inv;
            }

            for j in 0..4 {
                if j != col {
                    let save = m_inv[j][col];
                    m_inv[j][col] = 0.0;
                    for k in 0..4 {
                        m_inv[j][k] -= m_inv[col][k] * save;
                    }
                }
            }
        }

        for j in (0..=3).rev() {
            if ir[j] != ic[j] {
                for k in 0..4 {
                    let temp = m_inv[k][ir[j]];
                    m_inv[k][ir[j]] = m_inv[k][ic[j]];
                    m_inv[k][ic[j]] = temp;
                }
            }
        }

        Self::from(m_inv)
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

impl From<[[Float; 4]; 4]> for Mat4 {
    fn from(m: [[Float; 4]; 4]) -> Self {
        Self { m }
    }
}
