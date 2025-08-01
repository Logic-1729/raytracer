// src/hittable.rs
use crate::ray::Ray;
use crate::interval::Interval;
use crate::vec3::{Point3, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
    pub front_face: bool, // 新增字段，表示是否正面击中
}

impl HitRecord {
    pub fn default() -> Self {
        HitRecord {
            p: Vec3::zero(),
            normal: Vec3::zero(),
            t: 0.0,
            front_face: false,
        }
    }
    // 设置法线并确定是否正面击中s
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = Vec3::dot(&r.direction,&outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

/// 可命中物体特质（Trait）
pub trait Hittable: Send + Sync {
    fn hit(
        &self,
        r: &Ray,
        ray_t: Interval,
        rec: &mut HitRecord,
    ) -> bool;
}