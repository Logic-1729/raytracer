// src/main.rs
use std::io::{self, Write};
use std::sync::Arc;

mod color;
mod hittable;
mod hittable_list;
mod ray;
mod rtweekend;
mod sphere;
mod vec3;
mod camera;
pub mod interval;

use color::write_color;
use hittable::HitRecord;
use ray::Ray;
use rtweekend::{degrees_to_radians, INFINITY, Shared};
use vec3::{Color, Vec3};
use interval::Interval;

use crate::camera::Camera;
use crate::vec3::Point3;
use crate::sphere::Sphere;
use crate::hittable::Hittable;
use crate::hittable_list::HittableList;

fn main() {
    // 创建世界
    let mut world = HittableList::new();

    world.add(Arc::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
    world.add(Arc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));

    // 创建 camera 对象，使用 new 方法初始化
    let mut cam = Camera::new();

    // 分别设置参数
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;

    // 渲染
    cam.render(&world, &mut std::io::stdout()).unwrap();
}