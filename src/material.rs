
// 数字材质：数字部分发光，背景为石板色
pub struct NumberMaterial {
    pub base: std::sync::Arc<crate::texture::ImageTexture>,
    pub slab_color: crate::Color,
    pub emit_color: crate::Color,
}

impl NumberMaterial {
    pub fn new(base: std::sync::Arc<crate::texture::ImageTexture>, slab_color: crate::Color, emit_color: crate::Color) -> Self {
        Self { base, slab_color, emit_color }
    }
    // 判断是否为数字（红色），允许一定容差
    fn is_digit(&self, c: &crate::Color) -> bool {
        // 只要红色分量远大于其他分量即可
        c.x > 0.5 && c.x - c.y > 0.3 && c.x - c.z > 0.3
    }
    // 判断是否为背景（近黑、近灰、近蓝、近透明合成色等）
    fn is_bg(&self, c: &crate::Color) -> bool {
        // 允许较宽容差，兼容png背景合成色（如淡蓝、淡灰等）
        let avg = (c.x + c.y + c.z) / 3.0;
        let maxc = c.x.max(c.y).max(c.z);
        let minc = c.x.min(c.y).min(c.z);
        // 亮度低或接近灰/蓝/黑
        (avg < 0.55 && (maxc - minc) < 0.15) || (c.x < 0.35 && c.y < 0.35 && c.z > 0.25)
    }
}

impl Material for NumberMaterial {
    fn scatter(&self, r_in: &crate::Ray, rec: &crate::hittable::HitRecord, srec: &mut ScatterRecord, rng: &mut rand::rngs::ThreadRng) -> bool {
        let u = rec.u;
        let v = rec.v;
        let c = self.base.value(u, v, &rec.p);
        if self.is_digit(&c) {
            // 红色数字：不散射
            false
        } else {
            // 非红色部分全部为黑色（不反射）
            false
        }
    }
    fn emitted(&self, u: f64, v: f64, p: &crate::vec3::Vec3) -> crate::Color {
        let c = self.base.value(u, v, p);
        if self.is_digit(&c) {
            self.emit_color
        } else {
            // 非红色部分全部为黑色
            crate::Color::new(0.0, 0.0, 0.0)
        }
    }
}
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


    // C++: virtual bool scatter(const ray& r_in, const hit_record& rec, scatter_record& srec) const { return false; }
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        srec: &mut ScatterRecord,
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
    pub albedo: f64,
}

impl Lambertian {
    pub fn from_color(albedo: Color) -> Self {
        Lambertian {
            tex: Arc::new(SolidColor::new(albedo)),
            albedo: 1.0,
        }
    }

    pub fn from_texture(tex: Arc<dyn Texture + Send + Sync>) -> Self {
        Lambertian { tex, albedo: 1.0 }
    }

    pub fn from_texture_with_albedo(tex: Arc<dyn Texture + Send + Sync>, albedo: f64) -> Self {
        Lambertian { tex, albedo }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        _r_in: &Ray,
        rec: &HitRecord,
        srec: &mut ScatterRecord,
        _rng: &mut ThreadRng,
    ) -> bool {
        use crate::pdf::CosinePdf;
        use std::sync::Arc;
        srec.attenuation = self.tex.value(rec.u, rec.v, &rec.p) * self.albedo;
        srec.pdf_ptr = Some(Arc::new(CosinePdf::new(rec.normal)));
        srec.skip_pdf = false;
        srec.skip_pdf_ray = None;
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cos_theta = Vec3::dot(&rec.normal, &Vec3::unit_vector(scattered.direction));
        if cos_theta < 0.0 {
            0.0
        } else {
            cos_theta / std::f64::consts::PI
        }
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
    // C++: bool scatter(const ray& r_in, const hit_record& rec, scatter_record& srec) const override
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        srec: &mut ScatterRecord,
        rng: &mut ThreadRng,
    ) -> bool {
        let mut reflected = Vec3::reflect(&r_in.direction, &rec.normal);
        reflected = Vec3::unit_vector(reflected) + (self.fuzz * random_unit_vector_with_rng(rng));

        srec.attenuation = self.albedo;
        srec.pdf_ptr = None;
        srec.skip_pdf = true;
        srec.skip_pdf_ray = Some(Ray::new(rec.p, reflected, r_in.time()));

        true
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
    // C++: bool scatter(const ray& r_in, const hit_record& rec, scatter_record& srec) const override
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        srec: &mut ScatterRecord,
        _rng: &mut ThreadRng,
    ) -> bool {
        srec.attenuation = Color::new(1.0, 1.0, 1.0);
        srec.pdf_ptr = None;
        srec.skip_pdf = true;
        let ri = if rec.front_face { 1.0 / self.refraction_index } else { self.refraction_index };
        let unit_direction = Vec3::unit_vector(r_in.direction);
        let cos_theta = Vec3::dot(&-unit_direction, &rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = ri * sin_theta > 1.0;
        let direction = if cannot_refract || Dielectric::reflectance(cos_theta, ri) > random_double() {
            Vec3::reflect(&unit_direction, &rec.normal)
        } else {
            Vec3::refract(&unit_direction, &rec.normal, ri)
        };
        srec.skip_pdf_ray = Some(Ray::new(rec.p, direction, r_in.time()));
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
    // C++: bool scatter(const ray& r_in, const hit_record& rec, scatter_record& srec) const override
    fn scatter(
        &self,
        _r_in: &Ray,
        _rec: &HitRecord,
        _srec: &mut ScatterRecord,
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
    fn scatter(
        &self,
        _r_in: &Ray,
        rec: &HitRecord,
        srec: &mut ScatterRecord,
        _rng: &mut ThreadRng,
    ) -> bool {
        use crate::pdf::SpherePdf;
        use std::sync::Arc;
        srec.attenuation = self.tex.value(rec.u, rec.v, &rec.p);
        srec.pdf_ptr = Some(Arc::new(SpherePdf::new()));
        srec.skip_pdf = false;
        srec.skip_pdf_ray = None;
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 / (4.0 * std::f64::consts::PI)
    }
}

pub struct ScatterRecord {
    pub attenuation: Color,
    pub pdf_ptr: Option<std::sync::Arc<dyn crate::pdf::Pdf + Send + Sync>>,
    pub skip_pdf: bool,
    pub skip_pdf_ray: Option<crate::Ray>,
}