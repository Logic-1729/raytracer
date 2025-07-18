
use crate::vec3::{Vec3, Point3};

pub struct Onb {
    axis: [Vec3; 3],
}

impl Onb {
    pub fn new(n: &Vec3) -> Self {
        let w = Vec3::unit_vector(*n);
        let a = if w.x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let v = Vec3::unit_vector(Vec3::cross(&w, &a));
        let u = Vec3::cross(&w, &v);
        Self { axis: [u, v, w] }
    }

    pub fn u(&self) -> &Vec3 {
        &self.axis[0]
    }

    pub fn v(&self) -> &Vec3 {
        &self.axis[1]
    }

    pub fn w(&self) -> &Vec3 {
        &self.axis[2]
    }

    pub fn transform(&self, v: &Vec3) -> Vec3 {
        v.x * self.axis[0] + v.y * self.axis[1] + v.z * self.axis[2]
    }
}
