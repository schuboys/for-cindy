const fs = require('fs');
const path = require('path');
const { withServer, renderFrame, getItemSpread, chromium } = require('./render.js');

const DIST = path.join(__dirname, '..', 'dist', 'wallpapers');
const ALTS = path.join(DIST, 'alts');
fs.mkdirSync(DIST, { recursive: true });
fs.mkdirSync(ALTS, { recursive: true });

const SIZES = [
  { w: 1920, h: 1080, tag: '1920x1080' },
  { w: 2560, h: 1440, tag: '2560x1440' },
  { w: 3840, h: 2160, tag: '3840x2160' },
  { w: 1290, h: 2796, tag: '1290x2796' },
];
const SEEDS = [7, 21, 42];
const T = 12;

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  const results = {};
  await withServer(async (base) => {
    for (const sz of SIZES) {
      const candidates = [];
      for (const seed of SEEDS) {
        const buf = await renderFrame(page, base, sz.w, sz.h, seed, T);
        const spread = await getItemSpread(page);
        candidates.push({ seed, buf, spread });
        console.log(`${sz.tag} seed=${seed} overlapFrac=${spread.overlapFrac.toFixed(3)} bigCount=${spread.bigCount} hasItems=${spread.hasItems}`);
      }
      // pick lowest center-overlap fraction (best spread = least clutter over center)
      candidates.sort((a, b) => a.spread.overlapFrac - b.spread.overlapFrac);
      const chosen = candidates[0];
      const runnersUp = candidates.slice(1);
      const outPath = path.join(DIST, `cindy-wallpaper-${sz.tag}.png`);
      fs.writeFileSync(outPath, chosen.buf);
      for (const r of runnersUp) {
        fs.writeFileSync(path.join(ALTS, `cindy-wallpaper-${sz.tag}-seed${r.seed}.png`), r.buf);
      }
      results[sz.tag] = { chosen: chosen.seed, runnersUp: runnersUp.map(r => r.seed) };
    }
  });
  await browser.close();
  console.log(JSON.stringify(results, null, 2));
})();
