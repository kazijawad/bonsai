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
        let mut m_inverse = self.m.clone();

        let mut index_c: [i32; 4] = [0; 4];
        let mut index_r: [i32; 4] = [0; 4];
        let mut pivot: [i32; 4] = [0; 4];

        for i in 0..4 {
            let mut row = 0;
            let mut col = 0;
            let mut big: Float = 0.0;

            // Choose pivot.
            for j in 0..4 {
                if pivot[j] != 1 {
                    for k in 0..4 {
                        if pivot[k] == 0 {
                            if m_inverse[j][k].abs() >= big {
                                big = m_inverse[j][k].abs() as Float;
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
                    let temp = m_inverse[row][k];
                    m_inverse[row][k] = m_inverse[col][k];
                    m_inverse[col][k] = temp;
                }
            }

            index_r[i] = row as i32;
            index_c[i] = col as i32;
            if m_inverse[col][col] == 0.0 {
                eprintln!("Mat4::inverse produced a singular matrix");
            }

            let pivot_inverse = 1.0 / m_inverse[col][col];
            m_inverse[col][col] = 1.0;
            for j in 0..4 {
                m_inverse[col][j] *= pivot_inverse;
            }

            for j in 0..4 {
                if j != col {
                    let save: Float = m_inverse[j][col];
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

impl From<[[Float; 4]; 4]> for Mat4 {
    fn from(m: [[Float; 4]; 4]) -> Self {
        Self { m }
    }
}

#[cfg(test)]
mod tests {
    use crate::geometries::mat4::Mat4;

    #[test]
    fn new() {
        let a = Mat4::new(
            5.0, 7.0, 9.0, 10.0, 2.0, 3.0, 3.0, 8.0, 8.0, 10.0, 2.0, 3.0, 3.0, 3.0, 4.0, 8.0,
        );

        assert_eq!(a.m[0], [5.0, 7.0, 9.0, 10.0]);
        assert_eq!(a.m[0][0], 5.0);
        assert_eq!(a.m[0][1], 7.0);
        assert_eq!(a.m[0][2], 9.0);
        assert_eq!(a.m[0][3], 10.0);

        assert_eq!(a.m[1], [2.0, 3.0, 3.0, 8.0]);
        assert_eq!(a.m[1][0], 2.0);
        assert_eq!(a.m[1][1], 3.0);
        assert_eq!(a.m[1][2], 3.0);
        assert_eq!(a.m[1][3], 8.0);

        assert_eq!(a.m[2], [8.0, 10.0, 2.0, 3.0]);
        assert_eq!(a.m[2][0], 8.0);
        assert_eq!(a.m[2][1], 10.0);
        assert_eq!(a.m[2][2], 2.0);
        assert_eq!(a.m[2][3], 3.0);

        assert_eq!(a.m[3], [3.0, 3.0, 4.0, 8.0]);
        assert_eq!(a.m[3][0], 3.0);
        assert_eq!(a.m[3][1], 3.0);
        assert_eq!(a.m[3][2], 4.0);
        assert_eq!(a.m[3][3], 8.0);
    }

    #[test]
    fn mul() {
        let a = Mat4::new(
            5.0, 7.0, 9.0, 10.0, 2.0, 3.0, 3.0, 8.0, 8.0, 10.0, 2.0, 3.0, 3.0, 3.0, 4.0, 8.0,
        );
        let b = Mat4::new(
            3.0, 10.0, 12.0, 18.0, 12.0, 1.0, 4.0, 9.0, 9.0, 10.0, 12.0, 2.0, 3.0, 12.0, 4.0, 10.0,
        );
        let c = Mat4::new(
            210.0, 267.0, 236.0, 271.0, 93.0, 149.0, 104.0, 149.0, 171.0, 146.0, 172.0, 268.0,
            105.0, 169.0, 128.0, 169.0,
        );
        assert_eq!(Mat4::mul(&a, &b), c);
    }

    #[test]
    fn transpose() {
        let a = Mat4::new(
            5.0, 7.0, 9.0, 10.0, 2.0, 3.0, 3.0, 8.0, 8.0, 10.0, 2.0, 3.0, 3.0, 3.0, 4.0, 8.0,
        );
        let b = Mat4::new(
            5.0, 2.0, 8.0, 3.0, 7.0, 3.0, 10.0, 3.0, 9.0, 3.0, 2.0, 4.0, 10.0, 8.0, 3.0, 8.0,
        );
        assert_eq!(a.transpose(), b);
    }

    #[test]
    fn inverse() {
        let a = Mat4::new(
            10.0, 20.0, 10.0, 3.0, 4.0, 5.0, 6.0, 2.0, 2.0, 3.0, 5.0, 6.0, 8.0, 1.0, 9.0, 6.0,
        );
        let b = Mat4::new(
            0.10898379,
            -0.3932253,
            -0.14506626,
            0.22164947,
            0.055964652,
            -0.039764352,
            0.047128126,
            -0.061855666,
            -0.13549337,
            0.56995577,
            -0.008836538,
            -0.11340205,
            0.04860087,
            -0.32400587,
            0.19882178,
            0.05154638,
        );
        assert_eq!(a.inverse(), b);
    }

    #[test]
    fn default() {
        let a = Mat4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );
        assert_eq!(a, Mat4::default());
    }

    #[test]
    fn from() {
        let a = Mat4::new(
            5.0, 7.0, 9.0, 10.0, 2.0, 3.0, 3.0, 8.0, 8.0, 10.0, 2.0, 3.0, 3.0, 3.0, 4.0, 8.0,
        );
        assert_eq!(
            a,
            Mat4::from([
                [5.0, 7.0, 9.0, 10.0],
                [2.0, 3.0, 3.0, 8.0],
                [8.0, 10.0, 2.0, 3.0],
                [3.0, 3.0, 4.0, 8.0]
            ])
        )
    }
}
