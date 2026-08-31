use png::{ColorType, Encoder, BitDepth};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

fn create_sprite_sheet(width: u32, height: u32, color: (u8, u8, u8, u8), path: &str) {
    let mut image_data = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            // Draw border & inner colored square
            if x < 2 || x >= width - 2 || y < 2 || y >= height - 2 {
                image_data[idx] = 0;
                image_data[idx + 1] = 0;
                image_data[idx + 2] = 0;
                image_data[idx + 3] = 255;
            } else {
                image_data[idx] = color.0;
                image_data[idx + 1] = color.1;
                image_data[idx + 2] = color.2;
                image_data[idx + 3] = color.3;
            }
        }
    }

    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);
    let mut encoder = Encoder::new(w, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&image_data).unwrap();
}

fn main() {
    fs::create_dir_all("crates/companion-render/src/assets").unwrap();
    // Idle: 4 frames = 256x64
    create_sprite_sheet(256, 64, (100, 200, 100, 255), "crates/companion-render/src/assets/idle.png");
    // Walking: 6 frames = 384x64
    create_sprite_sheet(384, 64, (100, 100, 200, 255), "crates/companion-render/src/assets/walking.png");
    // Working: 4 frames = 256x64
    create_sprite_sheet(256, 64, (200, 200, 100, 255), "crates/companion-render/src/assets/working.png");
    // Sleeping: 4 frames = 256x64
    create_sprite_sheet(256, 64, (150, 150, 150, 255), "crates/companion-render/src/assets/sleeping.png");
    // Distressed: 4 frames = 256x64
    create_sprite_sheet(256, 64, (200, 100, 100, 255), "crates/companion-render/src/assets/distressed.png");
    // Celebrating: 6 frames = 384x64
    create_sprite_sheet(384, 64, (200, 100, 200, 255), "crates/companion-render/src/assets/celebrating.png");
    println!("Assets generated successfully!");
}
