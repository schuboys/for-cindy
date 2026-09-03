//! The drift itself — a port of the `cindyfaves` Canvas2D scene to CPU rasters.

use crate::raster::{pack, Rng, Target, Tex};
use crate::sprites;

const SWIRL_SPEED: f32 = 0.04;
const SKY_DOWNSCALE: usize = 4;
/// The sky only moves as fast as the swirl, so it does not need a fresh bake
/// every frame.
const SKY_REBAKE: f32 = 0.2;

const SKY_STOPS: [(f32, (f32, f32, f32)); 4] = [
    (0.00, (0xff as f32, 0xb3 as f32, 0xd9 as f32)),
    (0.40, (0xe8 as f32, 0xb3 as f32, 0xf0 as f32)),
    (0.75, (0xb3 as f32, 0xdf as f32, 0xff as f32)),
    (1.00, (0x8c as f32, 0xc8 as f32, 0xfc as f32)),
];

const STAR_COLORS: [(f32, f32, f32); 4] = [
    (255.0, 214.0, 232.0),
    (255.0, 247.0, 173.0),
    (179.0, 240.0, 255.0),
    (232.0, 179.0, 240.0),
];

struct Item {
    kind: usize,
    x: f32,
    y: f32,
    size: f32,
    vx: f32,
    vy: f32,
    rot: f32,
    v_rot: f32,
    wobble: f32,
    phase: f32,
    pulse: f32,
    pulse_freq: f32,
    pulse_amp: f32,
}

struct Star {
    x: f32,
    y: f32,
    r: f32,
    speed: f32,
    spin: f32,
    phase: f32,
    color: (f32, f32, f32),
}

struct Sparkle {
    x: f32,
    y: f32,
    r: f32,
    speed: f32,
    phase: f32,
}

pub struct Scene {
    w: usize,
    h: usize,
    s: f32,
    rng: Rng,
    drifters: Vec<Tex>,
    cottage_tex: Tex,
    items: Vec<Item>,
    stars: Vec<Star>,
    sparkles: Vec<Sparkle>,
    sky: Vec<u32>,
    sky_w: usize,
    sky_h: usize,
    sky_baked_at: f32,
    cottage: Option<Item>,
    next_cottage: f32,
}

impl Scene {
    pub fn new(w: usize, h: usize, seed: u32) -> Self {
        let s = h as f32 / 1080.0;
        let rng = Rng::new(seed);

        // One scale-up front: sprites are rasterised at the largest size they
        // will ever be drawn, then sampled down per frame.
        let canon = (220.0 * s).round().max(96.0) as u32;
        let cottage_tex = sprites::build(
            sprites::COTTAGE,
            (canon as f32 * 0.45).round().max(48.0) as u32,
            sprites::COTTAGE_SHADOW,
        );

        let sky_w = (w / SKY_DOWNSCALE).max(1);
        let sky_h = (h / SKY_DOWNSCALE).max(1);

        let mut scene = Scene {
            w,
            h,
            s,
            rng,
            drifters: Vec::new(),
            cottage_tex,
            items: Vec::new(),
            stars: Vec::new(),
            sparkles: Vec::new(),
            sky: vec![0; sky_w * sky_h],
            sky_w,
            sky_h,
            sky_baked_at: f32::MIN,
            cottage: None,
            next_cottage: 0.0,
        };
        let picks = scene.seed();
        // Only the variants that actually drift get rasterised.
        scene.drifters = picks
            .iter()
            .map(|&(cat, variant)| {
                let cat = &sprites::CATEGORIES[cat];
                sprites::build(cat.variants[variant], canon, cat.shadow)
            })
            .collect();
        scene.next_cottage = scene.rng.range(20.0, 60.0);
        scene
    }

    /// Lays out the roster and reports the `(category, variant)` pair behind
    /// each texture slot, in `drifters` order.
    fn seed(&mut self) -> Vec<(usize, usize)> {
        let (w, h, s) = (self.w as f32, self.h as f32, self.s);

        let mut picks: Vec<(usize, usize)> = Vec::new();
        let items = sprites::spawn_plan()
            .into_iter()
            .map(|cat| {
                let variant = self.rng.below(sprites::CATEGORIES[cat].variants.len());
                let kind = match picks.iter().position(|&p| p == (cat, variant)) {
                    Some(i) => i,
                    None => {
                        picks.push((cat, variant));
                        picks.len() - 1
                    }
                };
                Item {
                    kind,
                    x: self.rng.range(0.0, w),
                    y: self.rng.range(0.0, h * 0.85) + h * 0.08,
                    size: self.rng.range(110.0, 200.0) * s * sprites::CATEGORIES[cat].scale,
                    vx: self.rng.range(6.0, 40.0) * self.rng.sign(),
                    vy: self.rng.range(4.0, 24.0) * self.rng.sign(),
                    rot: self.rng.range(-0.3, 0.3),
                    v_rot: self.rng.range(-0.35, 0.35),
                    wobble: self.rng.range(0.5, 2.7),
                    phase: self.rng.range(0.0, std::f32::consts::TAU),
                    pulse: 1.0,
                    pulse_freq: self.rng.range(1.6, 4.4),
                    pulse_amp: self.rng.range(0.08, 0.16),
                }
            })
            .collect();

        let stars = (0..14)
            .map(|_| Star {
                x: self.rng.range(0.0, w),
                y: self.rng.range(0.0, h),
                r: self.rng.range(6.0, 16.0) * s,
                speed: self.rng.range(0.7, 1.9),
                spin: self.rng.range(-0.4, 0.4),
                phase: self.rng.range(0.0, std::f32::consts::TAU),
                color: STAR_COLORS[self.rng.below(STAR_COLORS.len())],
            })
            .collect();

        let sparkles = (0..22)
            .map(|_| Sparkle {
                x: self.rng.range(0.0, w),
                y: self.rng.range(0.0, h),
                r: self.rng.range(2.0, 7.0) * s,
                speed: self.rng.range(1.5, 4.0),
                phase: self.rng.range(0.0, std::f32::consts::TAU),
            })
            .collect();

        self.items = items;
        self.stars = stars;
        self.sparkles = sparkles;
        picks
    }

    fn bake_sky(&mut self, t: f32) {
        let (w, h) = (self.w as f32, self.h as f32);
        let cx = w * (0.45 + 0.1 * (t * SWIRL_SPEED).sin());
        let cy = h * (0.35 + 0.08 * (t * SWIRL_SPEED * 0.7).cos());
        let radius = w.max(h) * 1.1;
        let half_diag = 0.5 * (w * w + h * h).sqrt();
        let step = SKY_DOWNSCALE as f32;

        for sy in 0..self.sky_h {
            let py = (sy as f32 + 0.5) * step;
            for sx in 0..self.sky_w {
                let px = (sx as f32 + 0.5) * step;
                let d = (((px - cx).powi(2) + (py - cy).powi(2)).sqrt() / radius).clamp(0.0, 1.0);
                let (mut r, mut g, mut b) = SKY_STOPS[SKY_STOPS.len() - 1].1;
                for pair in SKY_STOPS.windows(2) {
                    let (p0, c0) = pair[0];
                    let (p1, c1) = pair[1];
                    if d <= p1 {
                        let f = ((d - p0) / (p1 - p0)).clamp(0.0, 1.0);
                        r = c0.0 + (c1.0 - c0.0) * f;
                        g = c0.1 + (c1.1 - c0.1) * f;
                        b = c0.2 + (c1.2 - c0.2) * f;
                        break;
                    }
                }
                // Gentle vignette, baked in so it costs nothing per frame.
                let vd = (((px - w * 0.5).powi(2) + (py - h * 0.5).powi(2)).sqrt() / half_diag)
                    .clamp(0.0, 1.0);
                let vig = 1.0 - 0.22 * smoothstep(0.55, 1.0, vd);
                self.sky[sy * self.sky_w + sx] =
                    pack((r * vig) as u8, (g * vig) as u8, (b * vig) as u8);
            }
        }
        self.sky_baked_at = t;
    }

    pub fn update(&mut self, t: f32, dt: f32) {
        let (w, h, s) = (self.w as f32, self.h as f32, self.s);

        for item in &mut self.items {
            item.x += item.vx * dt * s;
            item.y += item.vy * dt * s + (t * item.wobble + item.phase).sin() * 0.5 * s;
            item.rot += item.v_rot * dt;
            item.pulse = 1.0 + item.pulse_amp * (t * item.pulse_freq + item.phase).sin();

            let margin = item.size * 0.6;
            if item.x > w + margin {
                item.x = -margin;
            }
            if item.x < -margin {
                item.x = w + margin;
            }
            if item.y > h - margin || item.y < margin {
                item.vy = -item.vy;
            }
        }

        // Cottage cheese: a rare small drift-through, every couple of minutes.
        let mut gone = false;
        if let Some(c) = &mut self.cottage {
            c.x += c.vx * dt * s;
            c.y += c.vy * dt * s + (t * c.wobble + c.phase).sin() * 0.4 * s;
            c.rot += c.v_rot * dt;
            c.pulse = 1.0 + c.pulse_amp * (t * c.pulse_freq + c.phase).sin();
            let margin = c.size * 1.2;
            gone = c.x > w + margin || c.x < -margin;
        }
        if gone {
            self.cottage = None;
            self.next_cottage = t + self.rng.range(120.0, 240.0);
        }
        if self.cottage.is_none() && t >= self.next_cottage {
            let leftward = self.rng.f32() < 0.5;
            let size = self.rng.range(64.0, 92.0) * s;
            self.cottage = Some(Item {
                kind: 0,
                x: if leftward { w + size } else { -size },
                y: self.rng.range(h * 0.15, h * 0.85),
                size,
                vx: self.rng.range(26.0, 44.0) * if leftward { -1.0 } else { 1.0 },
                vy: self.rng.range(-6.0, 6.0),
                rot: self.rng.range(-0.2, 0.2),
                v_rot: self.rng.range(-0.35, 0.35),
                wobble: self.rng.range(0.5, 2.2),
                phase: self.rng.range(0.0, std::f32::consts::TAU),
                pulse: 1.0,
                pulse_freq: self.rng.range(1.6, 4.4),
                pulse_amp: self.rng.range(0.08, 0.16),
            });
        }
    }

    pub fn render(&mut self, buf: &mut [u32], t: f32) {
        if t - self.sky_baked_at >= SKY_REBAKE {
            self.bake_sky(t);
        }
        // Nearest upscale of the low-res sky — the gradient is smooth enough
        // that the 4px steps are invisible.
        for y in 0..self.h {
            let sy = (y / SKY_DOWNSCALE).min(self.sky_h - 1);
            let src = &self.sky[sy * self.sky_w..sy * self.sky_w + self.sky_w];
            let dst = &mut buf[y * self.w..y * self.w + self.w];
            for x in 0..self.w {
                dst[x] = src[(x / SKY_DOWNSCALE).min(self.sky_w - 1)];
            }
        }

        let mut tgt = Target {
            buf,
            w: self.w,
            h: self.h,
        };
        let (w, h, s) = (self.w as f32, self.h as f32, self.s);

        // Candy stripes drifting behind everything.
        for i in 0..6 {
            let span = w + 360.0 * s;
            let x = ((t * 18.0 + i as f32 * 240.0) * s).rem_euclid(span) - 180.0 * s;
            let y = h * (0.1 + i as f32 * 0.16);
            let rx = 60.0 * s;
            let ry = 22.0 * s;
            stripe(&mut tgt, x + rx, y, rx, ry);
        }

        // Twinkling five-point stars.
        for star in &self.stars {
            let a = 0.35 + 0.35 * (t * star.speed + star.phase).sin();
            let r = star.r * (0.8 + 0.2 * (t * star.speed * 2.0 + star.phase).sin());
            let rot = t * star.spin + star.phase;
            let pts = star_points(star.x, star.y, r, r * 0.45, rot);
            tgt.fill_poly(&pts, star.color, a.max(0.0));
        }

        // The favorites, each with its soft colored shadow.
        for item in &self.items {
            let tex = &self.drifters[item.kind];
            draw_item(&mut tgt, tex, item, s);
        }
        if let Some(c) = &self.cottage {
            draw_item(&mut tgt, &self.cottage_tex, c, s);
        }

        // Soft twinkles on top.
        for sp in &self.sparkles {
            let a = 0.4 + 0.5 * (t * sp.speed + sp.phase).sin();
            let r = sp.r * (0.7 + 0.3 * (t * sp.speed * 1.7 + sp.phase).sin());
            tgt.soft_circle(sp.x, sp.y, r.max(0.5), (255.0, 255.0, 255.0), a.max(0.0));
        }
    }
}

fn draw_item(tgt: &mut Target, tex: &Tex, item: &Item, s: f32) {
    let scale = item.size * item.pulse / tex.content_w;
    tgt.blit(
        tex,
        item.x + 8.0 * s,
        item.y + 12.0 * s,
        scale,
        item.rot,
        true,
    );
    tgt.blit(tex, item.x, item.y, scale, item.rot, false);
}

/// A white candy stripe: an ellipse whose alpha fades out towards both ends.
fn stripe(tgt: &mut Target, cx: f32, cy: f32, rx: f32, ry: f32) {
    let x0 = ((cx - rx).floor() as isize).max(0) as usize;
    let x1 = ((cx + rx).ceil() as isize).min(tgt.w as isize).max(0) as usize;
    let y0 = ((cy - ry).floor() as isize).max(0) as usize;
    let y1 = ((cy + ry).ceil() as isize).min(tgt.h as isize).max(0) as usize;
    for y in y0..y1 {
        let dy = (y as f32 + 0.5 - cy) / ry;
        let row = y * tgt.w;
        for x in x0..x1 {
            let dx = (x as f32 + 0.5 - cx) / rx;
            let d = dx * dx + dy * dy;
            if d >= 1.0 {
                continue;
            }
            let edge = (1.0 - d).min(1.0);
            let along = 1.0 - dx.abs();
            let a = 0.12 * 0.7 * along * edge;
            crate::raster::blend_px(&mut tgt.buf[row + x], 255.0, 255.0, 255.0, a);
        }
    }
}

fn star_points(cx: f32, cy: f32, outer: f32, inner: f32, rot: f32) -> Vec<(f32, f32)> {
    let mut pts = Vec::with_capacity(10);
    let step = std::f32::consts::PI / 5.0;
    let mut a = -std::f32::consts::FRAC_PI_2 + rot;
    for i in 0..10 {
        let r = if i % 2 == 0 { outer } else { inner };
        pts.push((cx + a.cos() * r, cy + a.sin() * r));
        a += step;
    }
    pts
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::Scene;

    #[test]
    fn renders_a_frame_with_sprites_and_sky() {
        let (w, h) = (960usize, 540usize);
        let mut scene = Scene::new(w, h, 42);
        let mut buf = vec![0u32; w * h];
        for i in 0..180 {
            let t = i as f32 / 60.0;
            scene.update(t, 1.0 / 60.0);
            scene.render(&mut buf, t);
        }
        // The sky alone would already be many colours; assert the frame is
        // neither blank nor a single flat fill.
        assert!(buf.iter().any(|&p| p != buf[0]));
        assert!(buf.iter().all(|&p| p & 0xff00_0000 == 0));
        if std::env::var("CINDY_DUMP").is_ok() {
            let mut rgb = Vec::with_capacity(w * h * 3);
            for &p in &buf {
                rgb.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, p as u8]);
            }
            image::save_buffer(
                std::env::var("CINDY_DUMP").unwrap(),
                &rgb,
                w as u32,
                h as u32,
                image::ColorType::Rgb8,
            )
            .unwrap();
        }
    }

    #[test]
    fn items_pulse_and_drift_at_their_own_speeds() {
        let scene = Scene::new(960, 540, 42);
        let freqs: Vec<f32> = scene.items.iter().map(|i| i.pulse_freq).collect();
        let amps: Vec<f32> = scene.items.iter().map(|i| i.pulse_amp).collect();
        let speeds: Vec<f32> = scene.items.iter().map(|i| i.vx.abs()).collect();

        assert!(freqs.iter().all(|f| (1.6..=4.4).contains(f)));
        assert!(amps.iter().all(|a| (0.08..=0.16).contains(a)));
        assert!(spread(&freqs) > 1.0, "pulse frequencies too uniform");
        assert!(spread(&amps) > 0.03, "pulse amplitudes too uniform");
        assert!(spread(&speeds) > 15.0, "drift speeds too uniform");
    }

    fn spread(v: &[f32]) -> f32 {
        let lo = v.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = v.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        hi - lo
    }
}
