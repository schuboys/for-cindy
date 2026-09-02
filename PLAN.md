# For Cindy — the pink drift, bundled

Cindy loved the `cindyfaves` scene from the ambient wall (cotton-candy pink sky,
floating favorites, comic bursts) and wants it as a screensaver on her Windows
work computer (Microsoft-managed — corporate security matters) and her iPhone.

Corey's approved decisions (do not relitigate):

- **Deliverables:** interactive HTML page (the hub), static wallpapers,
  Windows Photos-slideshow stills pack, real Rust `.scr`, iPhone video loop.
- **Hosting:** BOTH a Claude Artifact (preview/iterate) and GitHub Pages
  (permanent link + downloads). Pages repo under the **schuboys** org.
- **New sprites:** Minnie Mouse, Minnie's bow (separate image), golden
  retriever, cottage cheese (small, easter-egg scale). **Real images found on
  the web with transparent backgrounds — do NOT draw them.**
- **Footer label** ("CINDY'S FAVORITE THINGS") is DROPPED from wallpaper
  renders. On the live page it may appear subtly or not at all.
- **Interactivity: all of it** — tap bursts (custom words incl. "LOVE YOU!"),
  drag/fling physics on items, clicking Minnie rains bows, cottage cheese
  easter egg (rare drift-through; tapping = giggle confetti), "Snap wallpaper"
  button rendering the current frame at the device's native resolution as a
  PNG download, and a download shelf (wallpapers / stills pack / .scr / video).

Source of truth for the look: `assets/cindyfaves-original.js` (recovered from
ambient-display git history, commit `13b1daa~1`) and the five original sprites
in `assets/original/` (candy, coke, cotton, disney, tiktok). Key look elements:
slow-rotating radial sky `#ffb3d9 → #e8b3f0 → #b3dfff → #8cc8fc`, drifting
white candy stripes, twinkling 5-point stars, sprites with thick colored drop
shadows and a size pulse, comic ellipse bursts with white stroke.

Project root: `~/Projects/for-cindy` (its own repo; ambient-display is NOT
touched). Layout: `assets/` (sprites), `web/` (the Pages site), `scr/` (Rust),
`dist/` (rendered wallpapers, stills, video, zip bundles).

---

## Phase 1 — Source the four new sprites  (mech-executor)

Find and download transparent-background PNGs into `assets/new/`:

- `minnie.png` — Minnie Mouse, full character, classic red/white polka dot look
- `bow.png` — Minnie's red polka-dot bow alone
- `golden.png` — golden retriever (photo cut-out or cute illustration; happy)
- `cottage.png` — a tub/bowl of cottage cheese (this one stays SMALL in-scene)

Requirements: genuinely transparent alpha (verify: `sips -g hasAlpha` or
python/PIL check that corner pixels are transparent), ≥500px on the long edge,
< 400KB each after `pngquant`-style compression is fine (match the originals'
weight class). Good sources: stickpng.com, pngimg.com, freepnglogos, pngwing
(direct-download the image file, not the HTML page; check magic bytes with
`file`). Personal, noncommercial use.

Acceptance: 4 PNGs in `assets/new/`, `file` says PNG, alpha verified
programmatically, each ≥500px long edge. Write a one-line provenance note per
image in `assets/new/SOURCES.md`.

## Phase 2 — The interactive page  (executor)

Build `web/index.html` — a single self-contained page (CSS/JS inline; sprites
referenced as `./assets/*.png` relative paths, copied into `web/assets/`).
Port the original scene faithfully (same sky, stripes, stars, shadows,
pulse, bursts), then add:

1. Nine drifting kinds: the 5 originals + minnie, bow, golden. Cottage cheese
   is NOT a regular drifter — see 5.
2. Tap/click empty sky → comic burst at that point. Word pool: originals plus
   "LOVE YOU!", "ATL ✈", "FILLS MY SOUL", "MINNIE!", "GOOD DOG!".
3. Pointer drag on an item → it follows with squash, releases with fling
   velocity, bounces off edges. Simple verlet or velocity tracking, no physics
   lib. Tapping (not dragging) an item → pop-scale bounce + burst.
4. Tapping Minnie (or the bow) → 12–20 small bows rain down and fade.
5. Cottage cheese easter egg: every 90–180 s a small cottage cheese drifts
   across once. Tapping it → confetti eruption + "COTTAGE CHEESE!!" burst.
6. "📸 Snap wallpaper" button (small, corner, fades out when idle): renders
   the CURRENT frame to an offscreen canvas at `screen.width×screen.height ×
   devicePixelRatio` (no UI, no label) and triggers a PNG download named
   `cindy-wallpaper.png`.
7. Download shelf: a small "♥ downloads" toggle opens a panel linking to
   `dist/` files (relative links: `./downloads/…`). Panel copy is warm, brief,
   includes one-line install instructions per item (Windows wallpaper, Photos
   slideshow screensaver steps, .scr right-click→Install + SmartScreen note,
   iPhone Live-Photo route).
8. Runs at 30fps cap, `requestAnimationFrame`, handles resize/orientation,
   works on iPhone Safari (touch events) and desktop. No external requests,
   no analytics, no fonts fetched (system font stack) — the page must look
   boring to a corporate proxy.

Acceptance: opens from `file://` and from a static server with zero console
errors; all interactions above demonstrable; total page weight (excl.
downloads) < 3 MB.

## Phase 3 — Wallpapers, stills pack, video  (mech-executor, after Phase 2)

Add a query-param headless mode to the page (`?render=WxH&seed=N&t=SEC`):
deterministic seed, advances the sim to `t` seconds, draws one frame at W×H
with no UI/label, exposes `window.__renderDone` and the canvas for capture.
Then with Playwright (or the session browser) capture:

- Wallpapers → `dist/wallpapers/`: 1920×1080, 2560×1440, 3840×2160,
  iphone 1290×2796. Pick 3 distinct seeds each; keep the best-spread frame
  per size (no overlapping sprites over the center; mechanical check only —
  Corey eyeballs final).
- Stills pack → `dist/stills/still-01.png … still-20.png` at 1920×1080,
  20 different seeds/times, plus `README.txt` with the Photos-screensaver
  setup steps.
- Video → `dist/cindy-loop.mp4`: capture 450 frames (15 s @ 30fps) at
  1290×2796 via the headless mode stepping `t`, assemble with ffmpeg
  (h264, yuv420p, crf 20). Must loop seamlessly-ish: pick a stretch without
  bursts. Also `dist/cindy-loop-desktop.mp4` at 1920×1080.

Zip: `dist/downloads/wallpapers.zip`, `stills-screensaver.zip`. Copy mp4s in.

## Phase 4 — The Rust .scr  (executor; Rust, so never mech)

`scr/` — a small crate, `winit + softbuffer + tiny-skia` (or `image` +
manual compositing; executor's call, but no GPU deps, no webview). Recreate
the drift faithfully: radial-ish sky gradient (approximation fine), drifting
sprites (embed PNGs with `include_bytes!`, decode with `image`), wrap/bounce,
size pulse, stars/sparkles. Bursts optional — skip if it pads scope.

Screensaver contract: handle args `/s` (run fullscreen), `/c` (config —
just exit 0 or a message-box-free no-op), `/p <hwnd>` (preview — exit 0).
Exit on any mouse-move (> small threshold), click, or key. Multi-monitor:
primary monitor fullscreen is enough.

Build: `cargo build --release --target x86_64-pc-windows-gnu`, copy to
`dist/downloads/CindyDrift.scr`. Acceptance: binary builds clean, < 10 MB,
`file` says PE32+ executable. (Runtime testing on real Windows is Corey's;
note that in the summary. SmartScreen warning is expected — instructions in
the download shelf must say "More info → Run anyway".)

## Phase 5 — Publish  (main session)

- New repo `schuboys/for-cindy` (public), push `web/` as Pages root with
  `dist/downloads/` included. Enable Pages. URL like
  `https://schuboys.github.io/for-cindy/`.
- Claude Artifact: inlined single-file variant (sprites as data URIs,
  download shelf links out to the Pages URL).
- Verify Pages serves, downloads download, page runs on mobile emulation.

## Phase 6 — Verify  (verifier)

Claim: all deliverables exist, match the approved decisions above, page is
self-contained/no external requests, .scr is a Windows PE, wallpapers have
no label text, cottage cheese is small and rare, Minnie/bow/golden/cottage
sprites have real transparency.

---

Rules: no silent deviation — stop and ask. Corey eyeballs aesthetics; agents
verify only mechanics. Do not touch ambient-display.

---

# Round 2 — Corey's feedback (2026-09-02)

Approved changes, in order:

## Phase 7 — Variant arrays + Disney category + burst redesign  (executor)

Edit web/index.html only.

1. **Variant arrays.** Replace the one-file-per-kind sprite model with a
   literal manifest of categories, each an ARRAY of image files. An item
   spawning in a category picks a variant uniformly (seeded RNG). Categories:
   - `coke`: [coke.png, + coke2.png if sourced]
   - `cotton`: [cotton.png]
   - `disney`: [disney.png (castle), minnie.png, minnie2.png, ears1.png,
     ears2.png, bow.png] — Minnie is generalized to Disney; the bow joins it.
     Bow-rain trigger: tapping ANY disney-category item rains bows.
   - `candy`: [candy1.png, candy2.png, candy3.png] — the candy PIECES replace
     the app icon (drop candy.png from the drift). Candy pieces render
     smaller than other sprites (they're game tokens, ~60-70% normal size).
   - `golden`: [golden.png, golden2.png, golden3.png]
   - `tiktok`: [tiktok.png]
   Keep 16-18 items with a spread across categories (weight disney and
   golden a bit higher — they're her favorites). The manifest must be a
   single obvious block at the top of the script with a comment telling
   Corey how to add his own curated files (drop PNG in assets/, add filename
   to the array).
2. **Burst redesign — old-school Batman starburst, rare and subtle.**
   - Shape: irregular spiky comic starburst polygon (11-14 points, alternating
     outer/inner radii with seeded jitter), NOT an ellipse. Flat saturated
     fill, thick white outline plus offset black outline underneath for the
     classic print look, slight rotation (±10°). Bold slanted uppercase text.
   - Cadence: ambient bursts every 25-45 s (was ~0.7-2 s), one on screen at
     a time, smaller than before (max ~12% of H), pop-in scale then gentle
     fade over ~2.5 s. Subtle > loud.
   - Ambient word pool = loving only: LOVE YOU! · SO LOVED! · MISS YOU! ·
     XOXO! · FILLS MY SOUL! · HUGS! (drop ATL ✈, YUM!, LIKE!, OMG!, WOW!,
     SWEET!, YAY! from ambient).
   - Tap-triggered bursts keep the new visual style; tap word pool = the
     loving pool + classic POP! BAM! (her own taps can be playful), plus the
     contextual GOOD DOG! (golden), MINNIE! (disney), COTTAGE CHEESE!!
     (easter egg) unchanged.
   - Render/headless mode: still zero bursts (unchanged gate).
3. Everything else (drag/fling, cottage cheese, snap, shelf, headless
   determinism contract) unchanged.

## Phase 8 — Re-render media + republish  (mech-executor, then main session)

Re-run wallpapers (same chosen seeds), stills, seamless videos with the new
sprite mix (tools/ scripts exist: make_wallpapers_fixed.js, make_stills.js,
make_video_frames_480.js + build_seamless.sh); rebuild zips; mirror into
web/downloads/ (CindyDrift.scr untouched). Main session commits, pushes
main + gh-pages, rebuilds + republishes the artifact variant.

The .scr keeps the old single-image set for now — updating it is a later
round once Corey has curated final images.
