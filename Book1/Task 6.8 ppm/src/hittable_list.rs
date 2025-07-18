// src/hittable_list.rs
use std::sync::Arc;
use crate::{
    hittable::{HitRecord, Hittable},
    ray::Ray,
};

/// 可命中物体列表
#[derive(Default)]
pub struct HittableList {
    objects: Vec<Arc<dyn Hittable + Send + Sync>>,
}

impl HittableList {
    /// 创建空列表
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建包含单个物体的列表
    pub fn with_object(object: Arc<dyn Hittable + Send + Sync>) -> Self {
        let mut list = Self::new();
        list.add(object);
        list
    }

    /// 清空列表
    pub fn clear(&mut self) {
        self.objects.clear();
    }

    /// 添加物体
    pub fn add(&mut self, object: Arc<dyn Hittable + Send + Sync>) {
        self.objects.push(object);
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64) -> Option<HitRecord> {
        let mut closest_hit = None;
        let mut closest_so_far = ray_tmax;

        for object in &self.objects {
            if let Some(hit) = object.hit(r, ray_tmin, closest_so_far) {
                closest_so_far = hit.t;
                closest_hit = Some(hit);
            }
        }

        closest_hit
    }
}