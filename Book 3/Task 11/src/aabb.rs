use crate::interval::Interval;
use crate::vec3::{Point3, Vec3};
use crate::ray::Ray;
use std::ops::Add;

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl Aabb {
    pub const EMPTY: Aabb = Aabb {
        x: Interval { min: f64::INFINITY, max: f64::NEG_INFINITY },
        y: Interval { min: f64::INFINITY, max: f64::NEG_INFINITY },
        z: Interval { min: f64::INFINITY, max: f64::NEG_INFINITY },
    };
    pub const UNIVERSE: Aabb = Aabb {
        x: Interval { min: f64::NEG_INFINITY, max: f64::INFINITY },
        y: Interval { min: f64::NEG_INFINITY, max: f64::INFINITY },
        z: Interval { min: f64::NEG_INFINITY, max: f64::INFINITY },
    };
    /// 默认AABB是空的，因为Interval默认是空的
    pub fn new_empty() -> Self {
        Self {
            x: Interval::empty(),
            y: Interval::empty(),
            z: Interval::empty(),
        }
    }

    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        let mut bbox = Self { x, y, z };
        bbox.pad_to_minimums();
        bbox
    }

    pub fn from_points(a: Point3, b: Point3) -> Self {
        let x = Interval::new(a.x.min(b.x), a.x.max(b.x));
        let y = Interval::new(a.y.min(b.y), a.y.max(b.y));
        let z = Interval::new(a.z.min(b.z), a.z.max(b.z));
        let mut bbox = Self { x, y, z };
        bbox.pad_to_minimums();
        bbox
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

    pub fn longest_axis(&self) -> usize {
        let x_size = self.x.size();
        let y_size = self.y.size();
        let z_size = self.z.size();

        if x_size > y_size {
            if x_size > z_size { 0 } else { 2 }
        } else {
            if y_size > z_size { 1 } else { 2 }
        }
    }
    fn pad_to_minimums(&mut self) {
        let delta = 0.0001;
        if self.x.size() < delta {
            self.x = self.x.expand(delta);
        }
        if self.y.size() < delta {
            self.y = self.y.expand(delta);
        }
        if self.z.size() < delta {
            self.z = self.z.expand(delta);
        }
    }
}

impl Add<Vec3> for Aabb {
    type Output = Aabb;

    fn add(self, offset: Vec3) -> Aabb {
        Aabb {
            x: self.x + offset.x,
            y: self.y + offset.y,
            z: self.z + offset.z,
        }
    }
}

// 可选：实现 Vec3 + Aabb，直接调用 Aabb + Vec3
impl Add<Aabb> for Vec3 {
    type Output = Aabb;

    fn add(self, bbox: Aabb) -> Aabb {
        bbox + self
    }
}