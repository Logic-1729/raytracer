use crate::rtweekend::random::random_int;
use crate::vec3::Vec3;
type Point3 = Vec3;

#[derive(Clone)]
pub struct Perlin {
    randvec: Vec<Vec3>,
    //randfloat: Vec<f64>,
    perm_x: Vec<i32>,
    perm_y: Vec<i32>,
    perm_z: Vec<i32>,
}

impl Perlin {
    const POINT_COUNT: usize = 256;

    /*randfloat
    pub fn new() -> Self {
        let mut randfloat = vec![0.0; Self::POINT_COUNT];
        for i in 0..Self::POINT_COUNT {
            randfloat[i] = crate::rtweekend::random::random_double();
        }

        let perm_x = Self::perlin_generate_perm();
        let perm_y = Self::perlin_generate_perm();
        let perm_z = Self::perlin_generate_perm();

        Self {
            //randvec,
            randfloat,
            perm_x,
            perm_y,
            perm_z,
        }
    }
    */

    pub fn new() -> Self {
        let mut randvec = vec![Vec3::default(); Self::POINT_COUNT];
        for i in 0..Self::POINT_COUNT {
            // 生成 [-1,1] 区间的随机向量并单位化
            randvec[i] = Vec3::unit_vector(Vec3::random()) * 2.0 - Vec3::new(1.0, 1.0, 1.0);
        }

        let perm_x = Self::perlin_generate_perm();
        let perm_y = Self::perlin_generate_perm();
        let perm_z = Self::perlin_generate_perm();

        Self {
            randvec,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    pub fn noise(&self, p: &Point3) -> f64 {
    let mut u = p.x - p.x.floor();
    let mut v = p.y - p.y.floor();
    let mut w = p.z - p.z.floor();
    u = u * u * (3.0 - 2.0 * u);
    v = v * v * (3.0 - 2.0 * v);
    w = w * w * (3.0 - 2.0 * w);

    let i = p.x.floor() as i32;
    let j = p.y.floor() as i32;
    let k = p.z.floor() as i32;
    let mut c = [[[Vec3::default(); 2]; 2]; 2];

    for di in 0..2 {
        for dj in 0..2 {
            for dk in 0..2 {
                c[di][dj][dk] = self.randvec[
                    (self.perm_x[((i + di as i32) & 255) as usize]
                    ^ self.perm_y[((j + dj as i32) & 255) as usize]
                    ^ self.perm_z[((k + dk as i32) & 255) as usize]) as usize
                ];
            }
        }
    }

    Self::perlin_interp(c, u, v, w)
}

    /* 最简单的噪声
    pub fn noise(&self, p: &Point3) -> f64 {
        let i = ((4.0 * p.x) as i32 & 255) as usize;
        let j = ((4.0 * p.y) as i32 & 255) as usize;
        let k = ((4.0 * p.z) as i32 & 255) as usize;

        self.randfloat[
            (self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]) as usize
        ]
    }*/

    /*第二简单的噪声
    pub fn noise(&self, p: &Point3) -> f64 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;
        let mut c = [[[0.0f64; 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.randfloat[
                        (self.perm_x[((i + di as i32) & 255) as usize]
                        ^ self.perm_y[((j + dj as i32) & 255) as usize]
                        ^ self.perm_z[((k + dk as i32) & 255) as usize]) as usize
                    ];
                }
            }
        }

        Self::trilinear_interp(c, u, v, w)
    }*/

    /*第三简单的噪声
    pub fn noise(&self, p: &Point3) -> f64 {
        let mut u = p.x - p.x.floor();
        let mut v = p.y - p.y.floor();
        let mut w = p.z - p.z.floor();
        u = u * u * (3.0 - 2.0 * u);
        v = v * v * (3.0 - 2.0 * v);
        w = w * w * (3.0 - 2.0 * w);

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;
        let mut c = [[[0.0f64; 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.randfloat[
                        (self.perm_x[((i + di as i32) & 255) as usize]
                        ^ self.perm_y[((j + dj as i32) & 255) as usize]
                        ^ self.perm_z[((k + dk as i32) & 255) as usize]) as usize
                    ];
                }
            }
        }

        Self::trilinear_interp(c, u, v, w)
    }*/

    pub fn turb(&self, p: &Point3, depth: i32) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = *p;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }

        accum.abs()
    }

    fn perlin_generate_perm() -> Vec<i32> {
        let mut p = vec![0; Self::POINT_COUNT];

        for i in 0..Self::POINT_COUNT {
            p[i] = i as i32;
        }

        Self::permute(&mut p, Self::POINT_COUNT);

        p
    }

    fn permute(p: &mut [i32], n: usize) {
        for i in (1..n).rev() {
            let target = random_int(0, i as i32) as usize;
            let tmp = p[i];
            p[i] = p[target];
            p[target] = tmp;
        }
    }

    fn trilinear_interp(c: [[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let mut accum = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    accum += (i as f64 * u + (1.0 - i as f64) * (1.0 - u))
                        * (j as f64 * v + (1.0 - j as f64) * (1.0 - v))
                        * (k as f64 * w + (1.0 - k as f64) * (1.0 - w))
                        * c[i][j][k];
                }
            }
        }

        accum
    }

    fn perlin_interp(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight_v = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += (i as f64 * uu + (1.0 - i as f64) * (1.0 - uu))
                        * (j as f64 * vv + (1.0 - j as f64) * (1.0 - vv))
                        * (k as f64 * ww + (1.0 - k as f64) * (1.0 - ww))
                        * Vec3::dot(&c[i][j][k], &weight_v);
                }
            }
        }

        accum
    }
}