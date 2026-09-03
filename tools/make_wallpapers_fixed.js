const fs = require('fs');
const path = require('path');
const { withServer, renderFrame, chromium } = require('./render.js');

const DIST = path.join(__dirname, '..', 'dist', 'wallpapers');
const ALTS = path.join(DIST, 'alts');
fs.mkdirSync(DIST, { recursive: true });
fs.mkdirSync(ALTS, { recursive: true });

const SIZES = [
  { w: 1920, h: 1080, tag: '1920x1080', chosen: 3 },
  { w: 2560, h: 1440, tag: '2560x1440', chosen: 3 },
  { w: 3840, h: 2160, tag: '3840x2160', chosen: 3 },
  { w: 1290, h: 2796, tag: '1290x2796', chosen: 17 },
];
const SEEDS = [3, 17, 19];
const T = 12;

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await withServer(async (base) => {
    for (const sz of SIZES) {
      for (const seed of SEEDS) {
        const buf = await renderFrame(page, base, sz.w, sz.h, seed, T);
        if (seed === sz.chosen) {
          fs.writeFileSync(path.join(DIST, `cindy-wallpaper-${sz.tag}.png`), buf);
        } else {
          fs.writeFileSync(path.join(ALTS, `cindy-wallpaper-${sz.tag}-seed${seed}.png`), buf);
        }
        console.log(`${sz.tag} seed=${seed} ${seed === sz.chosen ? '(chosen)' : ''}`);
      }
    }
  });
  await browser.close();
})();
