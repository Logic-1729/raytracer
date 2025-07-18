use std::sync::Arc;
use crate::hittable::HittableList;
use crate::vec3::{Point3, Vec3};
use crate::material::Material;
use crate::aabb::Aabb;
use crate::ray::Ray;
use crate::interval::Interval;
use crate::hittable::{HitRecord, Hittable};

pub struct Quad {
    q: Point3,
    u: Vec3,
    v: Vec3,
    normal: Vec3,
    d: f64,
    mat: Arc<dyn Material + Send + Sync>,
    bbox: Aabb,
    area : f64,
}

impl Quad {
    pub fn new(q: Point3, u: Vec3, v: Vec3, mat: Arc<dyn Material + Send + Sync>) -> Self {
        let n = Vec3::cross(&u, &v);
        let normal = Vec3::unit_vector(n);
        let d = Vec3::dot(&normal, &q);
        let area = Vec3::cross(&u, &v).length();

        let mut quad = Self {
            q,
            u,
            v,
            normal,
            d,
            mat,
            bbox: Aabb::new_empty(),
            area,
        };
        quad.set_bounding_box();
        quad
    }

    fn set_bounding_box(&mut self) {
        // Compute the bounding box of all four vertices.
        let bbox_diagonal1 = Aabb::from_points(self.q, self.q + self.u + self.v);
        let bbox_diagonal2 = Aabb::from_points(self.q + self.u, self.q + self.v);
        self.bbox = Aabb::surrounding_box(&bbox_diagonal1, &bbox_diagonal2);
    }

    fn is_interior(&self, a: f64, b: f64, rec: &mut HitRecord) -> bool {
        let unit_interval = Interval::new(0.0, 1.0);
        // 如果不在范围内，返回 false
        if !unit_interval.contains(a) || !unit_interval.contains(b) {
            return false;
        }
        rec.u = a;
        rec.v = b;
        true
    }
}

impl Hittable for Quad {
    fn bounding_box(&self) -> Aabb {
        self.bbox
    }

    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let denom = Vec3::dot(&self.normal, &r.direction());

        // No hit if the ray is parallel to the plane.
        if denom.abs() < 1e-8 {
            return false;
        }

        // Return false if the hit point parameter t is outside the ray interval.
        let t = (self.d - Vec3::dot(&self.normal, &r.origin())) / denom;
        if !ray_t.contains(t) {
            return false;
        }

        // 判断交点是否在四边形内部
        let intersection = r.at(t);
        let planar_hitpt_vector = intersection - self.q;
        let w = (Vec3::cross(&self.u,&self.v)) / Vec3::dot(&(Vec3::cross(&self.u,&self.v)),&(Vec3::cross(&self.u,&self.v)));
        let alpha = Vec3::dot(&w,&(Vec3::cross(&planar_hitpt_vector,&self.v)));
        let beta = Vec3::dot(&w,&(Vec3::cross(&self.u,&planar_hitpt_vector)));

        if !self.is_interior(alpha, beta, rec) {
            return false;
        }

        // 命中，设置 hit record
        rec.t = t;
        rec.p = intersection;
        rec.mat = Some(self.mat.clone());
        rec.set_face_normal(r, self.normal);

        true
    }
    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let ray = Ray::new(*origin, *direction, 0.0);
        let mut rec = HitRecord::default();
        if self.hit(&ray, Interval::new(0.001, f64::INFINITY), &mut rec) {
            let distance_squared = rec.t * rec.t * direction.length_squared();
            let cosine = Vec3::dot(direction, &rec.normal).abs() / direction.length();
            return distance_squared / (cosine * self.area);
        }
        0.0
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        use crate::rtweekend::random::random_double;
        let random_point = self.q
            + self.u * random_double()
            + self.v * random_double();
        random_point - *origin
    }
}

pub fn make_box(a: Point3, b: Point3, mat: Arc<dyn Material + Send + Sync>) -> Arc<HittableList> {
    let mut sides = HittableList::new();

    let min = Point3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
    let max = Point3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

    let dx = Vec3::new(max.x - min.x, 0.0, 0.0);
    let dy = Vec3::new(0.0, max.y - min.y, 0.0);
    let dz = Vec3::new(0.0, 0.0, max.z - min.z);

    // front
    sides.add(Arc::new(Quad::new(
        Point3::new(min.x, min.y, max.z),
        dx,
        dy,
        mat.clone(),
    )));
    // right
    sides.add(Arc::new(Quad::new(
        Point3::new(max.x, min.y, max.z),
        -dz,
        dy,
        mat.clone(),
    )));
    // back
    sides.add(Arc::new(Quad::new(
        Point3::new(max.x, min.y, min.z),
        -dx,
        dy,
        mat.clone(),
    )));
    // left
    sides.add(Arc::new(Quad::new(
        Point3::new(min.x, min.y, min.z),
        dz,
        dy,
        mat.clone(),
    )));
    // top
    sides.add(Arc::new(Quad::new(
        Point3::new(min.x, max.y, max.z),
        dx,
        -dz,
        mat.clone(),
    )));
    // bottom
    sides.add(Arc::new(Quad::new(
        Point3::new(min.x, min.y, min.z),
        dx,
        dz,
        mat.clone(),
    )));

    Arc::new(sides)
}