// 简单的编译测试文件
use crate::camera::{Camera, RussianRouletteStrategy};

fn main() {
    let mut camera = Camera::new();
    camera.set_russian_roulette_strategy(RussianRouletteStrategy::Conservative);
    println!("Camera created successfully with Russian Roulette strategy: {:?}", camera.get_russian_roulette_strategy());
}
