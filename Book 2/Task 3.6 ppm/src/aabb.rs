use crate::interval::Interval;
use crate::vec3::{Point3, Vec3};
use crate::ray::Ray;

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl Aabb {
    /// 默认AABB是空的，因为Interval默认是空的
    pub fn new_empty() -> Self {
        Self {
            x: Interval::empty(),
            y: Interval::empty(),
            z: Interval::empty(),
        }
    }

    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        Self { x, y, z }
    }

    pub fn from_points(a: Point3, b: Point3) -> Self {
        let x = if a.x <= b.x {
            Interval::new(a.x, b.x)
        } else {
            Interval::new(b.x, a.x)
        };
        let y = if a.y <= b.y {
            Interval::new(a.y, b.y)
        } else {
            Interval::new(b.y, a.y)
        };
        let z = if a.z <= b.z {
            Interval::new(a.z, b.z)
        } else {
            Interval::new(b.z, a.z)
        };
        Self { x, y, z }
    }

    pub fn surrounding_box(box0: &Aabb, box1: &Aabb) -> Self {
        let x = Interval::from_two(&box0.x, &box1.x);
        let y = Interval::from_two(&box0.y, &box1.y);
        let z = Interval::from_two(&box0.z, &box1.z);
        Self { x, y, z }
    }

    pub fn axis_interval(&self, n: usize) -> &Interval {
        match n {
            1 => &self.y,
            2 => &self.z,
            _ => &self.x,
        }
    }

    pub fn hit(&self, r: &Ray, mut ray_t: Interval) -> bool {
        let ray_orig = r.origin();
        let ray_dir = r.direction();

        for axis in 0..3 {
            let ax = self.axis_interval(axis);
            let (orig, dir) = match axis {
                0 => (ray_orig.x, ray_dir.x),
                1 => (ray_orig.y, ray_dir.y),
                2 => (ray_orig.z, ray_dir.z),
                _ => unreachable!(),
            };
            let adinv = 1.0 / dir;

            let t0 = (ax.min - orig) * adinv;
            let t1 = (ax.max - orig) * adinv;

            let (t0, t1) = if t0 < t1 { (t0, t1) } else { (t1, t0) };

            if t0 > ray_t.min { ray_t.min = t0; }
            if t1 < ray_t.max { ray_t.max = t1; }
            if ray_t.max <= ray_t.min { return false;}
        }
        true
    }
}