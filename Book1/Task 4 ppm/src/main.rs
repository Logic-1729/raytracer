mod vec3;
mod ray;
mod color;

use std::io::{self, Write};
use vec3::{Color, Point3, Vec3};
use ray::Ray;
use color::write_color;

fn ray_color(r: &ray::Ray) -> vec3::Color {
    let unit_direction = vec3::unit_vector(r.direction());
    let a = 0.5 * (unit_direction.y + 1.0);
    vec3::Color::new(1.0, 1.0, 1.0) * (1.0 - a) + vec3::Color::new(0.5, 0.7, 1.0) * a
}

fn main() -> io::Result<()> {
    // 图像参数
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let image_height = (image_width as f64 / aspect_ratio) as i32;
    let image_height = if image_height < 1 { 1 } else { image_height };

    // 相机参数
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (image_width as f64 / image_height as f64);
    let camera_center = Point3::new(0.0, 0.0, 0.0);

    // 计算视口向量
    let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
    let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

    // 计算像素增量
    let pixel_delta_u = viewport_u / image_width as f64;
    let pixel_delta_v = viewport_v / image_height as f64;

    // 计算左上角像素位置
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

    // 写入PPM头
    writeln!(handle, "P3\n{} {}\n255", image_width, image_height)?;

    for j in 0..image_height {
        // 更新进度
        write!(stderr_handle, "\rScanlines remaining: {} ", image_height - j)?;
        stderr_handle.flush()?;

        for i in 0..image_width {
            // 计算像素中心位置
            let pixel_center = pixel00_loc 
                + pixel_delta_u * i as f64 
                + pixel_delta_v * j as f64;
            
            // 创建光线
            let ray_direction = pixel_center - camera_center;
            let r = Ray::new(camera_center, ray_direction);

            // 计算颜色并写入
            let pixel_color = ray_color(&r);
            write_color(&mut handle, pixel_color)?;
        }
    }

    // 完成渲染
    writeln!(stderr_handle, "\rDone.                 ")?;
    Ok(())
}