//! Tiny CPU rasteriser: 0x00RRGGBB u32 target, premultiplied-alpha source blits.

/// xorshift32 — no `rand` dependency, deterministic enough for a screensaver.
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng(if seed == 0 { 0x9e37_79b9 } else { seed })
    }

    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    /// Uniform in [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / 16_777_216.0
    }

    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }

    pub fn sign(&mut self) -> f32 {
        if self.next_u32() & 1 == 0 {
            -1.0
        } else {
            1.0
        }
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u32() as usize) % n.max(1)
    }
}

#[inline]
pub fn pack(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

#[inline]
pub fn unpack(px: u32) -> (u32, u32, u32) {
    ((px >> 16) & 0xff, (px >> 8) & 0xff, px & 0xff)
}

/// Blend a straight-alpha colour over one pixel. `a` is 0..=1.
#[inline]
pub fn blend_px(dst: &mut u32, r: f32, g: f32, b: f32, a: f32) {
    if a <= 0.0 {
        return;
    }
    let a = a.min(1.0);
    let (dr, dg, db) = unpack(*dst);
    let nr = dr as f32 + (r - dr as f32) * a;
    let ng = dg as f32 + (g - dg as f32) * a;
    let nb = db as f32 + (b - db as f32) * a;
    *dst = pack(nr as u8, ng as u8, nb as u8);
}

/// A pre-scaled sprite: premultiplied RGBA plus a pre-blurred alpha mask used
/// as the cheap stand-in for a gaussian drop shadow.
pub struct Tex {
    pub w: usize,
    pub h: usize,
    /// width of the artwork inside the padded texture, in texture pixels
    pub content_w: f32,
    /// premultiplied RGBA, 4 bytes per pixel
    pub px: Vec<u8>,
    /// blurred alpha, 1 byte per pixel, same dimensions
    pub shadow: Vec<u8>,
    pub shadow_color: (f32, f32, f32),
}

impl Tex {
    /// Bilinear sample of the premultiplied colour at texture coords.
    #[inline]
    fn sample(&self, u: f32, v: f32) -> (f32, f32, f32, f32) {
        let x0 = u.floor();
        let y0 = v.floor();
        let fx = u - x0;
        let fy = v - y0;
        let x0 = x0 as isize;
        let y0 = y0 as isize;
        let mut acc = [0.0f32; 4];
        for (dy, wy) in [(0isize, 1.0 - fy), (1, fy)] {
            for (dx, wx) in [(0isize, 1.0 - fx), (1, fx)] {
                let w = wx * wy;
                if w <= 0.0 {
                    continue;
                }
                let x = x0 + dx;
                let y = y0 + dy;
                if x < 0 || y < 0 || x >= self.w as isize || y >= self.h as isize {
                    continue;
                }
                let i = (y as usize * self.w + x as usize) * 4;
                acc[0] += self.px[i] as f32 * w;
                acc[1] += self.px[i + 1] as f32 * w;
                acc[2] += self.px[i + 2] as f32 * w;
                acc[3] += self.px[i + 3] as f32 * w;
            }
        }
        (acc[0], acc[1], acc[2], acc[3])
    }

    #[inline]
    fn sample_shadow(&self, u: f32, v: f32) -> f32 {
        let x = u.round();
        let y = v.round();
        if x < 0.0 || y < 0.0 || x >= self.w as f32 || y >= self.h as f32 {
            return 0.0;
        }
        self.shadow[y as usize * self.w + x as usize] as f32
    }
}

pub struct Target<'a> {
    pub buf: &'a mut [u32],
    pub w: usize,
    pub h: usize,
}

impl<'a> Target<'a> {
    /// Inverse-mapped rotate + scale blit. `cx`,`cy` is the destination centre,
    /// `scale` maps texture pixels to screen pixels.
    pub fn blit(&mut self, tex: &Tex, cx: f32, cy: f32, scale: f32, rot: f32, shadow: bool) {
        if scale <= 0.0 {
            return;
        }
        let (tw, th) = (tex.w as f32, tex.h as f32);
        let radius = 0.5 * (tw * tw + th * th).sqrt() * scale + 2.0;
        let x0 = ((cx - radius).floor() as isize).max(0) as usize;
        let x1 = ((cx + radius).ceil() as isize).min(self.w as isize).max(0) as usize;
        let y0 = ((cy - radius).floor() as isize).max(0) as usize;
        let y1 = ((cy + radius).ceil() as isize).min(self.h as isize).max(0) as usize;
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        let inv = 1.0 / scale;
        let (sn, cs) = rot.sin_cos();
        let (sr, sg, sb) = tex.shadow_color;

        for y in y0..y1 {
            let dy = y as f32 + 0.5 - cy;
            let row = y * self.w;
            for x in x0..x1 {
                let dx = x as f32 + 0.5 - cx;
                // rotate by -rot, then to texture space
                let u = (dx * cs + dy * sn) * inv + tw * 0.5;
                let v = (-dx * sn + dy * cs) * inv + th * 0.5;
                if u < -1.0 || v < -1.0 || u > tw || v > th {
                    continue;
                }
                let dst = &mut self.buf[row + x];
                if shadow {
                    let a = tex.sample_shadow(u, v) / 255.0;
                    blend_px(dst, sr, sg, sb, a * 0.55);
                } else {
                    let (r, g, b, a) = tex.sample(u, v);
                    if a <= 0.5 {
                        continue;
                    }
                    // premultiplied source over destination
                    let af = a / 255.0;
                    let (dr, dg, db) = unpack(*dst);
                    let nr = r + dr as f32 * (1.0 - af);
                    let ng = g + dg as f32 * (1.0 - af);
                    let nb = b + db as f32 * (1.0 - af);
                    *dst = pack(
                        nr.clamp(0.0, 255.0) as u8,
                        ng.clamp(0.0, 255.0) as u8,
                        nb.clamp(0.0, 255.0) as u8,
                    );
                }
            }
        }
    }

    pub fn soft_circle(&mut self, cx: f32, cy: f32, r: f32, rgb: (f32, f32, f32), alpha: f32) {
        if r <= 0.0 || alpha <= 0.0 {
            return;
        }
        let x0 = ((cx - r - 1.0) as isize).max(0) as usize;
        let x1 = ((cx + r + 1.0).ceil() as isize).min(self.w as isize).max(0) as usize;
        let y0 = ((cy - r - 1.0) as isize).max(0) as usize;
        let y1 = ((cy + r + 1.0).ceil() as isize).min(self.h as isize).max(0) as usize;
        for y in y0..y1 {
            let dy = y as f32 + 0.5 - cy;
            let row = y * self.w;
            for x in x0..x1 {
                let dx = x as f32 + 0.5 - cx;
                let d = (dx * dx + dy * dy).sqrt();
                let cov = (r - d + 0.5).clamp(0.0, 1.0);
                if cov > 0.0 {
                    blend_px(&mut self.buf[row + x], rgb.0, rgb.1, rgb.2, alpha * cov);
                }
            }
        }
    }

    /// Fill a convex-ish polygon (used for the 5-point stars) with 2x2 coverage
    /// sampling so the spikes do not crawl.
    pub fn fill_poly(&mut self, pts: &[(f32, f32)], rgb: (f32, f32, f32), alpha: f32) {
        if pts.len() < 3 || alpha <= 0.0 {
            return;
        }
        let mut minx = f32::MAX;
        let mut maxx = f32::MIN;
        let mut miny = f32::MAX;
        let mut maxy = f32::MIN;
        for &(x, y) in pts {
            minx = minx.min(x);
            maxx = maxx.max(x);
            miny = miny.min(y);
            maxy = maxy.max(y);
        }
        let x0 = (minx.floor() as isize).max(0) as usize;
        let x1 = ((maxx.ceil() + 1.0) as isize).min(self.w as isize).max(0) as usize;
        let y0 = (miny.floor() as isize).max(0) as usize;
        let y1 = ((maxy.ceil() + 1.0) as isize).min(self.h as isize).max(0) as usize;
        for y in y0..y1 {
            let row = y * self.w;
            for x in x0..x1 {
                let mut hits = 0;
                for (ox, oy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                    if inside(pts, x as f32 + ox, y as f32 + oy) {
                        hits += 1;
                    }
                }
                if hits > 0 {
                    let cov = hits as f32 * 0.25;
                    blend_px(&mut self.buf[row + x], rgb.0, rgb.1, rgb.2, alpha * cov);
                }
            }
        }
    }
}

fn inside(pts: &[(f32, f32)], px: f32, py: f32) -> bool {
    let mut inside = false;
    let mut j = pts.len() - 1;
    for i in 0..pts.len() {
        let (xi, yi) = pts[i];
        let (xj, yj) = pts[j];
        if (yi > py) != (yj > py) && px < (xj - xi) * (py - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Separable box blur over a single-channel buffer, run three times to
/// approximate a gaussian. Cheap and only ever runs at startup.
pub fn blur_alpha(src: &mut Vec<u8>, w: usize, h: usize, radius: usize) {
    if radius == 0 || w == 0 || h == 0 {
        return;
    }
    let mut tmp = vec![0u8; w * h];
    for _ in 0..3 {
        // horizontal
        for y in 0..h {
            let row = y * w;
            for x in 0..w {
                let lo = x.saturating_sub(radius);
                let hi = (x + radius).min(w - 1);
                let mut sum = 0u32;
                for i in lo..=hi {
                    sum += src[row + i] as u32;
                }
                tmp[row + x] = (sum / (hi - lo + 1) as u32) as u8;
            }
        }
        // vertical
        for x in 0..w {
            for y in 0..h {
                let lo = y.saturating_sub(radius);
                let hi = (y + radius).min(h - 1);
                let mut sum = 0u32;
                for i in lo..=hi {
                    sum += tmp[i * w + x] as u32;
                }
                src[y * w + x] = (sum / (hi - lo + 1) as u32) as u8;
            }
        }
    }
}
