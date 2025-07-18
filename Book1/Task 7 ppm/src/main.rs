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
pub mod interval;

use color::write_color;
use hittable::{HitRecord, Hittable};
use hittable_list::{HittableList};
use ray::Ray;
use rtweekend::{degrees_to_radians, INFINITY, Shared};
use sphere::Sphere;
use interval::Interval;
use vec3::{Color, Point3, Vec3};

fn ray_color(r: &Ray, world: &dyn Hittable) -> Color {
    let mut rec = HitRecord::default();
    if world.hit(r, Interval::new(0.0, f64::INFINITY), &mut rec) {
        return (rec.normal + Color::new(1.0, 1.0, 1.0)) * 0.5;
    }
    let unit_direction = Vec3::unit_vector(r.direction);
    let a = 0.5 * (unit_direction.y + 1.0);
    Color::new(1.0, 1.0, 1.0) * (1.0 - a) + Color::new(0.5, 0.7, 1.0) * a
}

fn main() -> io::Result<()> {
    // 图像参数
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as i32;
    let image_height = image_height.max(1);

    // 世界场景
    let mut world = HittableList::new();
    world.add(Arc::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
    world.add(Arc::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));

    // 相机参数
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
    let camera_center = Point3::new(0.0, 0.0, 0.0);

    // 视口计算
    let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
    let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);
    let pixel_delta_u = viewport_u / image_width as f64;
    let pixel_delta_v = viewport_v / image_height as f64;

    // 初始像素位置
    let viewport_upper_left = camera_center 
        - Vec3::new(0.0, 0.0, focal_length) 
        - viewport_u / 2.0 
        - viewport_v / 2.0;
    let pixel00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

    // 渲染
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let stderr = io::stderr();
    let mut stderr_handle = stderr.lock();

    writeln!(handle, "P3\n{} {}\n255", image_width, image_height)?;

    for j in 0..image_height {
        write!(stderr_handle, "\rScanlines remaining: {} ", image_height - j)?;
        stderr_handle.flush()?;

        for i in 0..image_width {
            let pixel_center = pixel00_loc 
                + pixel_delta_u * i as f64 
                + pixel_delta_v * j as f64;
            let ray_direction = pixel_center - camera_center;
            let r = Ray::new(camera_center, ray_direction);

            let pixel_color = ray_color(&r, &world);
            write_color(&mut handle, pixel_color)?;
        }
    }

    writeln!(stderr_handle, "\rDone.                 ")?;
    Ok(())
}