use crate::vec3::{Point3, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    pub tm: f64,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3, time: f64) -> Self { Ray { origin, direction, tm: time }}
    
    pub fn default() -> Self { Ray {origin: Point3::default(),direction: Vec3::default(),tm: 0.0,}}
    
    pub fn origin(&self) -> Point3 { self.origin }

    /// 获取光线方向
    pub fn direction(&self) -> Vec3 { self.direction }

    pub fn time(&self) -> f64 { self.tm }

    pub fn at(&self, t: f64) -> Point3 {
        self.origin + Vec3 {
            x: self.direction.x * t,
            y: self.direction.y * t,
            z: self.direction.z * t,
        }
    }
}

impl Default for Ray {
    fn default() -> Self {
        Ray {
            origin: Point3::new(0.0, 0.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 0.0),
            tm: 0.0,
        }
    }
}