use std::sync::{Arc, Mutex, Condvar, atomic::{AtomicUsize, Ordering}};
use std::io::{self, Write};
use std::fs::File;
use crate::vec3::{Point3, Vec3};
use crate::ray::Ray;
use crate::color::*;
use crate::hittable::{Hittable,HitRecord};
use crate::interval::Interval;
use crate::rtweekend::*;
use crate::camera::random::random_double;
use crate::material::Material;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use crossbeam::thread;
use image::{ImageBuffer, RgbImage};

const AUTHOR: &str = "PhantomPhoenix";

pub struct Camera {
    pub aspect_ratio: f64,   // Ratio of image width over height
    pub image_width: usize,  // Rendered image width in pixel count
    pub samples_per_pixel: usize,
    pub max_depth: usize,
    pub vfov: f64,            // Vertical field of view in degrees
    pub lookfrom: Point3,   // Point camera is looking from
    pub lookat: Point3,     // Point camera is looking at
    pub vup: Vec3,          // Up vector for camera orientation
    pub defocus_angle: f64,  // Variation angle of rays through each pixel
    pub focus_dist: f64, 

    // Private fields (computed in initialize)
    image_height: usize,         // Rendered image height
    pixel_samples_scale: f64,
    center: Point3,              // Camera center
    pixel00_loc: Point3,         // Location of pixel 0, 0
    pixel_delta_u: Vec3,         // Offset to pixel to the right
    pixel_delta_v: Vec3,         // Offset to pixel below
    u: Vec3,                  // Right vector
    v: Vec3,                  // Up vector  
    w: Vec3,
    defocus_disk_u: Vec3,       // Defocus disk horizontal radius
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn new() -> Self {
        let mut cam = Camera {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            vfov: 90.0,
            lookfrom: Point3::default(),   // Point camera is looking from
            lookat: Point3::new(0.0, 0.0, -1.0),     // Point camera is looking at
            vup: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,  // Variation angle of rays through each pixel
            focus_dist: 10.0,
            image_height: 0,
            pixel_samples_scale: 0.0,
            center: Point3::default(),
            pixel00_loc: Point3::default(),
            pixel_delta_u: Vec3::default(),
            pixel_delta_v: Vec3::default(),
            u: Vec3::default(),          // Right vector
            v: Vec3::default(),          // Up vector
            w: Vec3::default(),          // Forward vector
            defocus_disk_u: Vec3::default(),       // Defocus disk horizontal radius
            defocus_disk_v: Vec3::default(),
        };
        cam.initialize();
        cam
    }

    fn initialize(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as usize;
        if self.image_height < 1 { self.image_height = 1;}

        self.pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;

        self.center = self.lookfrom;

        // 计算视口尺寸
        //let focal_length = (self.lookfrom - self.lookat).length();
        let theta = crate::rtweekend::degrees_to_radians(self.vfov);
        let tmp_theta = theta / 2.0; // 半角
        let h = tmp_theta.tan();
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        // 计算相机坐标系的正交基
        self.w = Vec3::unit_vector(self.lookfrom - self.lookat);    // 相机指向的方向   
        self.u = Vec3::unit_vector(Vec3::cross(&self.vup, &self.w)); // 右向量
        self.v = Vec3::cross(&self.w, &self.u);                     // 上向量

        // 计算视口边缘向量
        let viewport_u = self.u * viewport_width;      // 横向（右方向）视口向量
        let viewport_v = -self.v * viewport_height;    // 纵向（下方向）视口向量

        // 计算像素间隔向量
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        // 计算左上角像素中心位置
        let viewport_upper_left = self.center - (self.focus_dist * self.w) - viewport_u / 2.0 - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);

        let tmp_angle = self.defocus_angle / 2.0;
        let tmp_radian = degrees_to_radians(tmp_angle);
        let defocus_radius = self.focus_dist * tmp_radian.tan();
        let defocus_disk_u = self.u * defocus_radius;
        let defocus_disk_v = self.v * defocus_radius;
    }

    //单线程渲染函数
    /*pub fn render<w: Write>(&mut self, world: &dyn Hittable, out: &mut w) -> io::Result<()> {
        self.initialize();

        writeln!(out, "P3\n{} {}\n255", self.image_width, self.image_height)?;

        for j in 0..self.image_height {
            eprint!("\rScanlines remaining: {} ", self.image_height - j);
            io::stderr().flush().unwrap();

            for i in 0..self.image_width {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _sample in 0..self.samples_per_pixel {
                    let r = self.get_ray(i, j);
                    pixel_color = pixel_color + ray_color(&r, self.max_depth, world);
                }
                // 采样平均（注意不能提前gamma！）
                pixel_color = pixel_color * self.pixel_samples_scale as f64;
                write_color(out, &pixel_color)?; // write_color内部做gamma
            }
        }

        eprintln!("\rDone.                 ");
        Ok(())
    }*/

    //多线程渲染函数1

pub fn render<W: Write + Send>(&mut self,
    world: &dyn Hittable,
    out: &mut W,
) -> io::Result<()> {
    self.initialize();
    let path = "./test.jpg";
    let quality = 100;

    writeln!(out, "size: {} * {}", self.image_width, self.image_height)?;

    let width = self.image_width;
    let height = self.image_height;

    // 并行生成像素字符串和图片buffer
    let mut pixels = vec![String::new(); width * height];
    let mut img = RgbImage::new(width as u32, height as u32);

    pixels
        .par_iter_mut()
        .enumerate()
        .for_each(|(idx, pix)| {
            let i = idx % width;
            let j = idx / width;
            let mut pixel_color = Color::default();
            for _ in 0..self.samples_per_pixel {
                let r = self.get_ray(i, j);
                pixel_color = pixel_color + ray_color(&r, self.max_depth, world);
            }
            pixel_color = pixel_color * self.pixel_samples_scale as f64;
            *pix = color_to_string(pixel_color);

            // 写入 image::RgbImage
            let rgb = pixel_color.to_rgb_u8(); // 假设你有这个方法, 返回 [u8; 3]
            // Safety: img buffer并行写不安全，这里演示，实际应用需用Mutex或collect后再顺序写入
            // 这里推荐单线程遍历pixels后写img，见下方注释
        });

    // 顺序写出PPM，并填充img
    for j in 0..height {
        eprint!("\rScanlines remaining: {} ", height - j);
        io::stderr().flush().unwrap();
        for i in 0..width {
            let idx = j * width + i;
            //writeln!(out, "{}", pixels[idx])?;

            // 解析pixels[idx]成RGB
            let comps: Vec<u8> = pixels[idx]
                .split_whitespace()
                .take(3)
                .filter_map(|x| x.parse().ok())
                .collect();
            if comps.len() == 3 {
                img.put_pixel(i as u32, j as u32, image::Rgb([comps[0], comps[1], comps[2]]));
            }
        }
    }
    eprintln!("\rDone.                 ");
    println!("Ouput image as \"{}\"\n Author: {}", path, AUTHOR);

    // 保存JPEG
    let output_image = image::DynamicImage::ImageRgb8(img);
    let mut output_file: File = File::create(path).unwrap();
    output_image.write_to(&mut output_file, image::ImageOutputFormat::Jpeg(quality)).unwrap();

    Ok(())
}

    pub fn get_ray(&self, i: usize, j: usize) -> Ray {
        // 构造一条起点为相机中心、方向指向像素(i, j)周围随机采样点的射线

        let offset = self.sample_square();
        let pixel_sample = self.pixel00_loc
            + self.pixel_delta_u * ((i as f64) + offset.x)
            + self.pixel_delta_v * ((j as f64) + offset.y);

        let ray_origin = if self.defocus_angle <= 0.0 { self.center } else { self.defocus_disk_sample() };
        let ray_direction = pixel_sample - ray_origin;
        let ray_time = random_double();

        Ray::new(ray_origin, ray_direction,ray_time)
    }

    fn sample_square(&self) -> Vec3 {
        // 返回一个在[-0.5, -0.5]到[+0.5, +0.5]区间的随机向量
        Vec3::new(random_double() - 0.5,random_double() - 0.5,0.0,)
    }
    pub fn defocus_disk_sample(&self) -> Point3 {
        let p = Vec3::random_in_unit_disk();
        self.center + self.defocus_disk_u * p.x + self.defocus_disk_v * p.y
    }
}

fn ray_color(r: &Ray, depth: usize, world: &dyn Hittable) -> Color {
    // If we've exceeded the ray bounce limit, no more light is gathered.
    if depth == 0 { return Color::default();}

    let mut rec = HitRecord::default();

    if world.hit(r, Interval::new(0.001, f64::INFINITY), &mut rec) {
        if let Some(mat) = &rec.mat {
            let mut scattered = Ray::default();
            let mut attenuation = Color::default();
            if mat.scatter(r, &rec, &mut attenuation, &mut scattered) { return attenuation * ray_color(&scattered, depth - 1, world); }
        }
        return Color::default();
    }

    let unit_direction = Vec3::unit_vector(r.direction);
    let a = 0.5 * (unit_direction.y + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}
