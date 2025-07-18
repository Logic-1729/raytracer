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
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn default() -> Self {
        HitRecord {
            p: Vec3::default(),
            normal: Vec3::default(),
            mat: None,
            t: 0.0,
            u: 0.0,
            v: 0.0,
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
    
    // 允许 addlist 支持 Arc<HittableList>
    pub fn addlist<T: std::ops::Deref<Target = HittableList>>(&mut self, list: T) {
        for obj in &list.objects {
            self.objects.push(obj.clone());
        }
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

pub struct Translate {
    object: Arc<dyn Hittable + Send + Sync>,
    offset: Vec3,
    bbox: Aabb,
}

impl Translate {
    pub fn new(object: Arc<dyn Hittable + Send + Sync>, offset: Vec3) -> Self {
        let bbox = object.bounding_box() + offset;
        Self { object, offset, bbox }
    }
}

impl Hittable for Translate {
    fn hit(
        &self,
        r: &Ray,
        ray_t: Interval,
        rec: &mut HitRecord,
    ) -> bool {
        // 将射线原点向后平移 offset
        let moved_r = Ray::new(r.origin - self.offset, r.direction, r.time());

        // 判断平移后的射线是否与物体相交
        if !self.object.hit(&moved_r, ray_t, rec) {
            return false;
        }

        // 将交点向前平移 offset
        rec.p += self.offset;

        true
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}

pub struct RotateY {
    object: Arc<dyn Hittable + Send + Sync>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Aabb,
}

impl RotateY {
    pub fn new(object: Arc<dyn Hittable + Send + Sync>, angle: f64) -> Self {
        let radians = angle.to_radians();
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        let bbox0 = object.bounding_box();

        let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for i in 0..2 {
            let x = if i == 1 { bbox0.x.max } else { bbox0.x.min };
            for j in 0..2 {
                let y = if j == 1 { bbox0.y.max } else { bbox0.y.min };
                for k in 0..2 {
                    let z = if k == 1 { bbox0.z.max } else { bbox0.z.min };

                    let newx = cos_theta * x + sin_theta * z;
                    let newz = -sin_theta * x + cos_theta * z;
                    let tester = Vec3::new(newx, y, newz);

                    min.x = min.x.min(tester.x);
                    min.y = min.y.min(tester.y);
                    min.z = min.z.min(tester.z);

                    max.x = max.x.max(tester.x);
                    max.y = max.y.max(tester.y);
                    max.z = max.z.max(tester.z);
                }
            }
        }

        let bbox = Aabb::from_points(min, max);

        Self { object, sin_theta, cos_theta, bbox }
    }
}

impl Hittable for RotateY {
    fn hit(
        &self,
        r: &Ray,
        ray_t: Interval,
        rec: &mut HitRecord,
    ) -> bool {
        // 世界空间 -> 物体空间
        let origin = Point3::new(
            self.cos_theta * r.origin.x - self.sin_theta * r.origin.z,
            r.origin.y,
            self.sin_theta * r.origin.x + self.cos_theta * r.origin.z,
        );
        let direction = Vec3::new(
            self.cos_theta * r.direction.x - self.sin_theta * r.direction.z,
            r.direction.y,
            self.sin_theta * r.direction.x + self.cos_theta * r.direction.z,
        );
        let rotated_r = Ray::new(origin, direction, r.time());

        if !self.object.hit(&rotated_r, ray_t, rec) {
            return false;
        }

        // 物体空间 -> 世界空间
        let p = Point3::new(
            self.cos_theta * rec.p.x + self.sin_theta * rec.p.z,
            rec.p.y,
            -self.sin_theta * rec.p.x + self.cos_theta * rec.p.z,
        );
        let normal = Vec3::new(
            self.cos_theta * rec.normal.x + self.sin_theta * rec.normal.z,
            rec.normal.y,
            -self.sin_theta * rec.normal.x + self.cos_theta * rec.normal.z,
        );

        rec.p = p;
        rec.normal = normal;

        true
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}



