// Gx · Cindy's favorite things — loud, silly, and unapologetically fun.
//
// Real product/logos (Diet Coke, cotton candy, Disney castle, Candy Crush, TikTok)
// drift and bounce across a candy sky while comic "BAM! / POP!" bursts flash on
// screen. Everything is Canvas2D drawImage() + lightweight shapes so it stays
// smooth on the J4125 kiosk at 30 fps.

import { CanvasScene } from "./canvas-scene.js";
import { grain, vignette } from "../lib/frame-fx.js";

export const name = "cindyfaves";
export const pacing = { duration: 75, floor: 40, decayRatio: 0.85 };

export function create(ctx) {
  return new CindyFaves(ctx);
}

const ITEM_COUNT = 16;
const SWIRL_SPEED = 0.04;

const IMAGES = {
  coke: "/branding/cindyfaves/coke.png",
  cotton: "/branding/cindyfaves/cotton.png",
  disney: "/branding/cindyfaves/disney.png",
  candy: "/branding/cindyfaves/candy.png",
  tiktok: "/branding/cindyfaves/tiktok.png",
};

const KINDS = Object.keys(IMAGES);

// Comic bursts that randomly pop on top of items.
const BURST_WORDS = ["BAM!", "POP!", "WOW!", "SWEET!", "YUM!", "LIKE!", "OMG!", "YAY!"];
const BURST_COLORS = ["#ff2a6d", "#05d9e8", "#ff9f1c", "#7b2cbf", "#ff006e"];

class CindyFaves extends CanvasScene {
  constructor(ctx) {
    super(ctx);
    this._items = [];
    this._sparkles = [];
    this._stars = [];
    this._bursts = [];
    this._sprites = {};
    this._spritesReady = false;
    this._nextBurst = 0;
    this._loadImages();
  }

  enter() {
    super.enter();
    this._seedItems();
    this._seedSparkles();
    this._seedStars();
    this._bursts = [];
    this._nextBurst = 0;
  }

  setSize(w, h, dpr) {
    super.setSize(w, h, dpr);
    this._seedItems();
    this._seedSparkles();
    this._seedStars();
  }

  _loadImages() {
    let pending = KINDS.length;
    for (const kind of KINDS) {
      const img = new Image();
      img.crossOrigin = "anonymous";
      img.onload = () => {
        pending--;
        if (pending === 0) this._spritesReady = true;
      };
      img.onerror = () => {
        pending--;
        console.warn(`cindyfaves: failed to load ${IMAGES[kind]}`);
        if (pending === 0) this._spritesReady = true;
      };
      img.src = IMAGES[kind];
      this._sprites[kind] = img;
    }
  }

  draw(ctx, W, H, t, dt) {
    const s = H / 1080;

    // Cotton-candy sky: a slow-rotating radial wash.
    const cx = W * (0.45 + 0.1 * Math.sin(t * SWIRL_SPEED));
    const cy = H * (0.35 + 0.08 * Math.cos(t * SWIRL_SPEED * 0.7));
    const sky = ctx.createRadialGradient(cx, cy, 0, cx, cy, Math.max(W, H) * 1.1);
    sky.addColorStop(0, "#ffb3d9"); // hot cotton-candy pink
    sky.addColorStop(0.4, "#e8b3f0"); // bright lavender
    sky.addColorStop(0.75, "#b3dfff"); // baby blue
    sky.addColorStop(1, "#8cc8fc");
    ctx.fillStyle = sky;
    ctx.fillRect(0, 0, W, H);

    // Bold candy stripes drifting behind everything.
    ctx.save();
    ctx.globalAlpha = 0.12;
    for (let i = 0; i < 6; i++) {
      const x = ((t * 18 + i * 240) * s) % (W + 360 * s) - 180 * s;
      const y = H * (0.1 + i * 0.16);
      const w = 120 * s;
      const grad = ctx.createLinearGradient(x, 0, x + w, 0);
      grad.addColorStop(0, "rgba(255,255,255,0)");
      grad.addColorStop(0.5, "rgba(255,255,255,0.7)");
      grad.addColorStop(1, "rgba(255,255,255,0)");
      ctx.fillStyle = grad;
      ctx.beginPath();
      ctx.ellipse(x + w / 2, y, w / 2, 22 * s, 0, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();

    // Floating stars in the background.
    for (const star of this._stars) {
      const a = 0.35 + 0.35 * Math.sin(t * star.speed + star.phase);
      const r = star.r * (0.8 + 0.2 * Math.sin(t * star.speed * 2 + star.phase));
      ctx.save();
      ctx.translate(star.x, star.y);
      ctx.rotate(t * star.spin + star.phase);
      ctx.globalAlpha = a;
      ctx.fillStyle = star.color;
      this._drawStar(ctx, 0, 0, 5, r, r * 0.45);
      ctx.restore();
    }

    // Drifting favorites.
    for (const item of this._items) {
      item.x += item.vx * dt * s;
      item.y += item.vy * dt * s + Math.sin(t * item.wobble + item.phase) * 0.5 * s;
      item.rot += item.vRot * dt;
      item.pulse = 1 + 0.12 * Math.sin(t * 3.5 + item.phase);

      // Wrap horizontally, bounce vertically.
      const margin = item.size * 0.6;
      if (item.x > W + margin) item.x = -margin;
      if (item.x < -margin) item.x = W + margin;
      if (item.y > H - margin || item.y < margin) item.vy *= -1;

      const sprite = this._sprites[item.kind];
      if (!sprite || !sprite.complete || sprite.naturalWidth === 0) continue;

      ctx.save();
      ctx.translate(item.x, item.y);
      ctx.rotate(item.rot);
      ctx.scale(item.pulse, item.pulse);

      const sw = item.size;
      const sh = (sprite.naturalHeight / sprite.naturalWidth) * sw;

      // Thick colored drop shadow makes the sprite "pop" off the sky.
      ctx.save();
      ctx.shadowColor = item.shadowColor;
      ctx.shadowBlur = 28 * s;
      ctx.shadowOffsetX = 8 * s;
      ctx.shadowOffsetY = 12 * s;
      ctx.drawImage(sprite, -sw / 2, -sh / 2, sw, sh);
      ctx.restore();

      ctx.restore();
    }

    // Comic text bursts.
    this._updateBursts(dt, t, s);
    for (const b of this._bursts) {
      const life = b.life / b.maxLife;
      const scale = Math.sin((1 - life) * Math.PI * 0.9) * b.maxScale;
      ctx.save();
      ctx.translate(b.x, b.y);
      ctx.scale(scale, scale);
      ctx.rotate(b.rot);
      ctx.globalAlpha = Math.max(0, life);

      // Bubble background.
      ctx.fillStyle = b.color;
      ctx.beginPath();
      ctx.ellipse(0, 0, b.w, b.h, 0, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = "#ffffff";
      ctx.lineWidth = 5 * s;
      ctx.stroke();

      // Text.
      ctx.fillStyle = "#ffffff";
      ctx.font = `900 ${b.fontSize}px "Inter", "Helvetica Neue", sans-serif`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.strokeStyle = "rgba(0,0,0,0.25)";
      ctx.lineWidth = 3 * s;
      ctx.strokeText(b.text, 0, 0);
      ctx.fillText(b.text, 0, 0);
      ctx.restore();
    }

    // Soft twinkles.
    ctx.fillStyle = "#ffffff";
    for (const sp of this._sparkles) {
      const a = 0.4 + 0.5 * Math.sin(t * sp.speed + sp.phase);
      const r = sp.r * (0.7 + 0.3 * Math.sin(t * sp.speed * 1.7 + sp.phase));
      ctx.globalAlpha = a;
      ctx.beginPath();
      ctx.arc(sp.x, sp.y, r, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.globalAlpha = 1;

    // Footer label.
    ctx.save();
    ctx.textAlign = "center";
    ctx.textBaseline = "alphabetic";
    ctx.font = `800 ${24 * s}px "Inter", "Helvetica Neue", sans-serif`;
    ctx.letterSpacing = `${0.22 * s}px`;
    ctx.fillStyle = "rgba(140, 50, 120, 0.65)";
    ctx.strokeStyle = "rgba(255,255,255,0.5)";
    ctx.lineWidth = 2 * s;
    const label = "CINDY'S FAVORITE THINGS";
    ctx.strokeText(label, W / 2, H - H * 0.04);
    ctx.fillText(label, W / 2, H - H * 0.04);
    ctx.restore();

    vignette(ctx, W, H, { strength: 0.25, radius: 0.6 });
    grain(ctx, W, H, { alpha: 0.04 });
  }

  _updateBursts(dt, t, s) {
    // Spawn a new burst every so often once sprites are ready.
    if (this._spritesReady && t > this._nextBurst) {
      const anchor = this._items[Math.floor(Math.random() * this._items.length)];
      if (anchor) {
        const word = BURST_WORDS[Math.floor(Math.random() * BURST_WORDS.length)];
        const color = BURST_COLORS[Math.floor(Math.random() * BURST_COLORS.length)];
        this._bursts.push({
          x: anchor.x + (Math.random() - 0.5) * 80 * s,
          y: anchor.y - anchor.size * 0.6 - (20 + Math.random() * 40) * s,
          text: word,
          color,
          w: (35 + word.length * 12) * s,
          h: 42 * s,
          fontSize: 32 * s,
          rot: (Math.random() - 0.5) * 0.6,
          maxScale: 1.4 + Math.random() * 0.6,
          life: 1.2,
          maxLife: 1.2,
        });
      }
      this._nextBurst = t + 0.7 + Math.random() * 1.3;
    }

    // Age out bursts.
    for (let i = this._bursts.length - 1; i >= 0; i--) {
      this._bursts[i].life -= dt;
      if (this._bursts[i].life <= 0) this._bursts.splice(i, 1);
    }
  }

  _drawStar(ctx, cx, cy, spikes, outerRadius, innerRadius) {
    let rot = (Math.PI / 2) * 3;
    let x = cx;
    let y = cy;
    let step = Math.PI / spikes;
    ctx.beginPath();
    ctx.moveTo(cx, cy - outerRadius);
    for (let i = 0; i < spikes; i++) {
      x = cx + Math.cos(rot) * outerRadius;
      y = cy + Math.sin(rot) * outerRadius;
      ctx.lineTo(x, y);
      rot += step;
      x = cx + Math.cos(rot) * innerRadius;
      y = cy + Math.sin(rot) * innerRadius;
      ctx.lineTo(x, y);
      rot += step;
    }
    ctx.lineTo(cx, cy - outerRadius);
    ctx.closePath();
    ctx.fill();
  }

  _seedItems() {
    const W = this._w;
    const H = this._h;
    this._items = [];
    for (let i = 0; i < ITEM_COUNT; i++) {
      const kind = KINDS[i % KINDS.length];
      this._items.push({
        kind,
        x: Math.random() * W,
        y: Math.random() * H * 0.85 + H * 0.08,
        size: (110 + Math.random() * 90) * (H / 1080),
        vx: (10 + Math.random() * 22) * (Math.random() < 0.5 ? 1 : -1),
        vy: (6 + Math.random() * 14) * (Math.random() < 0.5 ? 1 : -1),
        rot: (Math.random() - 0.5) * 0.6,
        vRot: (Math.random() - 0.5) * 0.45,
        wobble: 0.8 + Math.random() * 1.4,
        phase: Math.random() * Math.PI * 2,
        pulse: 1,
        shadowColor: this._shadowFor(kind),
      });
    }
  }

  _shadowFor(kind) {
    switch (kind) {
      case "coke":
        return "rgba(255, 30, 80, 0.55)";
      case "cotton":
        return "rgba(255, 120, 220, 0.55)";
      case "disney":
        return "rgba(120, 80, 220, 0.55)";
      case "candy":
        return "rgba(255, 170, 0, 0.55)";
      case "tiktok":
        return "rgba(5, 217, 232, 0.55)";
      default:
        return "rgba(0,0,0,0.4)";
    }
  }

  _seedSparkles() {
    const W = this._w;
    const H = this._h;
    this._sparkles = [];
    for (let i = 0; i < 22; i++) {
      this._sparkles.push({
        x: Math.random() * W,
        y: Math.random() * H,
        r: (2 + Math.random() * 5) * (H / 1080),
        speed: 1.5 + Math.random() * 2.5,
        phase: Math.random() * Math.PI * 2,
      });
    }
  }

  _seedStars() {
    const W = this._w;
    const H = this._h;
    this._stars = [];
    for (let i = 0; i < 14; i++) {
      this._stars.push({
        x: Math.random() * W,
        y: Math.random() * H,
        r: (6 + Math.random() * 10) * (H / 1080),
        speed: 0.7 + Math.random() * 1.2,
        spin: (Math.random() - 0.5) * 0.8,
        phase: Math.random() * Math.PI * 2,
        color: ["#ffd6e8", "#fff7ad", "#b3f0ff", "#e8b3f0"][Math.floor(Math.random() * 4)],
      });
    }
  }
}
