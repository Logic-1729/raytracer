use crate::vec3::{Point3, Vec3};
use crate::ray::Ray;
use crate::interval::Interval;
use crate::aabb::Aabb;
use crate::material::Material;
use crate::hittable::{HitRecord, Hittable};
use std::sync::Arc;

pub struct XZRect {
    pub x0: f64,
    pub x1: f64,
    pub z0: f64,
    pub z1: f64,
    pub k: f64, // y = k
    pub mat: Arc<dyn Material + Send + Sync>,
}

impl XZRect {
    pub fn new(
        x0: f64,
        x1: f64,
        z0: f64,
        z1: f64,
        k: f64,
        mat: Arc<dyn Material + Send + Sync>,
    ) -> Self {
        Self { x0, x1, z0, z1, k, mat }
    }
}

impl Hittable for XZRect {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let t = (self.k - r.origin.y) / r.direction.y;
        if !ray_t.surrounds(t) { return false; }
        let x = r.origin.x + t * r.direction.x;
        let z = r.origin.z + t * r.direction.z;
        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 { return false; }

        rec.t = t;
        rec.p = Point3::new(x, self.k, z);
        let outward_normal = Vec3::new(0.0, 1.0, 0.0);
        rec.set_face_normal(r, outward_normal);
        rec.u = (x - self.x0) / (self.x1 - self.x0);
        rec.v = (z - self.z0) / (self.z1 - self.z0);
        rec.mat = Some(self.mat.clone());
        true
    }

    fn bounding_box(&self) -> Aabb {
        Aabb::from_points(
            Point3::new(self.x0, self.k - 0.0001, self.z0),
            Point3::new(self.x1, self.k + 0.0001, self.z1),
        )
    }
}