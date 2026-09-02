const fs = require('fs');
const path = require('path');
const { withServer, renderFrame, chromium } = require('./render.js');

const DIST = path.join(__dirname, '..', 'dist', 'stills');
fs.mkdirSync(DIST, { recursive: true });

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await withServer(async (base) => {
    for (let seed = 1; seed <= 20; seed++) {
      const t = 8 + seed * 3;
      const buf = await renderFrame(page, base, 1920, 1080, seed, t);
      const num = String(seed).padStart(2, '0');
      fs.writeFileSync(path.join(DIST, `still-${num}.png`), buf);
      console.log(`still-${num}.png seed=${seed} t=${t}`);
    }
  });
  await browser.close();
})();
