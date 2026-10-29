// Smoke test for the landing's "Get started" split: both columns, copy buttons, a link to /app/,
// device-aware install help per user agent, and no sideways scroll from 320 to 1280 px.
// Usage: node scripts/landing-install-smoke.mjs   (needs Playwright; see scripts/screens/playwright-browser.mjs)
// SHOTS=<dir> also saves one full-page PNG per case.
import assert from 'node:assert/strict';
import { join } from 'node:path';
import { ROOT, launch } from './screens/playwright-browser.mjs';
import { withScene } from './screens/static-site-scene.mjs';

// localhost is a secure context, so navigator.clipboard (and the Copy buttons) exist.
const ORIGIN = 'http://localhost:4321';
const DIR = join(ROOT, 'site');
const UA = {
  iphoneSafari: 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1',
  iphoneChrome: 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/130.0.0.0 Mobile/15E148 Safari/604.1',
  androidChrome: 'Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Mobile Safari/537.36',
  desktopChrome: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36',
};
const CASES = [
  { name: 'iphone-safari', viewport: 'mobile', ua: UA.iphoneSafari, help: 'Add to Home Screen', stacked: true },
  { name: 'iphone-chrome', viewport: 'narrow', ua: UA.iphoneChrome, help: 'Open in Safari to install', stacked: true },
  { name: 'android-chrome', viewport: 'mobile', ua: UA.androidChrome, help: 'Install Whereabot', stacked: true },
  { name: 'desktop-chrome', viewport: 'desktop', ua: UA.desktopChrome, help: 'Install Whereabot', stacked: false },
];

const browser = await launch();
try {
  for (const c of CASES) {
    await withScene(browser, { viewport: c.viewport, origin: ORIGIN, dir: DIR, userAgent: c.ua }, async (page) => {
      await page.goto(`${ORIGIN}/`);
      const agent = page.locator('#agent');
      const device = page.locator('#get-app');
      await agent.getByRole('heading', { name: 'Add to your agent' }).waitFor();
      await device.getByRole('heading', { name: 'Install on your device' }).waitFor();
      assert.ok(await page.getByText('You need both: your agent asks, your phone answers.').isVisible(), `${c.name}: bridge line`);
      assert.ok(await page.locator('a[href="app/"]', { hasText: 'Open the browser app' }).isVisible(), `${c.name}: link to /app/`);
      assert.equal(await agent.getByRole('button', { name: /^Copy / }).count(), 2, `${c.name}: copy buttons`);
      await page.locator('[data-install-help]').getByText(c.help).first().waitFor();
      assert.equal(await page.locator('[data-install-help]').count(), 1, `${c.name}: one install help`);
      const [a, d] = [await agent.boundingBox(), await device.boundingBox()];
      assert.equal(d.y > a.y + a.height - 1, c.stacked, `${c.name}: columns ${c.stacked ? 'stacked' : 'side by side'}`);
      const over = await page.evaluate(() => document.documentElement.scrollWidth - innerWidth);
      assert.equal(over, 0, `${c.name}: no sideways scroll`);
      if (process.env.SHOTS) await page.screenshot({ path: join(process.env.SHOTS, `landing-${c.name}.png`), fullPage: true });
    });
  }
  console.log(`Landing install smoke passed: ${CASES.map((c) => c.name).join(', ')}`);
} finally {
  await browser.close();
}
