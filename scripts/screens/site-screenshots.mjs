import { join } from 'node:path';
import { OUT_DIR, ROOT, VIEWPORTS, shoot } from './playwright-browser.mjs';
import { withScene } from './static-site-scene.mjs';

const ORIGIN = 'http://site.local';
const DIR = join(ROOT, 'site');

// An iPhone user agent on the phone width, so Get started opens on its Device tab as on a real phone.
const IPHONE_UA = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
const TAB = { desktop: 'Agent tab (computers)', mobile: 'Device tab with the iPhone install path (phones)' };

/** Landing page at both widths, plus the skip link that shows on keyboard focus. */
export async function captureSite(browser) {
  const shots = [];
  for (const viewport of ['desktop', 'mobile']) {
    const file = `site-landing-${viewport}.png`;
    const userAgent = viewport === 'mobile' ? IPHONE_UA : undefined;
    await withScene(browser, { viewport, origin: ORIGIN, dir: DIR, userAgent }, async (page) => {
      await page.goto(`${ORIGIN}/`);
      await shoot(page, join(OUT_DIR, file), { fullPage: true });
    });
    shots.push({
      file,
      shows: `Landing page at ${VIEWPORTS[viewport].width} px: hero, how it works, sample questions, get started opened on its ${TAB[viewport]}, privacy.`,
    });
  }

  const file = 'site-landing-skip-link-focus-desktop.png';
  await withScene(browser, { viewport: 'desktop', origin: ORIGIN, dir: DIR }, async (page) => {
    await page.goto(`${ORIGIN}/`);
    await page.keyboard.press('Tab');
    await shoot(page, join(OUT_DIR, file), { fullPage: true });
  });
  shots.push({ file, shows: 'Landing page with keyboard focus on the "Skip to content" link.' });
  return shots;
}
