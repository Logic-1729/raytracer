use std::io::{self, Write};
use std::fmt;
use crate::vec3::Vec3;
use crate::interval::Interval;

pub type Color = Vec3;

pub fn write_color<W: std::io::Write>(out: &mut W, pixel_color: &Color) -> std::io::Result<()> {
    let r = pixel_color.x;
    let g = pixel_color.y;
    let b = pixel_color.z;

    // 在函数体内创建，不用 static
    let intensity = Interval::new(0.000, 0.999);
    let rbyte = (256.0 * intensity.clamp(r)).floor() as i32;
    let gbyte = (256.0 * intensity.clamp(g)).floor() as i32;
    let bbyte = (256.0 * intensity.clamp(b)).floor() as i32;

    writeln!(out, "{} {} {}", rbyte, gbyte, bbyte)
}

pub fn color_to_string(pixel_color: Color) -> String {
    let r = (255.999 * pixel_color.x) as u8;
    let g = (255.999 * pixel_color.y) as u8;
    let b = (255.999 * pixel_color.z) as u8;
    format!("{} {} {}", r, g, b)
}