use crate::vec3::{Point3, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    origin: Point3,
    direction: Vec3,
}

impl Ray {
    /// 创建一个新的光线
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// 获取光线原点
    pub fn origin(&self) -> Point3 {
        self.origin
    }

    /// 获取光线方向
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// 计算光线在参数 t 处的位置
    pub fn at(&self, t: f64) -> Point3 {
        self.origin + t * self.direction
    }
}