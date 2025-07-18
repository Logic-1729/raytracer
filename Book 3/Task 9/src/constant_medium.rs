use std::sync::Arc;
use crate::material::Isotropic;
use crate::hittable::{Hittable, HitRecord};
use crate::material::Material;
use crate::texture::Texture;
use crate::vec3::{Vec3, Point3};
use crate::ray::Ray;
use crate::interval::Interval;
use crate::aabb::Aabb;
use crate::rtweekend::random::*;
use crate::rtweekend::INFINITY;

pub struct ConstantMedium {
    boundary: Arc<dyn Hittable + Send + Sync>,
    neg_inv_density: f64,
    phase_function: Arc<dyn Material + Send + Sync>,
}

impl ConstantMedium {
    pub fn new_with_texture(
        boundary: Arc<dyn Hittable + Send + Sync>,
        density: f64,
        tex: Arc<dyn Texture + Send + Sync>,
    ) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Arc::new(Isotropic::new_with_texture(tex)),
        }
    }

    pub fn new_with_color(
        boundary: Arc<dyn Hittable + Send + Sync>,
        density: f64,
        albedo: Vec3,
    ) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Arc::new(Isotropic::new_with_color(albedo)),
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(
        &self,
        r: &Ray,
        ray_t: Interval,
        rec: &mut HitRecord,
    ) -> bool {
        let mut rec1 = HitRecord::default();
        let mut rec2 = HitRecord::default();

        if !self.boundary.hit(r, Interval::universe(), &mut rec1) {
            return false;
        }

        if !self.boundary.hit(r, Interval::new(rec1.t + 0.0001, INFINITY), &mut rec2) {
            return false;
        }

        let mut t1 = rec1.t;
        let mut t2 = rec2.t;

        if t1 < ray_t.min { t1 = ray_t.min; }
        if t2 > ray_t.max { t2 = ray_t.max; }

        if t1 >= t2 {
            return false;
        }

        if t1 < 0.0 {
            t1 = 0.0;
        }

        let ray_length = r.direction.length();
        let distance_inside_boundary = (t2 - t1) * ray_length;
        let hit_distance = self.neg_inv_density * random_double().ln();

        if hit_distance > distance_inside_boundary {
            return false;
        }

        rec.t = t1 + hit_distance / ray_length;
        rec.p = r.at(rec.t);

        rec.normal = Vec3::new(1.0, 0.0, 0.0); // 任意
        rec.front_face = true;                  // 任意
        rec.mat = Some(self.phase_function.clone());

        true
    }

    fn bounding_box(&self) -> Aabb {
        self.boundary.bounding_box()
    }
}