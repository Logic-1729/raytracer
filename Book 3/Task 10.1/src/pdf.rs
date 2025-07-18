
use crate::vec3::Vec3;
use crate::onb::Onb;
use crate::rtweekend::random::random_unit_vector_with_rng;
use rand::Rng;
use rand::rngs::ThreadRng;
use std::f64::consts::PI;

/// PDF trait
pub trait Pdf: Send + Sync {
    fn value(&self, direction: &Vec3) -> f64;
    fn generate(&self, rng: &mut ThreadRng) -> Vec3;
}

/// 均匀球面 PDF
pub struct SpherePdf;

impl Pdf for SpherePdf {
    fn value(&self, _direction: &Vec3) -> f64 {
        1.0 / (4.0 * PI)
    }
    fn generate(&self, rng: &mut ThreadRng) -> Vec3 {
        random_unit_vector_with_rng(rng)
    }
}

/// 余弦加权 PDF
pub struct CosinePdf {
    uvw: Onb,
}

impl CosinePdf {
    pub fn new(w: &Vec3) -> Self {
        Self { uvw: Onb::new(w) }
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: &Vec3) -> f64 {
        let cosine_theta = Vec3::dot(&Vec3::unit_vector(*direction), &self.uvw.w());
        if cosine_theta <= 0.0 {
            0.0
        } else {
            cosine_theta / PI
        }
    }
    fn generate(&self, rng: &mut ThreadRng) -> Vec3 {
        self.uvw.transform(&random_cosine_direction_with_rng(rng))
    }
}

// 余弦加权半球采样
fn random_cosine_direction_with_rng(rng: &mut ThreadRng) -> Vec3 {
    let r1: f64 = rng.gen();
    let r2: f64 = rng.gen();
    let z = (1.0 - r2).sqrt();
    let phi = 2.0 * PI * r1;
    let x = phi.cos() * r2.sqrt();
    let y = phi.sin() * r2.sqrt();
    Vec3::new(x, y, z)
}