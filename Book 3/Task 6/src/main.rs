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
use crate::material::{Material,Lambertian,Metal,Dielectric};
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
    let (duration, result) = time_it(|| cam.render(&world, &mut std::io::stdout()).unwrap());
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

    let (duration, result) = time_it(|| cam.render(&world, &mut std::io::stdout()).unwrap());
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

    let (duration, result) = time_it(|| cam.render(&world, &mut std::io::stdout()).unwrap());
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

    let (duration, result) = time_it(|| cam.render(&world, &mut std::io::stdout()).unwrap());
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

    let (duration, _) = crate::rtweekend::time_it(|| cam.render(&world,&mut std::io::stdout()));
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
        difflight,
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

    let (duration, _) = crate::rtweekend::time_it(|| cam.render(&world,&mut std::io::stdout()));
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

    let box1 = make_box(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        white.clone(),
    );
    let box1 = Arc::new(RotateY::new(box1, 15.0));
    let box1 = Arc::new(Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));
    world.add(box1);

    let box2 = make_box(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 165.0, 165.0),
        white.clone(),
    );
    let box2 = Arc::new(RotateY::new(box2, -18.0));
    let box2 = Arc::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));
    world.add(box2);
    let mut cam = Camera::new();
    cam.aspect_ratio = 1.0;
    cam.image_width = 600;
    cam.samples_per_pixel = 1000;
    cam.max_depth = 50;
    cam.background = Color::new(0.0, 0.0, 0.0);

    cam.vfov = 40.0;
    cam.lookfrom = Point3::new(278.0, 278.0, -800.0);
    cam.lookat = Point3::new(278.0, 278.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);
    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;
    
    // Cornell box场景需要高质量的光线追踪，使用保守策略
    cam.set_russian_roulette_strategy(RussianRouletteStrategy::Conservative);

    let (duration, _) = crate::rtweekend::time_it(|| cam.render(&world,&mut std::io::stdout()));
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

    let (duration, _) = crate::rtweekend::time_it(|| cam.render(&world,&mut std::io::stdout()));
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "CornellSmoke Render耗时: {:?}", duration).unwrap();
}

fn final_scene(image_width: usize, samples_per_pixel: usize, max_depth: usize) {
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
    cam.aspect_ratio = 1.0;
    cam.image_width = image_width;
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

    let (duration, _) = crate::rtweekend::time_it(|| cam.render(&world, &mut std::io::stdout()));
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open("output.txt").unwrap();
    writeln!(file, "FinalScene Render耗时: {:?}", duration).unwrap();
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
    
    match 7 {
        1 => bouncing_spheres(),
        2 => checkered_spheres(),
        3 => earth_sphere(),
        4 => perlin_spheres(),
        5 => quads(),
        6 => simple_light(),
        7 => cornell_box(),
        8 => cornell_smoke(),
        9 => final_scene(800, 10000, 40),
        10 => final_scene(400, 500, 20),
        _ => final_scene(400, 250, 4),
    }
}