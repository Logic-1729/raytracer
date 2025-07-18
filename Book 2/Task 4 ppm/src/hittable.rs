// src/hittable.rs
use std::sync::Arc;
use crate::aabb::*;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};
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
            p: Vec3::default(),
            normal: Vec3::default(),
            mat: None,
            t: 0.0,
            front_face: false,
        }
    }

    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = Vec3::dot(&r.direction, &outward_normal) < 0.0;
        self.normal = if self.front_face { outward_normal } else { -outward_normal };
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
    fn bounding_box(&self) -> Aabb;
}

pub struct HittableList { pub objects: Vec<Arc<dyn Hittable + Send + Sync>>,pub bbox: Aabb,}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            bbox: Aabb::new_empty(),
        }
    }

    pub fn default() -> Self {
        Self::new()
    }

    pub fn with_object(object: Arc<dyn Hittable + Send + Sync>) -> Self {
        let mut objects = Vec::new();
        let bbox = object.bounding_box();
        objects.push(object);
        Self { objects, bbox }
    }

    pub fn add(&mut self, object: Arc<dyn Hittable + Send + Sync>) {
        self.bbox = if self.objects.is_empty() {
            object.bounding_box()
        } else {
            Aabb::surrounding_box(&self.bbox, &object.bounding_box())
        };
        self.objects.push(object);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.bbox = Aabb::new_empty();
    }
}

impl Hittable for HittableList {
    fn hit(
        &self,
        r: &Ray,
        ray_t: Interval,
        rec: &mut HitRecord,
    ) -> bool {
        let mut temp_rec = HitRecord::default();
        let mut hit_anything = false;
        let mut closest_so_far = ray_t.max;

        for object in &self.objects {
            if object.hit(r, Interval::new(ray_t.min, closest_so_far), &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                *rec = temp_rec.clone();
            }
        }
        hit_anything
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}