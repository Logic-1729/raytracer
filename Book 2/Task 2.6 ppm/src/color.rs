use std::io::{self, Write};
use std::fmt;
use crate::vec3::Vec3;
use crate::interval::Interval;

pub type Color = Vec3;

pub fn write_color<W: std::io::Write>(out: &mut W, pixel_color: &Color) -> std::io::Result<()> {
    // Divide the color by the number of samples and apply gamma correction.
    let scale = 1.0 as f64;
    let mut r = pixel_color.x * scale;
    let mut g = pixel_color.y * scale;
    let mut b = pixel_color.z * scale;

    // Apply gamma correction for gamma=2.0.
    r = linear_to_gamma(r);
    g = linear_to_gamma(g);
    b = linear_to_gamma(b);

    // Clamp to [0.0, 0.999] and convert to [0, 255] for output.
    let intensity = Interval::new(0.0, 0.999);
    let rbyte = (256.0 * intensity.clamp(r)) as i32;
    let gbyte = (256.0 * intensity.clamp(g)) as i32;
    let bbyte = (256.0 * intensity.clamp(b)) as i32;

    writeln!(out, "{} {} {}", rbyte, gbyte, bbyte)
}

fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 { linear_component.sqrt() } else { 0.0 }
}

pub fn color_to_string(pixel_color: Color) -> String {
    let r = (255.999 * pixel_color.x) as u8;
    let g = (255.999 * pixel_color.y) as u8;
    let b = (255.999 * pixel_color.z) as u8;
    format!("{} {} {}", r, g, b)
}