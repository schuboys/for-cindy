//! The embedded artwork. Every PNG ships inside the .scr — the screensaver
//! never touches the disk or the network once installed.

use image::imageops::FilterType;
use image::GenericImageView;

use crate::raster::{blur_alpha, Tex};

pub const CANDY: &[u8] = include_bytes!("../../assets/original/candy.png");
pub const COKE: &[u8] = include_bytes!("../../assets/original/coke.png");
pub const COTTON: &[u8] = include_bytes!("../../assets/original/cotton.png");
pub const DISNEY: &[u8] = include_bytes!("../../assets/original/disney.png");
pub const TIKTOK: &[u8] = include_bytes!("../../assets/original/tiktok.png");
pub const MINNIE: &[u8] = include_bytes!("../../assets/new/minnie.png");
pub const BOW: &[u8] = include_bytes!("../../assets/new/bow.png");
pub const GOLDEN: &[u8] = include_bytes!("../../assets/new/golden.png");
pub const COTTAGE: &[u8] = include_bytes!("../../assets/new/cottage.png");

/// The eight regular drifters, with the drop-shadow tint the original scene
/// used for each one.
pub const DRIFTERS: [(&[u8], (f32, f32, f32)); 8] = [
    (CANDY, (255.0, 170.0, 0.0)),
    (COKE, (255.0, 30.0, 80.0)),
    (COTTON, (255.0, 120.0, 220.0)),
    (DISNEY, (120.0, 80.0, 220.0)),
    (TIKTOK, (5.0, 217.0, 232.0)),
    (MINNIE, (255.0, 60.0, 120.0)),
    (BOW, (255.0, 90.0, 150.0)),
    (GOLDEN, (255.0, 190.0, 90.0)),
];

pub const COTTAGE_SHADOW: (f32, f32, f32) = (140.0, 200.0, 255.0);

/// Decode a PNG, scale it once to `target_w` (aspect preserved), premultiply,
/// and bake the blurred alpha that stands in for the drop shadow.
pub fn build(bytes: &[u8], target_w: u32, shadow_color: (f32, f32, f32)) -> Tex {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .expect("embedded sprite failed to decode");
    let (ow, oh) = img.dimensions();
    let target_w = target_w.max(8);
    let target_h = ((oh as f32 / ow as f32) * target_w as f32).round().max(8.0) as u32;
    let scaled = image::imageops::resize(&img.to_rgba8(), target_w, target_h, FilterType::Triangle);

    let blur = ((target_w as f32 / 16.0).round() as usize).max(2);
    let pad = blur * 3;
    let w = target_w as usize + pad * 2;
    let h = target_h as usize + pad * 2;

    let mut px = vec![0u8; w * h * 4];
    let mut shadow = vec![0u8; w * h];
    for y in 0..target_h as usize {
        for x in 0..target_w as usize {
            let s = scaled.get_pixel(x as u32, y as u32).0;
            let a = s[3] as u32;
            let di = ((y + pad) * w + (x + pad)) * 4;
            px[di] = ((s[0] as u32 * a) / 255) as u8;
            px[di + 1] = ((s[1] as u32 * a) / 255) as u8;
            px[di + 2] = ((s[2] as u32 * a) / 255) as u8;
            px[di + 3] = s[3];
            shadow[(y + pad) * w + (x + pad)] = s[3];
        }
    }
    blur_alpha(&mut shadow, w, h, blur);

    Tex {
        w,
        h,
        content_w: target_w as f32,
        px,
        shadow,
        shadow_color,
    }
}
