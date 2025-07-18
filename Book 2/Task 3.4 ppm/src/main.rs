// src/main.rs
use std::io::{self, Write};
use std::fs::OpenOptions;
use std::sync::Arc;

mod color;
mod hittable;
mod ray;
mod rtweekend;
mod sphere;
mod vec3;
mod camera;
mod material;
mod interval;
mod aabb;

use std::time::Instant;
use crate::color::write_color;
use crate::ray::Ray;
use crate::rtweekend::{degrees_to_radians, INFINITY};
use crate::rtweekend::random::{random_double, random_double_range};
use crate::rtweekend::time_it;
use crate::vec3::{Color, Point3,Vec3};
use crate::interval::Interval;
use crate::camera::Camera;
use crate::sphere::Sphere;
use crate::hittable::{HitRecord,Hittable,HittableList};
use crate::material::{Material,Lambertian,Metal,Dielectric};

fn main() {
     // 创建世界
    let mut world = HittableList::new();

    // 创建材质
    //let material_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    //let material_center = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    //let material_left   = Arc::new(Dielectric::new(1.50));
    //let material_bubble = Arc::new(Dielectric::new(1.00 / 1.50));
    //let material_right  = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));

    //world.add(Arc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0),100.0,Some(material_ground.clone()),)));
    //world.add(Arc::new(Sphere::new(Point3::new(0.0, 0.0, -1.2),0.5,Some(material_center.clone()),)));
    //world.add(Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0),0.5,Some(material_left.clone()),)));
    //world.add(Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0),0.4,Some(material_bubble.clone()),)));
    //world.add(Arc::new(Sphere::new(Point3::new(1.0, 0.0, -1.0),0.5,Some(material_right.clone()),)));

    // 创建两个球体，左边是蓝色漫反射球，右边是红色漫反射球
    // 这里使用 Arc::new 来创建材质的共享引用
    //let r = std::f64::consts::FRAC_1_SQRT_2; // cos(π/4) = 1/√2
    //let material_left  = Arc::new(Lambertian::new(Color::new(0.0, 0.0, 1.0)));
    //let material_right = Arc::new(Lambertian::new(Color::new(1.0, 0.0, 0.0)));
    //world.add(Arc::new(Sphere::new(Point3::new(-r, 0.0, -1.0),r,Some(material_left.clone()),)));
    //world.add(Arc::new(Sphere::new(Point3::new(r, 0.0, -1.0),r,Some(material_right.clone()),)));

    //let material_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    //let material_center = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    //let material_left   = Arc::new(Dielectric::new(1.50));  
    //let material_bubble = Arc::new(Dielectric::new(1.00 / 1.50));
    //let material_right  = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 1.0));

    //world.add(Arc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0),100.0,Some(material_ground.clone()),)));
    //world.add(Arc::new(Sphere::new(Point3::new(0.0, 0.0, -1.2),0.5,Some(material_center.clone()),)));
    //world.add(Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0),0.5,Some(material_left.clone()),)));
    //world.add(Arc::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0),0.4,Some(material_bubble.clone()),)));
    //world.add(Arc::new(Sphere::new(Point3::new(1.0, 0.0, -1.0),0.5,Some(material_right.clone()),)));

    // 地面
    let ground_material = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    world.add(Arc::new(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, Some(ground_material))));

    // 小球遍历
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_double();
            let center = Point3::new(
                a as f64 + 0.9 * random_double(),
                0.2,
                b as f64 + 0.9 * random_double(),
            );
            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // 漫反射
                    let albedo = Color::random() * Color::random();
                    let sphere_material = Arc::new(Lambertian::new(albedo));
                    let center2 = center + Vec3::new(0.0, random_double_range(0.0, 0.5), 0.0);
                    world.add(Arc::new(Sphere::moving(center, center2, 0.2, Some(sphere_material))));
                } else if choose_mat < 0.95 {
                    // 金属
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = random_double_range(0.0, 0.5);
                    let sphere_material = Arc::new(Metal::new(albedo, fuzz));
                    world.add(Arc::new(Sphere::new(center, 0.2, Some(sphere_material))));
                } else {
                    // 玻璃
                    let sphere_material = Arc::new(Dielectric::new(1.5));
                    world.add(Arc::new(Sphere::new(center, 0.2, Some(sphere_material))));
                }
            }
        }
    }

    // 大球们
    let material1 = Arc::new(Dielectric::new(1.5));
    world.add(Arc::new(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, Some(material1))));

    let material2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Arc::new(Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, Some(material2))));

    let material3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Arc::new(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, Some(material3))));

    // 创建 camera 对象，使用 new 方法初始化
    let mut cam = Camera::new();

    // 分别设置参数
    //cam.aspect_ratio = 16.0 / 9.0;
    //cam.image_width = 400;
    //cam.samples_per_pixel = 100;
    //cam.max_depth = 50;
    //cam.vfov = 20.0;
    //cam.lookfrom = Point3::new(-2.0, 2.0, 1.0);
    //cam.lookat   = Point3::new(0.0, 0.0, -1.0);
    //cam.vup      = Vec3::new(0.0, 1.0, 0.0);
    //cam.defocus_angle = 10.0;
    //cam.focus_dist    = 3.4;

    cam.aspect_ratio      = 16.0 / 9.0;
    cam.image_width       = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth         = 50;
    cam.vfov     = 20.0;
    cam.lookfrom = Point3::new(13.0,2.0,3.0);
    cam.lookat   = Point3::new(0.0,0.0,0.0);
    cam.vup      = Vec3::new(0.0,1.0,0.0);
    cam.defocus_angle = 0.6;
    cam.focus_dist    = 10.0;

    // 渲染
    let (duration, result) = time_it(|| cam.render(&world, &mut std::io::stdout()).unwrap());
    let mut file = OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "Render耗时: {:?}", duration).unwrap();
}