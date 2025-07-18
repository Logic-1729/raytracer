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
mod material;
mod interval;

use crate::color::write_color;
use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::rtweekend::{degrees_to_radians, INFINITY, Shared};
use crate::vec3::{Color, Vec3};
use crate::interval::Interval;
use crate::camera::Camera;
use crate::vec3::Point3;
use crate::sphere::Sphere;
use crate::hittable::Hittable;
use crate::hittable_list::HittableList;
use crate::material::Lambertian;
use crate::material::Metal;
use crate::material::Dielectric;

fn main() {
    // 创建世界
    let mut world = HittableList::new();

    // 创建材质
    let material_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let material_center = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let material_left   = Arc::new(Dielectric::new(1.50));
    let material_bubble = Arc::new(Dielectric::new(1.00 / 1.50));
    let material_right  = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));

    world.add(Arc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0),100.0,Some(material_ground.clone()),)));
    world.add(Arc::new(Sphere::new(Point3::new(0.0, 0.0, -1.2),0.5,Some(material_center.clone()),)));
    world.add(Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0),0.5,Some(material_left.clone()),)));
    world.add(Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0),0.4,Some(material_bubble.clone()),)));
    world.add(Arc::new(Sphere::new(Point3::new(1.0, 0.0, -1.0),0.5,Some(material_right.clone()),)));

    // 创建 camera 对象，使用 new 方法初始化
    let mut cam = Camera::new();

    // 分别设置参数
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth = 50;

    // 渲染
    cam.render(&world, &mut std::io::stdout()).unwrap();
}