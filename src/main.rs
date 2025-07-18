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
mod bvh;
mod texture;
mod rtw_stb_image;
mod perlin;
mod quad;
mod constant_medium;
mod onb;
mod pdf;

use std::time::Instant;
use crate::color::write_color;
use crate::ray::Ray;
use crate::rtweekend::{degrees_to_radians, INFINITY};
use crate::rtweekend::random::{random_double, random_double_range};
use crate::rtweekend::time_it;
use crate::vec3::{Color, Point3,Vec3};
use crate::interval::Interval;
use crate::camera::{Camera, RussianRouletteStrategy};
use crate::sphere::Sphere;
use crate::hittable::{HitRecord,Hittable,HittableList};
use crate::material::{Material,Lambertian,Metal,Dielectric,NumberMaterial};
use crate::bvh::BvhNode;
use crate::texture::{Texture, SolidColor, CheckerTexture,NoiseTexture};
use crate::quad::Quad;
use crate::quad::make_box;
 use crate::hittable::{RotateY, Translate};
use crate::constant_medium::ConstantMedium;
use crate::material::DiffuseLight;
use crate::texture::ImageTexture;

fn bouncing_spheres() {
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

    // 棋盘格地面
    let checker = Arc::new(CheckerTexture::from_color(
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));
    let ground_material = Arc::new(Lambertian::from_texture(checker));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Some(ground_material),
    )));

    // 小球遍历 - 优化内存分配
    // 预分配一些常用材质，避免重复创建相同的材质
    let glass_material = Arc::new(Dielectric::new(1.5));
    
    // 预计算固定点，避免重复计算
    let reference_point = Point3::new(4.0, 0.2, 0.0);
    let sphere_radius = 0.2;
    let sphere_height = 0.2;
    
    for a in -11..11 {
        let a_f64 = a as f64; // 避免重复类型转换
        for b in -11..11 {
            let b_f64 = b as f64; // 避免重复类型转换
            let choose_mat = random_double();
            let center = Point3::new(
                a_f64 + 0.9 * random_double(),
                sphere_height,
                b_f64 + 0.9 * random_double(),
            );
            if (center - reference_point).length() > 0.9 {
                if choose_mat < 0.8 {
                    // 漫反射
                    let albedo = Color::random() * Color::random();
                    let sphere_material = Arc::new(Lambertian::from_color(albedo));
                    let center2 = center + Vec3::new(0.0, random_double_range(0.0, 0.5), 0.0);
                    world.add(Arc::new(Sphere::moving(center, center2, sphere_radius, Some(sphere_material))));
                } else if choose_mat < 0.95 {
                    // 金属
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = random_double_range(0.0, 0.5);
                    let sphere_material = Arc::new(Metal::new(albedo, fuzz));
                    world.add(Arc::new(Sphere::new(center, sphere_radius, Some(sphere_material))));
                } else {
                    // 玻璃 - 重用预分配的材质
                    world.add(Arc::new(Sphere::new(center, sphere_radius, Some(glass_material.clone()))));
                }
            }
        }
    }

    // 大球们
    let material1 = Arc::new(Dielectric::new(1.5));
    world.add(Arc::new(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, Some(material1))));

    let material2 = Arc::new(Lambertian::from_color(Color::new(0.4, 0.2, 0.1)));
    world.add(Arc::new(Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, Some(material2))));

    let material3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Arc::new(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, Some(material3))));

    world = HittableList::with_object(Arc::new(BvhNode::new_from_list(&mut world.objects)));

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
    cam.background= Color::new(0.70, 0.80, 1.00);
    
    // 设置俄罗斯轮盘赌策略 - 为复杂场景选择最佳策略
    // HighQuality: 最高质量，最少噪点（默认）
    // Conservative: 平衡质量和性能
    // Adaptive: 智能自适应，根据材质动态调整
    // Aggressive: 追求性能，可能有轻微噪点 - 与SIMD优化配合效果最佳
    cam.set_russian_roulette_strategy(RussianRouletteStrategy::Aggressive);
    
    // SIMD优化会在支持AVX2的CPU上自动启用，无需额外配置
    // 编译时使用: RUSTFLAGS="-C target-cpu=native" cargo build --release

    // 渲染
    let lights = HittableList::new();
    let (duration, result) = time_it(|| cam.render(&world, &lights, &mut std::io::stdout()).unwrap());
    let mut file = OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "Render耗时: {:?}", duration).unwrap();
}

fn checkered_spheres() {
    let mut world = HittableList::new();

    let checker = Arc::new(CheckerTexture::from_color(
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));

    let mat_checker = Arc::new(Lambertian::from_texture(checker));
    // 避免重复clone，直接创建两个球体时复用引用
    let sphere1 = Arc::new(Sphere::new(
        Point3::new(0.0, -10.0, 0.0),
        10.0,
        Some(mat_checker.clone()),
    ));
    let sphere2 = Arc::new(Sphere::new(
        Point3::new(0.0, 10.0, 0.0),
        10.0,
        Some(mat_checker),
    ));
    world.add(sphere1);
    world.add(sphere2);

    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth = 50;
    cam.vfov = 20.0;
    cam.lookfrom = Point3::new(13.0, 2.0, 3.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);
    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;
    cam.background= Color::new(0.70, 0.80, 1.00);

    // 简单场景使用保守策略
    cam.set_russian_roulette_strategy(RussianRouletteStrategy::Conservative);

    let lights = HittableList::new();
    let (duration, result) = time_it(|| cam.render(&world, &lights, &mut std::io::stdout()).unwrap());
    let mut file = OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "Checkered Render耗时: {:?}", duration).unwrap();
}

fn earth_sphere() {
    use crate::texture::ImageTexture;
    let mut world = HittableList::new();
    let earth_texture = Arc::new(ImageTexture::new("input/earthmap.jpg"));
    let earth_surface = Arc::new(Lambertian::from_texture(earth_texture));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        2.0,
        Some(earth_surface),
    )));

    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth = 50;
    cam.vfov = 20.0;
    cam.lookfrom = Point3::new(0.0, 0.0, 12.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);
    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;
    cam.background= Color::new(0.70, 0.80, 1.00);

    // 纹理场景使用保守策略
    cam.set_russian_roulette_strategy(RussianRouletteStrategy::Conservative);

    let lights = HittableList::new();
    let (duration, result) = time_it(|| cam.render(&world, &lights, &mut std::io::stdout()).unwrap());
    let mut file = OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "Earth Render耗时: {:?}", duration).unwrap();
}

fn perlin_spheres() {
    let mut world = HittableList::new();

    let pertext: Arc<dyn Texture + Send + Sync> = Arc::new(texture::NoiseTexture::with_scale(4.0));
    let pertext_material = Arc::new(Lambertian::from_texture(pertext));
    
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Some(pertext_material.clone()),
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        Some(pertext_material),
    )));

    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth = 50;
    cam.vfov = 20.0;
    cam.lookfrom = Point3::new(13.0, 2.0, 3.0);
    cam.lookat = Point3::default();
    cam.vup = Vec3::new(0.0, 1.0, 0.0);
    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;
    cam.background= Color::new(0.70, 0.80, 1.00);

    // 噪声纹理场景使用自适应策略
    cam.set_russian_roulette_strategy(RussianRouletteStrategy::Adaptive);

    let lights = HittableList::new();
    let (duration, result) = time_it(|| cam.render(&world, &lights, &mut std::io::stdout()).unwrap());
    let mut file = OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "Perlin Render耗时: {:?}", duration).unwrap();
}

fn quads() {
    let mut world = HittableList::new();

    // 材质
    let left_red     = Arc::new(Lambertian::from_color(Color::new(1.0, 0.2, 0.2)));
    let back_green   = Arc::new(Lambertian::from_color(Color::new(0.2, 1.0, 0.2)));
    let right_blue   = Arc::new(Lambertian::from_color(Color::new(0.2, 0.2, 1.0)));
    let upper_orange = Arc::new(Lambertian::from_color(Color::new(1.0, 0.5, 0.0)));
    let lower_teal   = Arc::new(Lambertian::from_color(Color::new(0.2, 0.8, 0.8)));

    // 四边形
    world.add(Arc::new(Quad::new(
        Point3::new(-3.0, -2.0, 5.0),
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 4.0, 0.0),
        left_red,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(-2.0, -2.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
        back_green,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(3.0, -2.0, 1.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 4.0, 0.0),
        right_blue,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(-2.0, 3.0, 1.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        upper_orange,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(-2.0, -3.0, 5.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -4.0),
        lower_teal,
    )));

    // 相机
    let mut cam = Camera::new();
    cam.aspect_ratio = 1.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth = 50;
    cam.vfov = 80.0;
    cam.lookfrom = Point3::new(0.0, 0.0, 9.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);
    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;
    cam.background= Color::new(0.70, 0.80, 1.00);

    // 几何测试场景使用保守策略
    cam.set_russian_roulette_strategy(RussianRouletteStrategy::Conservative);

    let lights = HittableList::new();
    let (duration, _) = crate::rtweekend::time_it(|| cam.render(&world, &lights, &mut std::io::stdout()));
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "Quads Render耗时: {:?}", duration).unwrap();
}

fn simple_light() {
    let mut world = HittableList::new();

    // 噪声纹理
    let pertext: Arc<dyn Texture + Send + Sync> = Arc::new(NoiseTexture::with_scale(4.0));
    let lambertian_pertext = Arc::new(Lambertian::from_texture(pertext));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Some(lambertian_pertext.clone()),
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        Some(lambertian_pertext),
    )));

    // 发光材质
    let difflight = Arc::new(DiffuseLight::from_color(Color::new(4.0, 4.0, 4.0)));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 7.0, 0.0),
        2.0,
        Some(difflight.clone()),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        difflight.clone(),
    )));

    // 相机
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth = 50;
    cam.background = Color::new(0.0, 0.0, 0.0);

    cam.vfov = 20.0;
    cam.lookfrom = Point3::new(26.0, 3.0, 6.0);
    cam.lookat = Point3::new(0.0, 2.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);
    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;
    
    // simple_light场景使用保守策略，平衡质量和性能
    cam.set_russian_roulette_strategy(RussianRouletteStrategy::Conservative);

    // 构建光源列表
    let mut lights = HittableList::new();
    lights.add(Arc::new(Quad::new(
        Point3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        difflight.clone(),
    )));
    lights.add(Arc::new(Sphere::new(
        Point3::new(0.0, 7.0, 0.0),
        2.0,
        Some(difflight.clone()),
    )));
    let (duration, _) = crate::rtweekend::time_it(|| cam.render(&world, &lights, &mut std::io::stdout()));
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "SimpleLight Render耗时: {:?}", duration).unwrap();
}

fn cornell_box() {
    let mut world = HittableList::new();

    let red   = Arc::new(Lambertian::from_color(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::from_color(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::from_color(Color::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::from_color(Color::new(15.0, 15.0, 15.0)));

    world.add(Arc::new(Quad::new(
        Point3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        white.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        white.clone(),
    )));


    // Box
    let box1 = make_box(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        white.clone(),
    );
    let box1 = Arc::new(RotateY::new(box1, 15.0));
    let box1 = Arc::new(Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));
    world.add(box1);

    // Glass Sphere
    let glass = Arc::new(Dielectric::new(1.5));
    world.add(Arc::new(Sphere::new(
        Point3::new(190.0, 90.0, 190.0),
        90.0,
        Some(glass),
    )));

    let mut cam = Camera::new();
    cam.aspect_ratio = 1.0;
    cam.image_width = 600;
    cam.samples_per_pixel = 10000;
    cam.max_depth = 50;
    cam.background = Color::new(0.0, 0.0, 0.0);

    cam.vfov = 40.0;
    cam.lookfrom = Point3::new(278.0, 278.0, -800.0);
    cam.lookat = Point3::new(278.0, 278.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);
    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;

    // Cornell box场景需要高质量的光线追踪，使用保守策略
    cam.set_russian_roulette_strategy(RussianRouletteStrategy::None);

    // 关闭BVH: 不再用BvhNode封装world，直接用HittableList

    // 构建Cornell Box光源列表
    let mut lights = HittableList::new();
    // 使用空材质作为光源列表的标记（不影响采样，只用于PDF/importance sampling）
    let empty_material: Arc<dyn Material + Send + Sync> = Arc::new(Lambertian::from_color(Color::new(0.0, 0.0, 0.0)));
    lights.add(Arc::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        empty_material.clone(),
    )));
    lights.add(Arc::new(Sphere::new(
        Point3::new(190.0, 90.0, 190.0),
        90.0,
        Some(empty_material.clone()),
    )));
    let (duration, _) = crate::rtweekend::time_it(|| cam.render(&world, &lights, &mut std::io::stdout()));
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "CornellBox Render耗时: {:?}", duration).unwrap();
}

fn cornell_smoke() {
    let mut world = HittableList::new();

    let red   = Arc::new(Lambertian::from_color(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::from_color(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::from_color(Color::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::from_color(Color::new(7.0, 7.0, 7.0)));

    world.add(Arc::new(Quad::new(
        Point3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(113.0, 554.0, 127.0),
        Vec3::new(330.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 305.0),
        light.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 555.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        white.clone(),
    )));

    let box1 = make_box(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        white.clone(),
    );
    let box1 = Arc::new(RotateY::new(box1, 15.0));
    let box1 = Arc::new(Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));

    let box2 = make_box(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 165.0, 165.0),
        white.clone(),
    );
    let box2 = Arc::new(RotateY::new(box2, -18.0));
    let box2 = Arc::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));

    world.add(Arc::new(ConstantMedium::new_with_color(
        box1,
        0.01,
        Color::new(0.0, 0.0, 0.0),
    )));
    world.add(Arc::new(ConstantMedium::new_with_color(
        box2,
        0.01,
        Color::new(1.0, 1.0, 1.0),
    )));

    let mut cam = Camera::new();
    cam.aspect_ratio = 1.0;
    cam.image_width = 600;
    cam.samples_per_pixel = 200;
    cam.max_depth = 50;
    cam.background = Color::new(0.0, 0.0, 0.0);

    cam.vfov = 40.0;
    cam.lookfrom = Point3::new(278.0, 278.0, -800.0);
    cam.lookat = Point3::new(278.0, 278.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);
    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;

    // 体积渲染场景使用自适应策略
    cam.set_russian_roulette_strategy(RussianRouletteStrategy::Adaptive);

    let lights = HittableList::new();
    let (duration, _) = crate::rtweekend::time_it(|| cam.render(&world, &lights, &mut std::io::stdout()));
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "CornellSmoke Render耗时: {:?}", duration).unwrap();
}

fn former_final_scene(image_width: usize, samples_per_pixel: usize, max_depth: usize) {
    let mut boxes = HittableList::new();
    let ground = Arc::new(Lambertian::from_color(Color::new(0.48, 0.83, 0.53)));

    let boxes_per_side = 20;
    // 预计算常量，避免重复计算
    let w = 100.0;
    let base_x = -1000.0;
    let base_z = -1000.0;
    let y0 = 0.0;
    
    for i in 0..boxes_per_side {
        let x0 = base_x + i as f64 * w;
        let x1 = x0 + w;
        for j in 0..boxes_per_side {
            let z0 = base_z + j as f64 * w;
            let z1 = z0 + w;
            let y1 = random_double_range(1.0, 101.0);

            // 避免在循环中clone，直接使用引用
            boxes.addlist(make_box(
                Point3::new(x0, y0, z0),
                Point3::new(x1, y1, z1),
                ground.clone(),
            ));
        }
    }

    let mut world = HittableList::new();
    world.add(Arc::new(BvhNode::new_from_list(&mut boxes.objects)));

    let light = Arc::new(DiffuseLight::from_color(Color::new(7.0, 7.0, 7.0)));
    world.add(Arc::new(Quad::new(
        Point3::new(123.0, 554.0, 147.0),
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 265.0),
        light,
    )));

    let center1 = Point3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
    let sphere_material = Arc::new(Lambertian::from_color(Color::new(0.7, 0.3, 0.1)));
    world.add(Arc::new(Sphere::moving(center1, center2, 50.0, Some(sphere_material))));

    // 预分配常用材质，避免重复创建
    let dielectric_1_5 = Arc::new(Dielectric::new(1.5));
    let metal_material = Arc::new(Metal::new(Color::new(0.8, 0.8, 0.9), 1.0));
    
    world.add(Arc::new(Sphere::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        Some(dielectric_1_5.clone()),
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 150.0, 145.0),
        50.0,
        Some(metal_material),
    )));

    let boundary = Arc::new(Sphere::new(
        Point3::new(360.0, 150.0, 145.0),
        70.0,
        Some(dielectric_1_5.clone()),
    ));
    world.add(boundary.clone());
    world.add(Arc::new(ConstantMedium::new_with_color(
        boundary.clone(),
        0.2,
        Color::new(0.2, 0.4, 0.9),
    )));

    let boundary2 = Arc::new(Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        5000.0,
        Some(dielectric_1_5.clone()),
    ));
    world.add(Arc::new(ConstantMedium::new_with_color(
        boundary2,
        0.0001,
        Color::new(1.0, 1.0, 1.0),
    )));

    let earth_texture = Arc::new(ImageTexture::new("input/earthmap.jpg"));
    let emat = Arc::new(Lambertian::from_texture(earth_texture));
    world.add(Arc::new(Sphere::new(
        Point3::new(400.0, 200.0, 400.0),
        100.0,
        Some(emat),
    )));

    let pertext: Arc<dyn Texture + Send + Sync> = Arc::new(NoiseTexture::with_scale(0.2));
    let pertext_material = Arc::new(Lambertian::from_texture(pertext));
    world.add(Arc::new(Sphere::new(
        Point3::new(220.0, 280.0, 300.0),
        80.0,
        Some(pertext_material),
    )));

    let mut boxes2 = HittableList::new();
    let white = Arc::new(Lambertian::from_color(Color::new(0.73, 0.73, 0.73)));
    let ns = 1000;
    // 批量创建球体，避免重复clone
    boxes2.objects.reserve(ns); // 预分配容量，避免动态扩容
    for _ in 0..ns {
        boxes2.add(Arc::new(Sphere::new(
            Point3::random_range(0.0, 165.0),
            10.0,
            Some(white.clone()),
        )));
    }

    world.add(Arc::new(Translate::new(
        Arc::new(RotateY::new(
            Arc::new(BvhNode::new_from_list(&mut boxes2.objects)),
            15.0,
        )),
        Vec3::new(-100.0, 270.0, 395.0),
    )));

    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 1600;
    cam.samples_per_pixel = samples_per_pixel;
    cam.max_depth = max_depth;
    cam.background = Color::new(0.0, 0.0, 0.0);

    cam.vfov = 40.0;
    cam.lookfrom = Point3::new(478.0, 278.0, -600.0);
    cam.lookat = Point3::new(278.0, 278.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);
    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;
    
    // Final scene是最复杂的场景，使用智能自适应策略获得最佳质量
    cam.set_russian_roulette_strategy(RussianRouletteStrategy::Adaptive);

    let lights = HittableList::new();
    let (duration, _) = crate::rtweekend::time_it(|| cam.render(&world, &lights, &mut std::io::stdout()));
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "FinalScene Render耗时: {:?}", duration).unwrap();
}

fn final_scene_1(image_width: usize, samples_per_pixel: usize, max_depth: usize) {
    // 预先加载所有数字图片材质，数字部分发光，背景为石板色
    let mut number_textures = Vec::new();
    // 石板颜色（黑色）
    let slab_color = Color::new(0.0, 0.0, 0.0);
    // 发光色（加深红色）
    let emit_color = Color::new(16.0, 0.02, 0.02);
    for n in 1..=11 {
        let filename = format!("input/{:02}-removebg-preview.png", (n+2)%13);
        let base_tex = Arc::new(ImageTexture::new(&filename));
        // 不再发光，仅普通漫反射
        let mat = Arc::new(Lambertian::from_texture(base_tex));
        number_textures.push(mat);
    }
    use std::f64::consts::PI;
    let mut world = HittableList::new();


    // 圆形排列参数
    let box_count = 12;
    let radius = 2.0 * image_width as f64 / 800.0 * 320.0;
    let box_height = 400.0;   // 降低高度
    let box_width = 1.8 * image_width as f64 / 800.0 *100.0;   // 增加宽度
    let box_depth = 16.0;    // 减小厚度，变为石板
    let base_y = 200.0;
    let top_y = base_y + box_height;

    // 材质
    let box_color = Arc::new(Lambertian::from_color(slab_color)); // 石板色
    let red_light = Arc::new(DiffuseLight::from_color(Color::new(8.0, 0.1, 0.1)));
    let white_light = Arc::new(DiffuseLight::from_color(Color::new(4.0, 4.0, 4.0)));
    // 背景色为蓝黑色
    let background_color = Color::new(0.05, 0.10, 0.20);


    // 生成长方体
    let mut slab_idx = 0;
    let strip_light_mat = Arc::new(DiffuseLight::from_color(Color::new(8.0, 8.0, 8.0)));
    for i in 0..box_count {
        if  i == box_count-3 {
            continue;
        }
        let theta = 2.0 * PI * (i as f64) / (box_count as f64);
        let x = 278.0 + radius * theta.cos();
        let z = 278.0 + radius * theta.sin();
        let angle = -theta * 180.0 / PI + 90.0; // 使正面朝向圆心

        // 长方体
        let box_obj = make_box(
            Point3::new(-box_width/2.0, base_y, -box_depth/2.0),
            Point3::new(box_width/2.0, top_y, box_depth/2.0),
            box_color.clone(),
        );
        let box_obj = Arc::new(RotateY::new(box_obj, angle));
        let box_obj = Arc::new(Translate::new(box_obj, Vec3::new(x, 0.0, z)));
        world.add(box_obj.clone());

        // 贴数字图片（背面，3/4高度，宽度与石板一致，高度为石板1/3，略微贴近表面）
        if slab_idx < number_textures.len() {
            let num_mat = number_textures[slab_idx].clone();
            let num_height = box_height / 3.0;
            let num_width = box_width * 0.8;
            let y_center = base_y + box_height * 0.75;
            // 背面中心点（未旋转前，-z方向）
            let quad_origin = Point3::new(-num_width/2.0, y_center - num_height/2.0, -box_depth/2.0 - 0.1); // 0.1略微浮出背面
            let u_vec = Vec3::new(num_width, 0.0, 0.0);
            let v_vec = Vec3::new(0.0, num_height, 0.0); // 取反法线
            // 使法线朝外（-z），Quad默认法线为u x v方向，这里顺序不变即可
            let rot = RotateY::new(Arc::new(Quad::new(quad_origin, u_vec, v_vec, num_mat)), angle);
            let quad = Translate::new(Arc::new(rot), Vec3::new(x, 0.0, z));
            world.add(Arc::new(quad));
            slab_idx += 1;
        }

        // 底部条状光源（正面底部，紧贴石板，略微浮出表面）
        let strip_width = box_width * 0.8;
        let strip_height = 6.0; // 条状光源高度
        let strip_y = base_y + 2.0; // 离地2个单位
        let strip_origin = Point3::new(-strip_width/2.0, strip_y, box_depth/2.0 + 0.2); // 正面，略微浮出
        let strip_u = Vec3::new(strip_width, 0.0, 0.0);
        let strip_v = Vec3::new(0.0, strip_height, 0.0);
        let strip_quad = Quad::new(strip_origin, strip_u, strip_v, strip_light_mat.clone());
        let strip_rot = RotateY::new(Arc::new(strip_quad), angle);
        let strip = Translate::new(Arc::new(strip_rot), Vec3::new(x, 0.0, z));
        world.add(Arc::new(strip));
    }

    // 去掉背景大球光源

    // 去掉平台（不添加地面大矩形）

    // 相机
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 1600;
    cam.samples_per_pixel = samples_per_pixel;
    cam.max_depth = max_depth;
    cam.background = Color::new(0.05, 0.10, 0.20);
    cam.vfov = 40.0;
    cam.lookfrom = Point3::new(278.0, 400.0, -800.0);
    cam.lookat = Point3::new(278.0, 120.0, 278.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);
    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;
    cam.set_russian_roulette_strategy(RussianRouletteStrategy::None);

    let lights = HittableList::new();
    let (duration, _) = crate::rtweekend::time_it(|| cam.render(&world, &lights, &mut std::io::stdout()));
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "FinalScene Render耗时: {:?}", duration).unwrap();
}

fn final_scene_2(image_width: usize, samples_per_pixel: usize, max_depth: usize) {
    // 融入final_scene_1的数字石板环（移到world定义后）
    // 融入final_scene_1的数字石板环（去除多余作用域）
    

    // 2001太空漫游三球日出场景
    use std::sync::Arc;
    use crate::texture::ImageTexture;
    let mut world = HittableList::new();


    let mut number_textures = Vec::new();
    let slab_color = Color::new(0.18, 0.18, 0.18); // 浅灰色石板
    let emit_color = Color::new(16.0, 0.02, 0.02);
    for n in 1..=11 {
        let filename = format!("input/{:02}.png", (n+2)%13);
        let base_tex = Arc::new(ImageTexture::new(&filename));
        // 直接用高albedo的Lambertian贴图，便于红色数字显示
        let mat = Arc::new(Lambertian::from_texture_with_albedo(base_tex, 0.7));
        number_textures.push(mat);
    }
    use std::f64::consts::PI;
    let box_count = 12;
    let radius = 2.0 * image_width as f64 / 1600.0 * 320.0;
    let box_height = 400.0;
    let box_width = 1.8 * image_width as f64 / 1600.0 * 100.0;
    let box_depth = 16.0;
    // 太阳球和木星球的中心y坐标
    let aspect = 16.0 / 9.0;
    let img_h = image_width as f64 / aspect;
    let r_sun = img_h * 0.08;
    let r_jupiter = img_h * 0.36;
    let sun_y = img_h * 0.73;
    let jupiter_y = sun_y - r_sun - r_jupiter + img_h * 0.005;
    // 石环圆心y与太阳球、木星球中心一致，base_y略高于太阳球底部
    let ring_center_y = (sun_y + jupiter_y) / 2.0;
    let ring_center_y = (sun_y + jupiter_y) / 2.0 + 320.0;
    let base_y = ring_center_y - box_height / 2.0 + r_sun * 0.15; // 整体上移230像素
    let top_y = base_y + box_height;
    let box_color = Arc::new(Lambertian::from_color(slab_color));
    let strip_light_mat = Arc::new(DiffuseLight::from_color(Color::new(8.0, 8.0, 8.0)));
    let mut slab_idx = 0;
    for i in 0..box_count {
        if i == box_count-3 {
            continue;
        }
        let theta = 2.0 * PI * (i as f64) / (box_count as f64);
        let x = 0.0 + radius * theta.cos(); // 圆心x=0
        let z = 0.0 + radius * theta.sin(); // 圆心z=0
        let angle = -theta * 180.0 / PI + 90.0;
        let box_obj = make_box(
            Point3::new(-box_width/2.0, base_y, -box_depth/2.0),
            Point3::new(box_width/2.0, top_y, box_depth/2.0),
            box_color.clone(),
        );
        let box_obj = Arc::new(RotateY::new(box_obj, angle));
        let box_obj = Arc::new(Translate::new(box_obj, Vec3::new(x, 0.0, z)));
        world.add(box_obj.clone());
        if slab_idx < number_textures.len() {
            let num_mat = number_textures[slab_idx].clone();
            let num_height = box_height / 3.0;
            let num_width = box_width * 0.8;
            let y_center = base_y + box_height * 0.75;
            let quad_origin = Point3::new(-num_width/2.0, y_center - num_height/2.0, -box_depth/2.0 - 0.1);
            let u_vec = Vec3::new(num_width, 0.0, 0.0);
            let v_vec = Vec3::new(0.0, num_height, 0.0);
            let rot = RotateY::new(Arc::new(Quad::new(quad_origin, u_vec, v_vec, num_mat)), angle);
            let quad = Translate::new(Arc::new(rot), Vec3::new(x, 0.0, z));
            world.add(Arc::new(quad));
            slab_idx += 1;
        }
        let strip_width = box_width * 0.8;
        let strip_height = 6.0;
        let strip_y = base_y + 2.0;
        let strip_origin = Point3::new(-strip_width/2.0, strip_y, box_depth/2.0 + 0.2);
        let strip_u = Vec3::new(strip_width, 0.0, 0.0);
        let strip_v = Vec3::new(0.0, strip_height, 0.0);
        let strip_quad = Quad::new(strip_origin, strip_u, strip_v, strip_light_mat.clone());
        let strip_rot = RotateY::new(Arc::new(strip_quad), angle);
        let strip = Translate::new(Arc::new(strip_rot), Vec3::new(x, 0.0, z));
        world.add(Arc::new(strip));
    }

    // 画面比例和高度
    let aspect = 16.0 / 9.0;
    let img_h = image_width as f64 / aspect;

    // 太阳球（光源）参数
    let r_sun = img_h * 0.08; // 太阳球更大，软化分界线
    let r_jupiter = img_h * 0.36; // 木星略大，接受更多光照
    let sun_y = img_h * 0.73;
    let jupiter_y = sun_y - r_sun - r_jupiter + img_h * 0.005;
    let sun_center = Point3::new(0.0, sun_y, 0.0);

    // 进一步降低太阳亮度
    let sun_light_color = Color::new(6.0, 6.0, 4.2); // 增加太阳亮度
    let sun_light = Arc::new(DiffuseLight::from_color(sun_light_color));
    world.add(Arc::new(Sphere::new(sun_center, r_sun, Some(sun_light.clone()))));

    // 木星球参数
    let jupiter_center = Point3::new(0.0, jupiter_y, 0.0);
    let jupiter_tex = Arc::new(ImageTexture::new("input/jupitermap.jpg"));
    // 提升albedo：让木星表面更亮（如果Lambertian::from_texture支持albedo参数，可用，否则用from_color近似）
    // 这里假设from_texture只接受贴图，无法直接调节亮度，则用from_color近似提升亮度
    // let jupiter_mat = Arc::new(Lambertian::from_texture(jupiter_tex));
    // world.add(Arc::new(Sphere::new(jupiter_center, r_jupiter, Some(jupiter_mat))));
    // 方案1：贴图亮度提升（如Lambertian::from_texture_with_albedo）
    // 方案2：直接用from_color提升亮度（近似）
    let jupiter_albedo_boost = 1.2; // 提升亮度，增强纹理可见性
    let jupiter_mat = Arc::new(Lambertian::from_texture_with_albedo(jupiter_tex, jupiter_albedo_boost));
    world.add(Arc::new(Sphere::new(jupiter_center, r_jupiter, Some(jupiter_mat))));

    // 木星大气辉光层：包裹一层稍大的球体，使用ConstantcMedium和淡色
    // 已去除木星大气辉光层

    // 背景贴图为star.jpg


    // 相机：对准视觉中心
    let mut cam = Camera::new();
    cam.aspect_ratio = aspect;
    cam.image_width = image_width;
    cam.samples_per_pixel = samples_per_pixel;
    cam.max_depth = max_depth;
    cam.set_background_texture(None); // 设置为黑色背景
    cam.background = Color::new(0.0, 0.0, 0.0); // 明确指定背景色为黑色
    cam.vfov = 60.0;
    let x_focus = 0.0;
    // 视觉重心略偏向木星
    let y_focus = (jupiter_y * 1.2 + sun_y * 0.8) / 2.0;
    cam.lookfrom = Point3::new(x_focus, y_focus, -img_h * 1.1);
    cam.lookat = Point3::new(x_focus, y_focus, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);
    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;
    cam.set_russian_roulette_strategy(RussianRouletteStrategy::None);

    // 构建光源列表，将太阳球加入lights，便于直接采样
    let mut lights = HittableList::new();
    lights.add(Arc::new(Sphere::new(sun_center, r_sun, None)));
    let (duration, _) = crate::rtweekend::time_it(|| cam.render(&world, &lights, &mut std::io::stdout()));
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "FinalScene2 Render耗时: {:?}", duration).unwrap();
}

fn main() {
    // SIMD优化检测和报告
    #[cfg(target_arch = "x86_64")]
    {
        println!("CPU特性检测:");
        if is_x86_feature_detected!("avx2") {
            println!("  AVX2 OK");
        }
        
        if is_x86_feature_detected!("fma") {
            println!("   FMA OK");
        }
        
        if is_x86_feature_detected!("avx512f") {
            println!("   AVX-512 OK");
        }
        println!();
    }
    
    // 俄罗斯轮盘赌策略选择指南：
    // 1. bouncing_spheres - 复杂多材质场景，使用 HighQuality 策略
    // 2. checkered_spheres - 简单场景，可以使用 Conservative 策略  
    // 3. earth_sphere - 纹理场景，使用 Conservative 策略
    // 4. perlin_spheres - 噪声纹理，使用 Adaptive 策略
    // 5. quads - 几何测试，使用 Conservative 策略
    // 6. simple_light - 光照场景，使用 Conservative 策略
    // 7. cornell_box - 经典场景，需要高质量，使用 Conservative 策略
    // 8. cornell_smoke - 体积渲染，使用 Adaptive 策略
    // 9. final_scene - 最复杂场景，使用 Adaptive 策略
    
    // 激进用户SIMD优化建议：
    // - 编译: RUSTFLAGS="-C target-cpu=native" cargo build --release  
    // - 策略: 使用Aggressive与SIMD配合获得最大性能
    
    match 12 {
        1 => bouncing_spheres(),
        2 => checkered_spheres(),
        3 => earth_sphere(),
        4 => perlin_spheres(),
        5 => quads(),
        6 => simple_light(),
        7 => cornell_box(),
        8 => cornell_smoke(),
        9 => former_final_scene(800, 10000, 40),
        10 => former_final_scene(400, 500, 20),
        11 => final_scene_1(800, 200, 20),
        12 => final_scene_2(1600, 4000, 40), // 1600x900, 400采样，20递归
        _ => former_final_scene(400, 250, 4),
    }
}