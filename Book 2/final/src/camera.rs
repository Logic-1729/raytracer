use std::sync::{Arc, Mutex, Condvar, atomic::{AtomicUsize, Ordering}};
use std::io::{self, Write};
use std::fs::File;
use crate::vec3::{Point3, Vec3};
use crate::ray::Ray;
use crate::color::*;
use crate::hittable::{Hittable,HitRecord};
use crate::interval::Interval;
use crate::rtweekend::*;
use crate::rtweekend::random::random_double;
use crate::material::Material;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use crossbeam::thread;
use image::{ImageBuffer, RgbImage};
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;

// SIMD优化相关导入
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// 使用feature gate来启用不稳定的SIMD功能
#[cfg(feature = "simd")]
use std::simd::{f64x4, f64x8, u64x4, Simd};

const AUTHOR: &str = "PhantomPhoenix";

// 俄罗斯轮盘赌策略枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RussianRouletteStrategy {
    None,         // 不使用俄罗斯轮盘赌
    HighQuality,  // 高质量策略：极致的质量保证
    Conservative, // 保守策略：质量和性能平衡，偏向质量
    Aggressive,   // 激进策略：更早开始终止
    Adaptive,     // 自适应策略：智能的质量优化
}

impl Default for RussianRouletteStrategy {
    fn default() -> Self {
        RussianRouletteStrategy::HighQuality // 默认使用高质量策略
    }
}

// SIMD优化的随机数生成器结构 - 移到模块作用域
#[cfg(feature = "simd")]
pub struct SimdRng {
    state: u64x4,
}

#[cfg(feature = "simd")]
impl SimdRng {
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        Self {
            state: u64x4::from_array([
                seed.wrapping_mul(1103515245).wrapping_add(12345),
                seed.wrapping_mul(1103515245).wrapping_add(12346),
                seed.wrapping_mul(1103515245).wrapping_add(12347),
                seed.wrapping_mul(1103515245).wrapping_add(12348),
            ])
        }
    }

    // SIMD版本的线性同余生成器
    pub fn next_f64x4(&mut self) -> f64x4 {
        // 使用线性同余生成器: state = state * a + c
        let a = u64x4::splat(1103515245);
        let c = u64x4::splat(12345);
        self.state = self.state * a + c;
        
        // 转换为[0, 1)范围的f64
        let max_u64_f64 = u64::MAX as f64;
        let normalized = self.state.cast::<f64>() / f64x4::splat(max_u64_f64);
        normalized
    }
}

// 自定义SIMD随机数生成器，用于与材质scatter函数兼容 - 移到模块作用域
#[cfg(feature = "simd")]
struct CustomSimdRng {
    values: f64x4,
    index: usize,
}

#[cfg(feature = "simd")]
impl Rng for CustomSimdRng {
    fn next_u32(&mut self) -> u32 {
        let val = self.values[self.index % 4];
        self.index += 1;
        (val * u32::MAX as f64) as u32
    }

    fn next_u64(&mut self) -> u64 {
        let val = self.values[self.index % 4];
        self.index += 1;
        (val * u64::MAX as f64) as u64
    }
}

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
    pub background: Color, // Background color for rays that miss all objects

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
    
    // Additional fields for multi-threading
    sqrt_spp: usize,            // Square root of samples per pixel
    bar: ProgressBar,           // Progress bar
    pub russian_roulette: RussianRouletteStrategy, // 俄罗斯轮盘赌策略
}

impl Camera {
    pub fn new() -> Self {
        let sqrt_spp = (10.0_f64.sqrt()) as usize; // Default sqrt of samples_per_pixel
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
            background: Color::new(0.70, 0.80, 1.00), // Default background color
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
            sqrt_spp,
            bar: ProgressBar::hidden(), // Initialize as hidden
            russian_roulette: RussianRouletteStrategy::default(),
        };
        cam.initialize();
        cam
    }

    fn initialize(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as usize;
        if self.image_height < 1 { self.image_height = 1;}

        self.pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;
        self.sqrt_spp = (self.samples_per_pixel as f64).sqrt() as usize;

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
        self.defocus_disk_u = self.u * defocus_radius;
        self.defocus_disk_v = self.v * defocus_radius;
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

    // 多线程渲染函数 - 简化版本避免死锁
    pub fn render<W: Write + Send>(&mut self, world: &dyn Hittable, out: &mut W) -> io::Result<()> {
        self.initialize();

        let HEIGHT_PARTITION: u32 = 32;  // 减少分区数量
        let WIDTH_PARTITION: u32 = 32;
        
        // 初始化进度条 - 确保只创建一次
        if self.bar.is_finished() || self.bar.is_hidden() {
            self.bar = ProgressBar::new((HEIGHT_PARTITION * WIDTH_PARTITION) as u64);
            self.bar.set_style(ProgressStyle::default_bar()
                .template("{bar:40.white/black} {pos:>7}/{len:7} {per_sec} 剩余: {eta_precise} {msg}")
                .unwrap());
        } else {
            // 重置已存在的进度条
            self.bar.reset();
            self.bar.set_length((HEIGHT_PARTITION * WIDTH_PARTITION) as u64);
        }

        writeln!(out, "size: {} * {}", self.image_width, self.image_height)?;

        let mut img: RgbImage = ImageBuffer::new(self.image_width as u32, self.image_height as u32);
        let img_mtx = Arc::new(Mutex::new(&mut img));

        crossbeam::thread::scope(|s| {
            let chunk_height = (self.image_height + HEIGHT_PARTITION as usize - 1) / HEIGHT_PARTITION as usize;
            let chunk_width = (self.image_width + WIDTH_PARTITION as usize - 1) / WIDTH_PARTITION as usize;

            let camera_arc = Arc::new(self as &Camera);
            let world_arc = Arc::new(world);
            let mut handles = vec![];

            for j in 0..HEIGHT_PARTITION {
                for i in 0..WIDTH_PARTITION {
                    // 避免在热点循环中clone，直接使用引用
                    let camera_ref = &camera_arc;
                    let world_ref = &world_arc;
                    let img_mtx_ref = &img_mtx;
                    
                    let x_min = (i as usize) * chunk_width;
                    let x_max = std::cmp::min((i as usize + 1) * chunk_width, self.image_width);
                    let y_min = (j as usize) * chunk_height;
                    let y_max = std::cmp::min((j as usize + 1) * chunk_height, self.image_height);

                    let handle = s.spawn({
                        let camera = Arc::clone(camera_ref);
                        let world = Arc::clone(world_ref);
                        let img_mtx = Arc::clone(img_mtx_ref);
                        move |_| {
                            // 检测CPU特性并选择最优化的渲染路径
                            #[cfg(all(feature = "simd", target_arch = "x86_64"))]
                            if is_x86_feature_detected!("avx2") {
                                camera.render_sub_simd(*world, img_mtx, x_min, x_max, y_min, y_max);
                            } else {
                                camera.render_sub(*world, img_mtx, x_min, x_max, y_min, y_max);
                            }
                            
                            #[cfg(not(all(feature = "simd", target_arch = "x86_64")))]
                            camera.render_sub(*world, img_mtx, x_min, x_max, y_min, y_max);
                        }
                    });
                    
                    handles.push(handle);
                }
            }

            // 等待所有线程完成
            for handle in handles {
                let _ = handle.join();
            }
        }).unwrap();

        // 完成进度条并清理
        if !self.bar.is_finished() {
            self.bar.finish_with_message("渲染完成 ✓");
        }

        let path = "test.jpg";
        println!("Output image as \"{}\"\nAuthor: {}", path, AUTHOR);
        
        let output_image = image::DynamicImage::ImageRgb8(img);
        let mut output_file = File::create(path)?;
        match output_image.write_to(&mut output_file, image::ImageOutputFormat::Jpeg(100)) {
            Ok(_) => println!("Image saved successfully!"),
            Err(e) => eprintln!("Error saving image: {}", e),
        }

        Ok(())
    }

    // 渲染子块函数 - 优化内存分配
    pub fn render_sub(&self, world: &dyn Hittable, img_mtx: Arc<Mutex<&mut RgbImage>>,
                      x_min: usize, x_max: usize, y_min: usize, y_max: usize) {
        
        // 检查边界条件，避免下溢错误
        if x_min >= x_max || y_min >= y_max {
            return;
        }
        
        // 创建线程本地随机数生成器，避免竞争
        let mut rng = thread_rng();
        
        // 创建写入缓冲区 - 正确的维度 [height][width]
        let width = x_max - x_min;
        let height = y_max - y_min;
        
        // 预分配缓冲区，避免运行时分配
        let mut write_buffer: Vec<Vec<Color>> = Vec::with_capacity(height);
        for _ in 0..height {
            write_buffer.push(vec![Color::new(0.0, 0.0, 0.0); width]);
        }
        
        // 预计算采样相关常数，避免重复计算
        let inv_samples = 1.0 / (self.sqrt_spp * self.sqrt_spp) as f64;
        let sqrt_spp_f64 = self.sqrt_spp as f64;
 
        // 渲染像素
        for j in y_min..y_max {
            let local_j = j - y_min;
            for i in x_min..x_max {
                let local_i = i - x_min;
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                
                // 使用分层采样 - 避免重复的类型转换
                for s_j in 0..self.sqrt_spp {
                    let s_j_f64 = s_j as f64;
                    for s_i in 0..self.sqrt_spp {
                        let s_i_f64 = s_i as f64;
                        let ray = self.get_ray_stratified_with_rng_optimized(i, j, s_i_f64, s_j_f64, sqrt_spp_f64, &mut rng);
                        let sample_color = self.ray_color_with_rr(&ray, self.max_depth, world);
                        pixel_color += sample_color;
                    }
                }
                
                // 直接计算最终颜色，避免额外的乘法对象创建
                write_buffer[local_j][local_i] = Color::new(
                    pixel_color.x * inv_samples,
                    pixel_color.y * inv_samples,
                    pixel_color.z * inv_samples
                );
            }
        }

        // 写入图像 - 减少边界检查
        {
            let mut binding = img_mtx.lock().unwrap();
            let img: &mut RgbImage = *binding;
            
            for j in 0..height {
                for i in 0..width {
                    let write_x = (i + x_min) as u32;
                    let write_y = (j + y_min) as u32;
                    let color = &write_buffer[j][i];
                    let rgb = [
                        (color.x.sqrt().clamp(0.0, 1.0) * 255.0) as u8,
                        (color.y.sqrt().clamp(0.0, 1.0) * 255.0) as u8,
                        (color.z.sqrt().clamp(0.0, 1.0) * 255.0) as u8
                    ];
                    img.put_pixel(write_x, write_y, image::Rgb(rgb));
                }
            }
        }
        
        // 只有在实际渲染了内容时才更新进度条
        if width > 0 && height > 0 {
            self.bar.inc(1);
        }
    }

    // 分层采样射线生成 - 使用线程本地随机数生成器
    fn get_ray_stratified_with_rng(&self, i: usize, j: usize, s_i: usize, s_j: usize, rng: &mut ThreadRng) -> Ray {
        let offset_x = (s_i as f64 + rng.r#gen::<f64>()) / self.sqrt_spp as f64 - 0.5;
        let offset_y = (s_j as f64 + rng.r#gen::<f64>()) / self.sqrt_spp as f64 - 0.5;
        
        let pixel_sample = self.pixel00_loc
            + self.pixel_delta_u * (i as f64 + offset_x)
            + self.pixel_delta_v * (j as f64 + offset_y);

        let ray_origin = if self.defocus_angle <= 0.0 { 
            self.center 
        } else { 
            self.defocus_disk_sample_with_rng(rng) 
        };
        let ray_direction = pixel_sample - ray_origin;
        let ray_time = rng.r#gen::<f64>();

        Ray::new(ray_origin, ray_direction, ray_time)
    }

    // 分层采样射线生成 - 使用全局随机数（保持向后兼容）
    fn get_ray_stratified(&self, i: usize, j: usize, s_i: usize, s_j: usize) -> Ray {
        let offset_x = (s_i as f64 + random_double()) / self.sqrt_spp as f64 - 0.5;
        let offset_y = (s_j as f64 + random_double()) / self.sqrt_spp as f64 - 0.5;
        
        let pixel_sample = self.pixel00_loc
            + self.pixel_delta_u * (i as f64 + offset_x)
            + self.pixel_delta_v * (j as f64 + offset_y);

        let ray_origin = if self.defocus_angle <= 0.0 { 
            self.center 
        } else { 
            self.defocus_disk_sample() 
        };
        let ray_direction = pixel_sample - ray_origin;
        let ray_time = random_double();

        Ray::new(ray_origin, ray_direction, ray_time)
    }

    // 优化的分层采样射线生成 - 减少重复计算和类型转换
    fn get_ray_stratified_with_rng_optimized(&self, i: usize, j: usize, s_i_f64: f64, s_j_f64: f64, sqrt_spp_f64: f64, rng: &mut ThreadRng) -> Ray {
        let offset_x = (s_i_f64 + rng.r#gen::<f64>()) / sqrt_spp_f64 - 0.5;
        let offset_y = (s_j_f64 + rng.r#gen::<f64>()) / sqrt_spp_f64 - 0.5;
        
        let i_f64 = i as f64;
        let j_f64 = j as f64;
        
        let pixel_sample = self.pixel00_loc
            + self.pixel_delta_u * (i_f64 + offset_x)
            + self.pixel_delta_v * (j_f64 + offset_y);

        let ray_origin = if self.defocus_angle <= 0.0 { 
            self.center 
        } else { 
            self.defocus_disk_sample_with_rng(rng) 
        };
        let ray_direction = pixel_sample - ray_origin;
        let ray_time = rng.r#gen::<f64>();

        Ray::new(ray_origin, ray_direction, ray_time)
    }

    // 使用线程本地随机数生成器的散焦圆盘采样
    pub fn defocus_disk_sample_with_rng(&self, rng: &mut ThreadRng) -> Point3 {
        let p = self.random_in_unit_disk_with_rng(rng);
        self.center + self.defocus_disk_u * p.x + self.defocus_disk_v * p.y
    }

    // 优化的单位圆内随机点生成 - 减少循环和函数调用开销
    fn random_in_unit_disk_with_rng(&self, rng: &mut ThreadRng) -> Vec3 {
        // 使用更高效的算法，减少拒绝采样的次数
        let theta = rng.gen_range(0.0f64..std::f64::consts::TAU); // 2π
        let r = rng.gen_range(0.0f64..1.0f64).sqrt(); // 均匀分布在圆盘内
        Vec3::new(r * theta.cos(), r * theta.sin(), 0.0f64)
    }

    // 射线颜色计算
    fn ray_color(&self, r: &Ray, world: &dyn Hittable, depth: usize) -> Color {
        self.ray_color_with_rr(r, depth, world)
    }

    // 带俄罗斯轮盘赌的射线颜色计算
    fn ray_color_with_rr(&self, r: &Ray, depth: usize, world: &dyn Hittable) -> Color {
        // 射线反弹深度限制
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        let mut rec = HitRecord::default();

        // 如果光线击中场景中的物体
        if world.hit(r, Interval::new(0.001, f64::INFINITY), &mut rec) {
            // 获取材质的散射信息
            let mat = match &rec.mat {
                Some(m) => m,
                None => return Color::new(0.0, 0.0, 0.0),
            };

            let mut scattered = Ray::default();
            let mut attenuation = Color::default();
            let mut rng = thread_rng();

            // 首先获取材质的发光颜色
            let color_from_emission = mat.emitted(rec.u, rec.v, &rec.p);
            
            if mat.scatter(r, &rec, &mut attenuation, &mut scattered, &mut rng) {
                // 俄罗斯轮盘赌判断是否继续追踪
                let continue_prob = self.russian_roulette_probability(&attenuation, depth);
                
                if thread_rng().gen::<f64>() < continue_prob {
                    // 继续递归，按概率缩放颜色以保持无偏性
                    let color_contribution = self.ray_color_with_rr(&scattered, depth - 1, world);
                    color_from_emission + attenuation * color_contribution / continue_prob
                } else {
                    // 终止路径，但仍然返回发光颜色
                    color_from_emission
                }
            } else {
                // 对于不散射的材质（如光源），返回发光颜色
                color_from_emission
            }
        } else {
            // 如果没有击中物体，返回背景颜色
            self.background
        }
    }

    // 辅助函数：将Color转换为RGB u8数组 - 内联版本已移至render_sub中优化



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

    // 俄罗斯轮盘赌优化的替代实现
    // 可以通过修改此函数来使用不同的终止策略
    fn russian_roulette_probability(&self, attenuation: &Color, depth: usize) -> f64 {
        match self.russian_roulette {
            RussianRouletteStrategy::None => 1.0, // 总是继续
            
            RussianRouletteStrategy::HighQuality => {
                // 高质量策略：极致的质量保证，减少噪点到最低
                if depth < 15 {
                    return 1.0; // 前15层总是继续，确保最高质量
                }
                
                // 使用感知亮度公式，更准确反映人眼敏感度
                let perceptual_luminance = 0.2126 * attenuation.x + 0.7152 * attenuation.y + 0.0722 * attenuation.z;
                let max_component = attenuation.x.max(attenuation.y).max(attenuation.z);
                
                // 考虑最大分量和感知亮度的加权平均，更全面评估重要性
                let brightness = 0.7 * perceptual_luminance + 0.3 * max_component;
                
                // 极其温和的深度衰减，使用平滑曲线
                let depth_excess = (depth as f64 - 15.0).max(0.0);
                let depth_factor = 1.0 / (1.0 + depth_excess * 0.008); // 更平滑的衰减
                
                // 非常高的最小概率，确保最少噪点
                (brightness * depth_factor).clamp(0.75, 1.0)
            },
            
            RussianRouletteStrategy::Conservative => {
                // 保守策略：在质量和性能间平衡，仍偏向质量
                if depth < 10 {
                    return 1.0; // 前10层总是继续，保证质量
                }
                
                let perceptual_luminance = 0.2126 * attenuation.x + 0.7152 * attenuation.y + 0.0722 * attenuation.z;
                let max_component = attenuation.x.max(attenuation.y).max(attenuation.z);
                let brightness = 0.6 * perceptual_luminance + 0.4 * max_component;
                
                // 使用平滑的指数衰减
                let depth_excess = (depth as f64 - 10.0).max(0.0);
                let depth_factor = (-depth_excess * 0.02).exp(); // 指数衰减，更平滑
                
                // 高最小概率，减少噪点
                (brightness * (0.3 + 0.7 * depth_factor)).clamp(0.5, 1.0)
            },
            
            RussianRouletteStrategy::Aggressive => {
                // 激进策略：更早开始终止，但保持合理的质量阈值
                if depth < 6 {
                    return 1.0; // 前6层总是继续
                }
                
                let perceptual_luminance = 0.2126 * attenuation.x + 0.7152 * attenuation.y + 0.0722 * attenuation.z;
                let max_component = attenuation.x.max(attenuation.y).max(attenuation.z);
                let brightness = 0.5 * perceptual_luminance + 0.5 * max_component;
                
                // 中等程度的深度衰减
                let depth_excess = (depth as f64 - 6.0).max(0.0);
                let depth_factor = (-depth_excess * 0.04).exp();
                
                // 中等最小概率
                (brightness * (0.2 + 0.8 * depth_factor)).clamp(0.3, 1.0)
            },
            
            RussianRouletteStrategy::Adaptive => {
                // 自适应策略：最智能的质量优化，动态调整所有参数
                let importance = self.sample_importance(attenuation, depth);
                let perceptual_luminance = 0.2126 * attenuation.x + 0.7152 * attenuation.y + 0.0722 * attenuation.z;
                let max_component = attenuation.x.max(attenuation.y).max(attenuation.z);
                
                // 基于材质特性的智能评估
                let brightness = 0.6 * perceptual_luminance + 0.4 * max_component;
                let material_strength = (brightness + max_component) * 0.5;
                
                // 重要性因子，用于调整所有参数
                let importance_factor = (importance * 1.5).clamp(0.6, 1.8);
                
                // 智能深度阈值：根据材质强度和重要性动态调整
                let base_threshold = if material_strength > 0.8 { 
                    14  // 强反射材质：更深追踪
                } else if material_strength > 0.6 { 
                    11 
                } else if material_strength > 0.4 { 
                    8 
                } else if material_strength > 0.2 { 
                    6 
                } else { 
                    4  // 弱反射材质：早期终止
                };
                
                // 重要性调整的动态阈值
                let adaptive_threshold = (base_threshold as f64 * importance_factor) as usize;
                
                if depth < adaptive_threshold {
                    return 1.0;
                }
                
                // 平滑的深度衰减，考虑重要性
                let depth_excess = (depth as f64 - adaptive_threshold as f64).max(0.0);
                let decay_rate = 0.015 / importance_factor; // 重要性高的样本衰减更慢
                let depth_factor = (-depth_excess * decay_rate).exp();
                
                // 自适应最小概率：重要性高的样本有更高的基础概率
                let min_prob = (0.2 + 0.3 * importance_factor / 1.8).min(0.6);
                let adaptive_prob = material_strength * importance_factor * (min_prob + (1.0 - min_prob) * depth_factor);
                
                adaptive_prob.clamp(min_prob, 1.0)
            }
        }
    }

    // 设置俄罗斯轮盘赌策略
    pub fn set_russian_roulette_strategy(&mut self, strategy: RussianRouletteStrategy) {
        self.russian_roulette = strategy;
    }

    // 获取当前俄罗斯轮盘赌策略
    pub fn get_russian_roulette_strategy(&self) -> RussianRouletteStrategy {
        self.russian_roulette
    }

    // 计算样本重要性，用于质量优化 - 增强版
    fn sample_importance(&self, attenuation: &Color, depth: usize) -> f64 {
        // 使用感知亮度和色彩饱和度来评估样本重要性
        let perceptual_luminance = 0.2126 * attenuation.x + 0.7152 * attenuation.y + 0.0722 * attenuation.z;
        let max_component = attenuation.x.max(attenuation.y).max(attenuation.z);
        let min_component = attenuation.x.min(attenuation.y).min(attenuation.z);
        
        // 色彩饱和度：高饱和度的样本通常更重要（如彩色反射）
        let saturation = if max_component > 0.0 {
            (max_component - min_component) / max_component
        } else {
            0.0
        };
        
        // 反射强度：考虑亮度和最大分量
        let reflectance_strength = (perceptual_luminance + max_component) * 0.5;
        
        // 深度重要性：浅层样本更重要，但使用平滑衰减
        let depth_importance = (-(depth as f64) * 0.08).exp();
        
        // 色彩活跃度：饱和度高的样本对最终图像贡献更大
        let color_activity = 0.7 * perceptual_luminance + 0.3 * saturation;
        
        // 综合重要性评分
        let base_importance = (reflectance_strength + color_activity) * 0.5;
        let final_importance = base_importance * depth_importance;
        
        // 确保重要性在合理范围内
        final_importance.clamp(0.1, 2.0)
    }

    // SIMD优化的射线生成 - 批量生成4条射线
    #[cfg(feature = "simd")]
    fn get_rays_simd_batch(&self, pixels: &[(usize, usize); 4], simd_rng: &mut SimdRng) -> [Ray; 4] {
        let random_offsets = simd_rng.next_f64x4() - f64x4::splat(0.5);
        let random_offsets2 = simd_rng.next_f64x4() - f64x4::splat(0.5);
        
        let mut rays = [Ray::default(); 4];
        
        for (idx, &(i, j)) in pixels.iter().enumerate() {
            let offset_x = random_offsets[idx];
            let offset_y = random_offsets2[idx];
            
            let pixel_sample = self.pixel00_loc
                + self.pixel_delta_u * (i as f64 + offset_x)
                + self.pixel_delta_v * (j as f64 + offset_y);

            let ray_origin = if self.defocus_angle <= 0.0 { 
                self.center 
            } else { 
                self.defocus_disk_sample() 
            };
            
            let ray_direction = pixel_sample - ray_origin;
            let ray_time = random_offsets[(idx + 2) % 4].abs(); // 重用随机数
            
            rays[idx] = Ray::new(ray_origin, ray_direction, ray_time);
        }
        
        rays
    }

    // SIMD优化的色彩计算 - 批量处理4个颜色值
    #[cfg(feature = "simd")]
    fn process_colors_simd(colors: &mut [Color; 4]) {
        // 将Color数组转换为SIMD向量进行并行处理
        let mut r_values = f64x4::from_array([colors[0].x, colors[1].x, colors[2].x, colors[3].x]);
        let mut g_values = f64x4::from_array([colors[0].y, colors[1].y, colors[2].y, colors[3].y]);
        let mut b_values = f64x4::from_array([colors[0].z, colors[1].z, colors[2].z, colors[3].z]);
        
        // 并行gamma校正 (sqrt)
        r_values = r_values.sqrt();
        g_values = g_values.sqrt();
        b_values = b_values.sqrt();
        
        // 并行钳制到[0, 1]范围
        let zero = f64x4::splat(0.0);
        let one = f64x4::splat(1.0);
        r_values = r_values.simd_clamp(zero, one);
        g_values = g_values.simd_clamp(zero, one);
        b_values = b_values.simd_clamp(zero, one);
        
        // 写回结果
        let r_array = r_values.to_array();
        let g_array = g_values.to_array();
        let b_array = b_values.to_array();
        
        for i in 0..4 {
            colors[i] = Color::new(r_array[i], g_array[i], b_array[i]);
        }
    }

    // SIMD优化的俄罗斯轮盘赌概率计算 - 批量处理
    #[cfg(feature = "simd")]
    fn russian_roulette_probabilities_simd(&self, attenuations: &[Color; 4], depths: &[usize; 4]) -> [f64; 4] {
        let mut probs = [0.0f64; 4];
        
        match self.russian_roulette {
            RussianRouletteStrategy::None => {
                return [1.0; 4]; // 所有都继续
            },
            
            RussianRouletteStrategy::HighQuality => {
                // 使用SIMD并行计算感知亮度
                let r_values = f64x4::from_array([attenuations[0].x, attenuations[1].x, attenuations[2].x, attenuations[3].x]);
                let g_values = f64x4::from_array([attenuations[0].y, attenuations[1].y, attenuations[2].y, attenuations[3].y]);
                let b_values = f64x4::from_array([attenuations[0].z, attenuations[1].z, attenuations[2].z, attenuations[3].z]);
                
                // 并行计算感知亮度
                let r_coeff = f64x4::splat(0.2126);
                let g_coeff = f64x4::splat(0.7152);
                let b_coeff = f64x4::splat(0.0722);
                let luminance = r_values * r_coeff + g_values * g_coeff + b_values * b_coeff;
                
                // 并行计算最大分量
                let max_rg = r_values.simd_max(g_values);
                let max_component = max_rg.simd_max(b_values);
                
                // 并行计算亮度
                let brightness_factor = f64x4::splat(0.7);
                let max_factor = f64x4::splat(0.3);
                let brightness = luminance * brightness_factor + max_component * max_factor;
                
                let brightness_array = brightness.to_array();
                
                for i in 0..4 {
                    if depths[i] < 15 {
                        probs[i] = 1.0;
                    } else {
                        let depth_excess = (depths[i] as f64 - 15.0).max(0.0);
                        let depth_factor = 1.0 / (1.0 + depth_excess * 0.008);
                        probs[i] = (brightness_array[i] * depth_factor).clamp(0.75, 1.0);
                    }
                }
            },
            
            _ => {
                // 对于其他策略，回退到标量实现
                for i in 0..4 {
                    probs[i] = self.russian_roulette_probability(&attenuations[i], depths[i]);
                }
            }
        }
        
        probs
    }

    // SIMD优化的渲染子块函数 - 激进优化版本
    #[cfg(feature = "simd")]
    pub fn render_sub_simd(&self, world: &dyn Hittable, img_mtx: Arc<Mutex<&mut RgbImage>>,
                          x_min: usize, x_max: usize, y_min: usize, y_max: usize) {
        
        // 检查边界条件，避免下溢错误
        if x_min >= x_max || y_min >= y_max {
            return;
        }
        
        let mut simd_rng = SimdRng::new();
        let width = x_max - x_min;
        let height = y_max - y_min;
        
        // 预分配缓冲区
        let mut write_buffer: Vec<Vec<Color>> = Vec::with_capacity(height);
        for _ in 0..height {
            write_buffer.push(vec![Color::new(0.0, 0.0, 0.0); width]);
        }
        
        let inv_samples = 1.0 / (self.sqrt_spp * self.sqrt_spp) as f64;
        
        // SIMD批量处理像素 - 每次处理4个像素
        for j in y_min..y_max {
            let local_j = j - y_min;
            let mut i = x_min;
            
            while i + 4 <= x_max {
                let local_i = i - x_min;
                let mut pixel_colors = [Color::new(0.0, 0.0, 0.0); 4];
                
                // 为4个像素生成样本
                for s_j in 0..self.sqrt_spp {
                    for s_i in 0..self.sqrt_spp {
                        // 批量生成4条射线
                        let pixels = [(i, j), (i+1, j), (i+2, j), (i+3, j)];
                        let rays = self.get_rays_simd_batch(&pixels, &mut simd_rng);
                        
                        // 批量追踪射线 (这里仍需要单独处理，因为世界交互复杂)
                        for (ray_idx, ray) in rays.iter().enumerate() {
                            if i + ray_idx < x_max {
                                let color = ray_color_with_rr_simd(self, ray, self.max_depth, world, self.background, &mut simd_rng);
                                pixel_colors[ray_idx] += color;
                            }
                        }
                    }
                }
                
                // SIMD批量处理最终颜色
                Self::process_colors_simd(&mut pixel_colors);
                
                // 写入缓冲区
                for (offset, &color) in pixel_colors.iter().enumerate() {
                    if local_i + offset < width {
                        write_buffer[local_j][local_i + offset] = Color::new(
                            color.x * inv_samples,
                            color.y * inv_samples,
                            color.z * inv_samples
                        );
                    }
                }
                
                i += 4;
            }
            
            // 处理剩余的像素 (非4的倍数)
            while i < x_max {
                let local_i = i - x_min;
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                
                for s_j in 0..self.sqrt_spp {
                    for s_i in 0..self.sqrt_spp {
                        let random_vals = simd_rng.next_f64x4();
                        let offset_x = (s_i as f64 + random_vals[0]) / self.sqrt_spp as f64 - 0.5;
                        let offset_y = (s_j as f64 + random_vals[1]) / self.sqrt_spp as f64 - 0.5;
                        
                        let pixel_sample = self.pixel00_loc
                            + self.pixel_delta_u * (i as f64 + offset_x)
                            + self.pixel_delta_v * (j as f64 + offset_y);

                        let ray_origin = if self.defocus_angle <= 0.0 { 
                            self.center 
                        } else { 
                            self.defocus_disk_sample() 
                        };
                        let ray_direction = pixel_sample - ray_origin;
                        let ray_time = random_vals[2].abs();

                        let ray = Ray::new(ray_origin, ray_direction, ray_time);
                        pixel_color += ray_color_with_rr_simd(self, &ray, self.max_depth, world, self.background, &mut simd_rng);
                    }
                }
                
                write_buffer[local_j][local_i] = Color::new(
                    pixel_color.x * inv_samples,
                    pixel_color.y * inv_samples,
                    pixel_color.z * inv_samples
                );
                
                i += 1;
            }
        }

        // 写入图像 - 使用SIMD批量转换RGB
        {
            let mut binding = img_mtx.lock().unwrap();
            let img: &mut RgbImage = *binding;
            
            for j in 0..height {
                let mut i = 0;
                
                // SIMD批量处理RGB转换
                while i + 4 <= width {
                    let colors = [
                        write_buffer[j][i],
                        write_buffer[j][i+1], 
                        write_buffer[j][i+2],
                        write_buffer[j][i+3]
                    ];
                    
                    // 并行gamma校正和RGB转换
                    let r_vals = f64x4::from_array([colors[0].x, colors[1].x, colors[2].x, colors[3].x]);
                    let g_vals = f64x4::from_array([colors[0].y, colors[1].y, colors[2].y, colors[3].y]);
                    let b_vals = f64x4::from_array([colors[0].z, colors[1].z, colors[2].z, colors[3].z]);
                    
                    let gamma_r = r_vals.sqrt();
                    let gamma_g = g_vals.sqrt();
                    let gamma_b = b_vals.sqrt();
                    
                    let zero = f64x4::splat(0.0);
                    let one = f64x4::splat(1.0);
                    let scale = f64x4::splat(255.0);
                    
                    let final_r = (gamma_r.simd_clamp(zero, one) * scale).cast::<u8>();
                    let final_g = (gamma_g.simd_clamp(zero, one) * scale).cast::<u8>();
                    let final_b = (gamma_b.simd_clamp(zero, one) * scale).cast::<u8>();
                    
                    // 写入像素
                    for k in 0..4 {
                        if i + k < width {
                            let write_x = (i + k + x_min) as u32;
                            let write_y = (j + y_min) as u32;
                            let rgb = [final_r[k], final_g[k], final_b[k]];
                            img.put_pixel(write_x, write_y, image::Rgb(rgb));
                        }
                    }
                    
                    i += 4;
                }
                
                // 处理剩余像素
                while i < width {
                    let write_x = (i + x_min) as u32;
                    let write_y = (j + y_min) as u32;
                    let color = &write_buffer[j][i];
                    let rgb = [
                        (color.x.sqrt().clamp(0.0, 1.0) * 255.0) as u8,
                        (color.y.sqrt().clamp(0.0, 1.0) * 255.0) as u8,
                        (color.z.sqrt().clamp(0.0, 1.0) * 255.0) as u8
                    ];
                    img.put_pixel(write_x, write_y, image::Rgb(rgb));
                    i += 1;
                }
            }
        }
        
        // 只有在实际渲染了内容时才更新进度条
        if width > 0 && height > 0 {
            self.bar.inc(1);
        }
    }
}

// SIMD优化的光线颜色计算函数 - 移到模块级别
#[cfg(feature = "simd")]
fn ray_color_with_rr_simd(camera: &Camera, r: &Ray, depth: usize, world: &dyn Hittable, background: Color, simd_rng: &mut SimdRng) -> Color {
    // If we've exceeded the ray bounce limit, no more light is gathered.
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let mut rec = HitRecord::default();

    // If the ray hits nothing, return the background color.
    if !world.hit(r, Interval::new(0.001, f64::INFINITY), &mut rec) {
        return background;
    }

    let mat = match &rec.mat {
        Some(m) => m,
        None => return Color::new(0.0, 0.0, 0.0),
    };

    let color_from_emission = mat.emitted(rec.u, rec.v, &rec.p);

    let mut scattered = Ray::default();
    let mut attenuation = Color::default();
    
    // 使用标准ThreadRng而不是CustomSimdRng，因为Material trait需要ThreadRng
    let mut thread_rng = thread_rng();

    if !mat.scatter(r, &rec, &mut attenuation, &mut scattered, &mut thread_rng) {
        return color_from_emission;
    }

    // 使用SIMD优化的俄罗斯轮盘赌概率计算
    let continue_prob = camera.russian_roulette_probability(&attenuation, depth);
    
    // 俄罗斯轮盘赌判断
    let next_random = simd_rng.next_f64x4()[0];
    if next_random > continue_prob {
        return color_from_emission;
    }
    
    // 继续追踪，需要根据概率调整颜色强度以保持无偏估计
    let color_from_scatter = if continue_prob < 1.0 {
        (attenuation * ray_color_with_rr_simd(camera, &scattered, depth - 1, world, background, simd_rng)) / continue_prob
    } else {
        attenuation * ray_color_with_rr_simd(camera, &scattered, depth - 1, world, background, simd_rng)
    };

    color_from_emission + color_from_scatter
}

