//! The embedded artwork. Every PNG ships inside the .scr — the screensaver
//! never touches the disk or the network once installed.
//!
//! The roster mirrors the manifest at the top of `web/index.html`: the same
//! seven categories, the same variant files, the same spawn weights.

use image::imageops::FilterType;
use image::GenericImageView;

use crate::raster::{blur_alpha, Tex};

macro_rules! asset {
    ($file:literal) => {
        include_bytes!(concat!("../../web/assets/", $file))
    };
}

/// One of the seven favorites, with every variant the web page can draw for it.
pub struct Category {
    /// The manifest key in `web/index.html`; carried so the two stay legible
    /// side by side.
    #[allow(dead_code)]
    pub name: &'static str,
    pub variants: &'static [&'static [u8]],
    /// The drop-shadow tint the original scene used for this category.
    pub shadow: (f32, f32, f32),
    /// How many of the drifters on screen belong to this category.
    pub weight: usize,
    /// Size multiplier — candy pieces read better a bit smaller.
    pub scale: f32,
}

pub const CATEGORIES: [Category; 7] = [
    Category {
        name: "coke",
        variants: &[asset!("coke.png")],
        shadow: (255.0, 30.0, 80.0),
        weight: 2,
        scale: 1.0,
    },
    Category {
        name: "cotton",
        variants: &[asset!("cotton.png")],
        shadow: (255.0, 120.0, 220.0),
        weight: 2,
        scale: 1.0,
    },
    Category {
        name: "disney",
        variants: &[
            asset!("disney.png"),
            asset!("minnie.png"),
            asset!("minnie2.png"),
            asset!("minnie3.png"),
            asset!("minnie4.png"),
            asset!("mickey.png"),
            asset!("castle2.png"),
            asset!("ears1.png"),
            asset!("ears2.png"),
            asset!("bow.png"),
        ],
        shadow: (120.0, 80.0, 220.0),
        weight: 5,
        scale: 1.0,
    },
    Category {
        name: "candy",
        variants: &[
            asset!("candy1.png"),
            asset!("candy2.png"),
            asset!("candy3.png"),
            asset!("candy4.png"),
            asset!("candy5.png"),
            asset!("candy6.png"),
        ],
        shadow: (255.0, 170.0, 0.0),
        weight: 3,
        scale: 0.65,
    },
    Category {
        name: "golden",
        variants: &[
            asset!("golden.png"),
            asset!("golden2.png"),
            asset!("golden3.png"),
            asset!("golden4.png"),
            asset!("golden5.png"),
        ],
        shadow: (255.0, 180.0, 90.0),
        weight: 3,
        scale: 1.0,
    },
    Category {
        name: "cookies",
        variants: &[
            asset!("cookie1.png"),
            asset!("cookie2.png"),
            asset!("cookie3.png"),
            asset!("cookie4.png"),
        ],
        shadow: (200.0, 130.0, 60.0),
        weight: 3,
        scale: 1.0,
    },
    Category {
        name: "tiktok",
        variants: &[asset!("tiktok.png")],
        shadow: (5.0, 217.0, 232.0),
        weight: 1,
        scale: 1.0,
    },
];

/// The rare easter-egg drifter, outside the weighted roster.
pub const COTTAGE: &[u8] = asset!("cottage.png");
pub const COTTAGE_SHADOW: (f32, f32, f32) = (120.0, 190.0, 255.0);

/// The per-item category roster, expanded from the weights above.
pub fn spawn_plan() -> Vec<usize> {
    let mut plan = Vec::new();
    for (i, cat) in CATEGORIES.iter().enumerate() {
        for _ in 0..cat.weight {
            plan.push(i);
        }
    }
    plan
}

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

#[cfg(test)]
mod tests {
    use super::{spawn_plan, CATEGORIES, COTTAGE};

    #[test]
    fn every_embedded_variant_decodes_with_alpha() {
        let all = CATEGORIES
            .iter()
            .flat_map(|c| c.variants.iter().copied())
            .chain(std::iter::once(COTTAGE));
        let mut n = 0;
        for bytes in all {
            let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
                .expect("embedded sprite failed to decode")
                .to_rgba8();
            assert!(
                img.pixels().any(|p| p.0[3] < 255),
                "sprite has no transparency"
            );
            n += 1;
        }
        assert_eq!(n, 29, "expected the full embedded roster");
    }

    #[test]
    fn spawn_plan_follows_the_category_weights() {
        let plan = spawn_plan();
        assert_eq!(plan.len(), 19);
        for (i, cat) in CATEGORIES.iter().enumerate() {
            assert_eq!(
                plan.iter().filter(|&&c| c == i).count(),
                cat.weight,
                "{} spawned off-weight",
                cat.name
            );
        }
    }
}
