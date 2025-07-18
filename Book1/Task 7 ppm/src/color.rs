use std::io::{self, Write};
use crate::vec3::Color;

pub fn write_color<W: Write>(out: &mut W, pixel_color: Color) -> io::Result<()> {
    let r = (255.999 * pixel_color.x) as u8;
    let g = (255.999 * pixel_color.y) as u8;
    let b = (255.999 * pixel_color.z) as u8;
    writeln!(out, "{} {} {}", r, g, b)
}

pub fn color_to_string(pixel_color: Color) -> String {
    let r = (255.999 * pixel_color.x) as u8;
    let g = (255.999 * pixel_color.y) as u8;
    let b = (255.999 * pixel_color.z) as u8;
    format!("{} {} {}", r, g, b)
}