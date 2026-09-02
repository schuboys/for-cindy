// Cindy render driver: capture headless frames from web/index.html
const path = require('path');
const fs = require('fs');
const { chromium } = require(path.join(__dirname, 'node_modules', 'playwright'));

const WEB_DIR = path.join(__dirname, '..', 'web');

async function withServer(fn) {
  const http = require('http');
  const serve = require('./serve.js');
  const server = serve.createServer(WEB_DIR);
  await new Promise((res) => server.listen(0, '127.0.0.1', res));
  const port = server.address().port;
  try {
    await fn(`http://127.0.0.1:${port}`);
  } finally {
    server.close();
  }
}

async function renderFrame(page, base, w, h, seed, t) {
  const url = `${base}/index.html?render=${w}x${h}&seed=${seed}&t=${t}`;
  await page.goto(url, { waitUntil: 'load' });
  await page.waitForFunction(() => window.__renderDone === true, null, { timeout: 30000 });
  const dataUrl = await page.evaluate(() => window.__renderCanvas.toDataURL('image/png'));
  return Buffer.from(dataUrl.split(',')[1], 'base64');
}

async function getItemSpread(page) {
  // mechanical heuristic (pixel-variance fallback, per PLAN.md Phase 3):
  // sample a grid inside the exact center 20% box and compute luminance
  // variance. The sky background is a smooth gradient + thin stripes/stars,
  // so low variance == mostly background; a large sprite overlapping the
  // center introduces sharp color blocks and pushes variance up. Lower
  // score wins (less clutter over the center).
  return await page.evaluate(() => {
    try {
      var canvas = window.__renderCanvas;
      var W = canvas.width, H = canvas.height;
      var cx0 = Math.floor(W * 0.4), cx1 = Math.floor(W * 0.6);
      var cy0 = Math.floor(H * 0.4), cy1 = Math.floor(H * 0.6);
      var ctx2 = canvas.getContext('2d');
      var data = ctx2.getImageData(cx0, cy0, cx1 - cx0, cy1 - cy0).data;
      var n = data.length / 4;
      var sum = 0, sumSq = 0;
      for (var i = 0; i < data.length; i += 4) {
        var lum = 0.299 * data[i] + 0.587 * data[i + 1] + 0.114 * data[i + 2];
        sum += lum;
        sumSq += lum * lum;
      }
      var mean = sum / n;
      var variance = sumSq / n - mean * mean;
      return { overlapFrac: variance, mean: mean, hasItems: true };
    } catch (e) {
      return { error: String(e), overlapFrac: -1, hasItems: false };
    }
  });
}

module.exports = { withServer, renderFrame, getItemSpread, chromium };
