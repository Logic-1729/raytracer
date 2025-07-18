use crate::{Ray, Color};
use crate::vec3::Vec3;
use crate::texture::*;
use crate::hittable::{HitRecord, Hittable};
use crate::rtweekend::random::{random_double, random_unit_vector_with_rng};
use std::sync::Arc;
use rand::{rngs::ThreadRng, Rng};

pub trait Material {

    // 判断该材质是否为发光体
    fn is_emissive(&self) -> bool {
        false
    }

    // 获取材质的发光颜色（如有）
    fn emission_color(&self, _u: f64, _v: f64, _p: &Vec3) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }


    // C++: virtual bool scatter(const ray& r_in, const hit_record& rec, color& attenuation, ray& scattered, double& pdf) const { return false; }
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
        pdf: &mut f64,
        rng: &mut ThreadRng,
    ) -> bool {
        false
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        0.0
    }

    fn emitted(&self, u: f64, v: f64, p: &Vec3) -> Color {
        self.emission_color(u, v, p)
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
    // C++: bool scatter(const ray& r_in, const hit_record& rec, color& attenuation, ray& scattered, double& pdf) const override
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
        pdf: &mut f64,
        rng: &mut ThreadRng,
    ) -> bool {
        // 构建正交基于法线
        let uvw = crate::onb::Onb::new(&rec.normal);
        // 采样余弦加权方向
        let scatter_direction = uvw.transform(&random_unit_vector_with_rng(rng));
        *scattered = Ray::new(rec.p, Vec3::unit_vector(scatter_direction), r_in.time());
        *attenuation = self.tex.value(rec.u, rec.v, &rec.p);
        *pdf = Vec3::dot(&uvw.w(), &scattered.direction) / std::f64::consts::PI;
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 / (2.0 * std::f64::consts::PI)
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
    // C++: bool scatter(const ray& r_in, const hit_record& rec, color& attenuation, ray& scattered, double& pdf) const override
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
        pdf: &mut f64,
        rng: &mut ThreadRng,
    ) -> bool {
        let reflected = Vec3::reflect(&Vec3::unit_vector(r_in.direction), &rec.normal);
        let scattered_direction = reflected + random_unit_vector_with_rng(rng) * self.fuzz;
        *scattered = Ray::new(rec.p, scattered_direction, r_in.time());
        *attenuation = self.albedo;
        *pdf = 1.0; // 金属为镜面反射，pdf 可设为 1 或 0，具体依赖于采样策略
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
    // C++: bool scatter(const ray& r_in, const hit_record& rec, color& attenuation, ray& scattered, double& pdf) const override
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
        pdf: &mut f64,
        rng: &mut ThreadRng,
    ) -> bool {
        *attenuation = Color::new(1.0, 1.0, 1.0);
        let ri = if rec.front_face { 1.0 / self.refraction_index } else { self.refraction_index };
        let unit_direction = Vec3::unit_vector(r_in.direction);
        let cos_theta = Vec3::dot(&-unit_direction, &rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = ri * sin_theta > 1.0;
        let direction = if cannot_refract || Dielectric::reflectance(cos_theta, ri) > rng.gen_range(0.0f64..1.0f64) {
            Vec3::reflect(&unit_direction, &rec.normal)
        } else {
            Vec3::refract(&unit_direction, &rec.normal, ri)
        };
        *scattered = Ray::new(rec.p, direction, r_in.time());
        *pdf = 1.0; // 玻璃为镜面反射/折射，pdf 可设为 1 或 0，具体依赖于采样策略
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
    // C++: bool scatter(const ray& r_in, const hit_record& rec, color& attenuation, ray& scattered, double& pdf) const override
    fn scatter(
        &self,
        _r_in: &Ray,
        _rec: &HitRecord,
        _attenuation: &mut Color,
        _scattered: &mut Ray,
        _pdf: &mut f64,
        _rng: &mut ThreadRng,
    ) -> bool {
        false
    }

    // 保留 trait 默认实现，不在此重复定义

    fn emission_color(&self, u: f64, v: f64, p: &Vec3) -> Color {
        self.tex.value(u, v, p)
    }

    fn is_emissive(&self) -> bool {
        true
    }

    // emitted_with_rec 不是 trait 的一部分，移除实现，若需要请在外部函数中实现辅助逻辑
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
    // C++: bool scatter(const ray& r_in, const hit_record& rec, color& attenuation, ray& scattered, double& pdf) const override
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
        pdf: &mut f64,
        rng: &mut ThreadRng,
    ) -> bool {
        *scattered = Ray::new(rec.p, random_unit_vector_with_rng(rng), r_in.time());
        *attenuation = self.tex.value(rec.u, rec.v, &rec.p);
        *pdf = 1.0 / (4.0 * std::f64::consts::PI);
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 / (4.0 * std::f64::consts::PI)
    }
}