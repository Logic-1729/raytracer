use std::fs;
use std::path::Path;
use std::env;

pub struct RtwImage {
    pub image_width: usize,
    pub image_height: usize,
    pub bytes_per_pixel: usize,
    pub bytes_per_scanline: usize,
    pub bdata: Option<Vec<u8>>, // 8-bit per channel
    pub fdata: Option<Vec<f32>>, // float per channel
}

impl RtwImage {
    pub fn new() -> Self {
        Self {
            image_width: 0,
            image_height: 0,
            bytes_per_pixel: 3,
            bytes_per_scanline: 0,
            bdata: None,
            fdata: None,
        }
    }

    pub fn from_file(image_filename: &str) -> Self {
        let mut img = Self::new();
        let filename = image_filename.to_string();
        let imagedir = env::var("RTW_IMAGES").ok();
        let mut tried = vec![];
        let mut try_load = |path: &str| {
            tried.push(path.to_string());
            img.load(path)
        };
        if let Some(ref dir) = imagedir {
            if try_load(&format!("{}/{}", dir, image_filename)) { return img; }
        }
        if try_load(&filename) { return img; }
        if try_load(&format!("images/{}", filename)) { return img; }
        if try_load(&format!("../images/{}", filename)) { return img; }
        if try_load(&format!("../../images/{}", filename)) { return img; }
        if try_load(&format!("../../../images/{}", filename)) { return img; }
        if try_load(&format!("../../../../images/{}", filename)) { return img; }
        if try_load(&format!("../../../../../images/{}", filename)) { return img; }
        if try_load(&format!("../../../../../../images/{}", filename)) { return img; }
        eprintln!("ERROR: Could not load image file '{}'. Tried: {:?}", image_filename, tried);
        img
    }

    pub fn load(&mut self, filename: &str) -> bool {
        match image::open(filename) {
            Ok(img) => {
                let img = img.to_rgb8();
                self.image_width = img.width() as usize;
                self.image_height = img.height() as usize;
                self.bytes_per_pixel = 3;
                self.bytes_per_scanline = self.image_width * self.bytes_per_pixel;
                self.bdata = Some(img.as_raw().clone());
                // fdata: [0.0, 1.0] float
                self.fdata = Some(img.as_raw().iter().map(|&b| b as f32 / 255.0).collect());
                true
            },
            Err(_) => false,
        }
    }

    pub fn width(&self) -> usize {
        if self.bdata.is_none() { 0 } else { self.image_width }
    }
    pub fn height(&self) -> usize {
        if self.bdata.is_none() { 0 } else { self.image_height }
    }

    pub fn pixel_data(&self, mut x: isize, mut y: isize) -> &[u8] {
        static MAGENTA: [u8; 3] = [255, 0, 255];
        if self.bdata.is_none() { return &MAGENTA; }
        let w = self.image_width as isize;
        let h = self.image_height as isize;
        x = clamp(x, 0, w);
        y = clamp(y, 0, h);
        let idx = (y * w + x) as usize * self.bytes_per_pixel;
        &self.bdata.as_ref().unwrap()[idx..idx + 3]
    }
}

fn clamp(x: isize, low: isize, high: isize) -> isize {
    if x < low { low }
    else if x < high { x }
    else { high - 1 }
}
