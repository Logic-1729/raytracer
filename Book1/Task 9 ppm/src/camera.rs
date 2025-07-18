use std::io::{self, Write};
use crate::vec3::{Point3, Vec3};
use crate::ray::Ray;
use crate::color::{Color, write_color};
use crate::hittable::{Hittable,HitRecord};
use crate::interval::Interval;
use crate::rtweekend::random::random_double;

pub struct Camera {
    pub aspect_ratio: f64,   // Ratio of image width over height
    pub image_width: usize,  // Rendered image width in pixel count
    pub samples_per_pixel: usize,

    // Private fields (computed in initialize)
    image_height: usize,         // Rendered image height
    pixel_samples_scale: f64,
    center: Point3,              // Camera center
    pixel00_loc: Point3,         // Location of pixel 0, 0
    pixel_delta_u: Vec3,         // Offset to pixel to the right
    pixel_delta_v: Vec3,         // Offset to pixel below
}

impl Camera {
    pub fn new() -> Self {
        let mut cam = Camera {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            image_height: 0,
            pixel_samples_scale: 0.0,
            center: Point3::new(0.0, 0.0, 0.0),
            pixel00_loc: Point3::new(0.0, 0.0, 0.0),
            pixel_delta_u: Vec3::new(0.0, 0.0, 0.0),
            pixel_delta_v: Vec3::new(0.0, 0.0, 0.0),
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
                    pixel_color = pixel_color + ray_color(&r, world);
                }
                write_color(
                    out,
                    &(pixel_color * self.pixel_samples_scale)
                ).unwrap();
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

        self.center = Point3::new(0.0, 0.0, 0.0);

        // Determine viewport dimensions.
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left =
            self.center
            - Vec3::new(0.0, 0.0, focal_length)
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

fn ray_color(r: &Ray, world: &dyn Hittable) -> Color {
    let mut rec = HitRecord::default();

    if world.hit(r, Interval::new(0.0, f64::INFINITY), &mut rec) {
        let direction = Vec3::random_on_hemisphere(&rec.normal);
        return 0.5 * ray_color(&Ray::new(rec.p, direction), world);
    }

    let unit_direction = Vec3::unit_vector(r.direction);
    let a = 0.5 * (unit_direction.y + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}
