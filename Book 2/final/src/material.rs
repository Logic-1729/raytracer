use crate::{Ray, Color};
use crate::vec3::Vec3;
use crate::texture::*;
use crate::hittable::{HitRecord, Hittable};
use crate::rtweekend::random::{random_double, random_unit_vector_with_rng};
use std::sync::Arc;
use rand::{rngs::ThreadRng, Rng};

pub trait Material {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
        rng: &mut ThreadRng,
    ) -> bool {
        false
    }

    fn emitted(&self, _u: f64, _v: f64, _p: &Vec3) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }
}

pub struct Lambertian {
    pub tex: Arc<dyn Texture + Send + Sync>,
}

impl Lambertian {
    pub fn from_color(albedo: Color) -> Self {
        Lambertian {
            tex: Arc::new(SolidColor::new(albedo)),
        }
    }

    pub fn from_texture(tex: Arc<dyn Texture + Send + Sync>) -> Self {
        Lambertian { tex }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
        rng: &mut ThreadRng,
    ) -> bool {
        let mut scatter_direction = rec.normal + random_unit_vector_with_rng(rng);

        // 处理退化方向
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        *scattered = Ray::new(rec.p, scatter_direction, r_in.time());
        *attenuation = self.tex.value(rec.u, rec.v, &rec.p);
        true
    }
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Metal {
            albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        }
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
        rng: &mut ThreadRng,
    ) -> bool {
        let reflected = Vec3::reflect(&Vec3::unit_vector(r_in.direction), &rec.normal);
        let scattered_direction = reflected + random_unit_vector_with_rng(rng) * self.fuzz;
        *scattered = Ray::new(rec.p, scattered_direction,r_in.time());
        *attenuation = self.albedo;
        Vec3::dot(&scattered.direction, &rec.normal) > 0.0
    }
}

pub struct Dielectric { pub refraction_index: f64,}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Self { Dielectric { refraction_index }}

    // Schlick 近似反射率
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
        rng: &mut ThreadRng,
    ) -> bool {
        *attenuation = Color::new(1.0, 1.0, 1.0);
        let ri = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = Vec3::unit_vector(r_in.direction);
        let cos_theta = Vec3::dot(&-unit_direction, &rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;
        let direction = if cannot_refract || Dielectric::reflectance(cos_theta, ri) > rng.gen_range(0.0f64..1.0f64) {
            Vec3::reflect(&unit_direction, &rec.normal)
        } else {
            Vec3::refract(&unit_direction, &rec.normal, ri)
        };

        *scattered = Ray::new(rec.p, direction,r_in.time());
        true
    }
}

pub struct DiffuseLight {
    pub tex: Arc<dyn Texture + Send + Sync>,
}

impl DiffuseLight {
    pub fn from_texture(tex: Arc<dyn Texture + Send + Sync>) -> Self {
        DiffuseLight { tex }
    }

    pub fn from_color(emit: Color) -> Self {
        DiffuseLight {
            tex: Arc::new(SolidColor::new(emit)),
        }
    }
}

impl Material for DiffuseLight {
    fn scatter(
        &self,
        _r_in: &Ray,
        _rec: &HitRecord,
        _attenuation: &mut Color,
        _scattered: &mut Ray,
        _rng: &mut ThreadRng,
    ) -> bool {
        false
    }

    fn emitted(&self, u: f64, v: f64, p: &Vec3) -> Color {
        self.tex.value(u, v, p)
    }
}

pub struct Isotropic {
    pub tex: Arc<dyn Texture + Send + Sync>,
}

impl Isotropic {
    pub fn new_with_color(albedo: Color) -> Self {
        Isotropic {
            tex: Arc::new(SolidColor::new(albedo)),
        }
    }

    pub fn new_with_texture(tex: Arc<dyn Texture + Send + Sync>) -> Self {
        Isotropic { tex }
    }
}

impl Material for Isotropic {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
        rng: &mut ThreadRng,
    ) -> bool {
        *scattered = Ray::new(rec.p, random_unit_vector_with_rng(rng), r_in.time());
        *attenuation = self.tex.value(rec.u, rec.v, &rec.p);
        true
    }
}