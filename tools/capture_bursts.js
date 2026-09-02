const { chromium } = require('playwright');
const { spawn } = require('child_process');
(async () => {
  const srv = spawn('python3', ['-m', 'http.server', '8471'], { cwd: __dirname + '/../web' });
  await new Promise(r => setTimeout(r, 800));
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1600, height: 900 } });
  await page.goto('http://127.0.0.1:8471/index.html');
  await page.waitForTimeout(2500); // sprites load, sim running
  // trigger three tap bursts in open sky areas, staggered so all are visible at once
  await page.mouse.click(400, 250); await page.waitForTimeout(300);
  await page.mouse.click(1150, 550); await page.waitForTimeout(300);
  await page.mouse.click(750, 720); await page.waitForTimeout(250);
  const data = await page.evaluate(() => document.querySelector('canvas').toDataURL('image/png'));
  require('fs').writeFileSync('../dist/burst-preview.png', Buffer.from(data.split(',')[1], 'base64'));
  await browser.close(); srv.kill();
  console.log('saved');
})();
