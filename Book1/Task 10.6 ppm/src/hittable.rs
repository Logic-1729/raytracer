// src/hittable.rs
use crate::vec3::{Point3, Vec3};
use crate::ray::Ray;
use std::sync::Arc;
use crate::material::Material;
use crate::interval::Interval;

#[derive(Clone)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Option<Arc<dyn Material + Send + Sync>>,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn default() -> Self {
        HitRecord {
            p: Vec3::zero(),
            normal: Vec3::zero(),
            mat: None,
            t: 0.0,
            front_face: false,
        }
    }

    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = Vec3::dot(&r.direction, &outward_normal) < 0.0;
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