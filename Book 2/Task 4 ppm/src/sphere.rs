// src/shapes/sphere.rs
use std::sync::Arc;
use crate::aabb::Aabb;
use crate::{
    hittable::{HitRecord, Hittable},
    ray::Ray,
    vec3::{Point3, Vec3},
    interval::Interval,
    material::Material,
};

pub struct Sphere {
    center: Ray,
    radius: f64,
    pub mat: Option<Arc<dyn Material + Send + Sync>>,
    bbox: Aabb,
}

impl Sphere {
    /// 静止球体
    pub fn new(static_center: Point3, radius: f64, mat: Option<Arc<dyn Material + Send + Sync>>) -> Self {
        let rvec = Vec3::new(radius, radius, radius);
        let bbox = Aabb::from_points(static_center - rvec, static_center + rvec);
        Self {
            center: Ray::new(static_center, Vec3::default(), 0.0),
            radius: radius.max(0.0),
            mat,
            bbox,
        }
    }

    /// 移动球体
    pub fn moving(center1: Point3, center2: Point3, radius: f64, mat: Option<Arc<dyn Material + Send + Sync>>) -> Self {
        let rvec = Vec3::new(radius, radius, radius);
        let ray = Ray::new(center1, center2 - center1, 0.0);
        let box1 = Aabb::from_points(ray.at(0.0) - rvec, ray.at(0.0) + rvec);
        let box2 = Aabb::from_points(ray.at(1.0) - rvec, ray.at(1.0) + rvec);
        let bbox = Aabb::surrounding_box(&box1, &box2);
        Self {
            center: ray,
            radius: radius.max(0.0),
            mat,
            bbox,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let current_center = self.center.at(r.time());
        let oc = current_center - r.origin;
        let a = r.direction.length_squared();
        let h = Vec3::dot(&r.direction, &oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = h * h - a * c;
        if discriminant < 0.0 { return false;}
        let sqrtd = discriminant.sqrt();
        
        // 寻找在有效范围内的最近根
        let mut root = (h - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) { return false;}
        }

        rec.t = root;
        rec.p = r.at(root);
        let outward_normal = (rec.p - current_center) / self.radius;
        rec.set_face_normal(r, outward_normal);
        rec.mat = self.mat.clone(); // 关键：赋值材质
        
        true
    }
    fn bounding_box(&self) -> Aabb { self.bbox }
}