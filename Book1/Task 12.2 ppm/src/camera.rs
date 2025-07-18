use std::io::{self, Write};
use crate::vec3::{Point3, Vec3};
use crate::ray::Ray;
use crate::color::{Color, write_color};
use crate::hittable::{Hittable,HitRecord};
use crate::interval::Interval;
use crate::rtweekend::random::random_double;
use crate::material::Material;

pub struct Camera {
    pub aspect_ratio: f64,   // Ratio of image width over height
    pub image_width: usize,  // Rendered image width in pixel count
    pub samples_per_pixel: usize,
    pub max_depth: usize,
    pub vfov: f64,            // Vertical field of view in degrees
    pub lookfrom: Point3,   // Point camera is looking from
    pub lookat: Point3,     // Point camera is looking at
    pub vup: Vec3,          // Up vector for camera orientation

    // Private fields (computed in initialize)
    image_height: usize,         // Rendered image height
    pixel_samples_scale: f64,
    center: Point3,              // Camera center
    pixel00_loc: Point3,         // Location of pixel 0, 0
    pixel_delta_u: Vec3,         // Offset to pixel to the right
    pixel_delta_v: Vec3,         // Offset to pixel below
    u: Vec3,                  // Right vector
    v: Vec3,                  // Up vector  
    W: Vec3,
}

impl Camera {
    pub fn new() -> Self {
        let mut cam = Camera {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            vfov: 90.0,
            lookfrom: Point3::new(0.0, 0.0, 0.0),   // Point camera is looking from
            lookat: Point3::new(0.0, 0.0, -1.0),     // Point camera is looking at
            vup: Vec3::new(0.0, 1.0, 0.0),
            image_height: 0,
            pixel_samples_scale: 0.0,
            center: Point3::new(0.0, 0.0, 0.0),
            pixel00_loc: Point3::new(0.0, 0.0, 0.0),
            pixel_delta_u: Vec3::new(0.0, 0.0, 0.0),
            pixel_delta_v: Vec3::new(0.0, 0.0, 0.0),
            u: Vec3::new(0.0, 0.0, 0.0),          // Right vector
            v: Vec3::new(0.0, 0.0, 0.0),          // Up vector
            W: Vec3::new(0.0, 0.0, 0.0),          // Forward vector
        };
        cam.initialize();
        cam
    }

    pub fn render<W: Write>(&mut self, world: &dyn Hittable, out: &mut W) -> io::Result<()> {
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
    }

    fn initialize(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as usize;
        if self.image_height < 1 {
            self.image_height = 1;
        }

        self.pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;

        self.center = self.lookfrom;

        // 计算视口尺寸
        let focal_length = (self.lookfrom - self.lookat).length();
        let theta = crate::rtweekend::degrees_to_radians(self.vfov);
        let tmp_theta = theta / 2.0; // 半角
        let h = tmp_theta.tan();
        let viewport_height = 2.0 * h * focal_length;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        // 计算相机坐标系的正交基
        self.W = Vec3::unit_vector(self.lookfrom - self.lookat);    // 相机指向的方向   
        self.u = Vec3::unit_vector(Vec3::cross(&self.vup, &self.W)); // 右向量
        self.v = Vec3::cross(&self.W, &self.u);                     // 上向量

        // 计算视口边缘向量
        let viewport_u = self.u * viewport_width;      // 横向（右方向）视口向量
        let viewport_v = -self.v * viewport_height;    // 纵向（下方向）视口向量

        // 计算像素间隔向量
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        // 计算左上角像素中心位置
        let viewport_upper_left =
            self.center
            - focal_length * self.W
            - viewport_u / 2.0
            - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);
    }

    pub fn get_ray(&self, i: usize, j: usize) -> Ray {
        // 构造一条起点为相机中心、方向指向像素(i, j)周围随机采样点的射线

        let offset = self.sample_square();
        let pixel_sample = self.pixel00_loc
            + self.pixel_delta_u * ((i as f64) + offset.x)
            + self.pixel_delta_v * ((j as f64) + offset.y);

        let ray_origin = self.center;
        let ray_direction = pixel_sample - ray_origin;

        Ray::new(ray_origin, ray_direction)
    }

    fn sample_square(&self) -> Vec3 {
        // 返回一个在[-0.5, -0.5]到[+0.5, +0.5]区间的随机向量
        Vec3::new(
            random_double() - 0.5,
            random_double() - 0.5,
            0.0,
        )
    }
}

fn ray_color(r: &Ray, depth: usize, world: &dyn Hittable) -> Color {
    // If we've exceeded the ray bounce limit, no more light is gathered.
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let mut rec = HitRecord::default();

    if world.hit(r, Interval::new(0.001, f64::INFINITY), &mut rec) {
    if let Some(mat) = &rec.mat {
        let mut scattered = Ray::default();
        let mut attenuation = Color::zero();
        if mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
            return attenuation * ray_color(&scattered, depth - 1, world);
        }
    }
    return Color::new(0.0, 0.0, 0.0);
}

    let unit_direction = Vec3::unit_vector(r.direction);
    let a = 0.5 * (unit_direction.y + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}
