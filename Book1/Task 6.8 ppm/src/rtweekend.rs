// src/rtweekend.rs
use std::f64::consts::PI as STD_PI;
use std::f64::INFINITY as STD_INFINITY;
use rand::Rng;  // 添加这行

// 重新导出常用模块
pub use crate::color::*;
pub use crate::ray::*;
pub use crate::vec3::*;

pub const PI: f64 = STD_PI;
pub const INFINITY: f64 = STD_INFINITY;

/// 实用函数
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

/// 随机数工具模块
pub mod random {
    use rand::Rng;
    use std::f64::consts::PI;

    /// 生成 [0,1) 范围内的随机浮点数
    pub fn random_double() -> f64 {
        rand::thread_rng().r#gen() 
    } 

    /// 生成 [min,max) 范围内的随机浮点数
    pub fn random_double_range(min: f64, max: f64) -> f64 {
        rand::thread_rng().gen_range(min..max)
    }

    /// 生成随机单位向量
    pub fn random_unit_vector() -> crate::vec3::Vec3 {
        let a = random_double_range(0.0, 2.0 * PI);
        let z = random_double_range(-1.0, 1.0);
        let r = (1.0 - z * z).sqrt();
        crate::vec3::Vec3::new(r * a.cos(), r * a.sin(), z)
    }
}

/// 替代 C++ 的 shared_ptr
pub type Shared<T> = std::sync::Arc<T>;