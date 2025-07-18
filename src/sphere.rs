use std::sync::Arc;
use crate::rtweekend::random::random_double;
use crate::aabb::Aabb;
use crate::onb::Onb;
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
    /// 计算从给定点(origin)沿给定方向(direction)射向球体的概率密度函数值
    pub fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        // 仅适用于静止球体
        let mut rec = HitRecord::default();
        let ray = Ray::new(*origin, *direction, 0.0);
        if !self.hit(&ray, Interval::new(0.001, f64::INFINITY), &mut rec) {
            return 0.0;
        }
        let dist_squared = (self.center.at(0.0) - *origin).length_squared();
        let radius2 = self.radius * self.radius;
        if dist_squared <= radius2 {
            return 0.0; // 避免 sqrt 负数
        }
        let cos_theta_max = (1.0 - radius2 / dist_squared).sqrt();
        let solid_angle = 2.0 * std::f64::consts::PI * (1.0 - cos_theta_max);
        1.0 / solid_angle
    }

    /// 从给定点(origin)随机采样射向球体的方向
    pub fn random(&self, origin: &Point3) -> Vec3 {
        let direction = self.center.at(0.0) - *origin;
        let distance_squared = direction.length_squared();
        let uvw = Onb::new(&direction);
        uvw.transform(&random_to_sphere(self.radius, distance_squared))
    }
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

        // 修改这里，直接传递 rec.u, rec.v 的可变引用给 get_sphere_uv
        get_sphere_uv(&outward_normal, &mut rec.u, &mut rec.v);

        rec.mat = self.mat.clone(); // 关键：赋值材质
        
        true
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}

/// 计算球面上的点的纹理坐标 (u, v)，参数为 &Point3, &mut u, &mut v
pub fn get_sphere_uv(p: &Point3, u: &mut f64, v: &mut f64) {
    let theta = (-p.y).acos();
    let phi = (-p.z).atan2(p.x) + std::f64::consts::PI;
    *u = (phi / (2.0 * std::f64::consts::PI)).clamp(0.0, 1.0);
    *v = (theta / std::f64::consts::PI).clamp(0.0, 1.0);
}

/// 球面均匀采样方向（用于 importance sampling PDF）
fn random_to_sphere(radius: f64, distance_squared: f64) -> Vec3 {
    let r1 = random_double();
    let r2 = random_double();
    let radius2 = radius * radius;
    if distance_squared <= radius2 {
        // 防止 sqrt 负数
        return Vec3::new(0.0, 0.0, 1.0);
    }
    let z = 1.0 + r2 * ((1.0 - radius2 / distance_squared).sqrt() - 1.0);
    let phi = 2.0 * std::f64::consts::PI * r1;
    let sqrt_val: f64 = (1.0 - z * z).sqrt();
    let x = phi.cos() * sqrt_val;
    let y = phi.sin() * sqrt_val;
    Vec3::new(x, y, z)
}