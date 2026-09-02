const fs = require('fs');
const path = require('path');
const { withServer, renderFrame, chromium } = require('./render.js');

const W = parseInt(process.argv[2], 10);
const H = parseInt(process.argv[3], 10);
const OUTDIR = process.argv[4];
const SEED = 7;
const T0 = 10.0;
const STEP = 1 / 30;
const NFRAMES = 480;

fs.mkdirSync(OUTDIR, { recursive: true });

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  const t0 = Date.now();
  let n = 0;
  await withServer(async (base) => {
    for (let i = 1; i <= NFRAMES; i++) {
      const t = T0 + (i - 1) * STEP;
      const buf = await renderFrame(page, base, W, H, SEED, t);
      fs.writeFileSync(path.join(OUTDIR, `f${String(i).padStart(4, '0')}.png`), buf);
      n++;
      if (n % 50 === 0) console.log(`${n} frames, ${((Date.now() - t0) / 1000).toFixed(1)}s`);
    }
    console.log(`done: ${n} frames in ${((Date.now() - t0) / 1000).toFixed(1)}s`);
  });
  await browser.close();
})();
