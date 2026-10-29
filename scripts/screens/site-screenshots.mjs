import { join } from 'node:path';
import { OUT_DIR, ROOT, VIEWPORTS, shoot } from './playwright-browser.mjs';
import { withScene } from './static-site-scene.mjs';

const ORIGIN = 'http://site.local';
const DIR = join(ROOT, 'site');

/** Landing page at both widths, plus the skip link that shows on keyboard focus. */
export async function captureSite(browser) {
  const shots = [];
  for (const viewport of ['desktop', 'mobile']) {
    const file = `site-landing-${viewport}.png`;
    await withScene(browser, { viewport, origin: ORIGIN, dir: DIR }, async (page) => {
      await page.goto(`${ORIGIN}/`);
      await shoot(page, join(OUT_DIR, file), { fullPage: true });
    });
    shots.push({
      file,
      shows: `Landing page at ${VIEWPORTS[viewport].width} px: hero, how it works, sample questions, get started (add to your agent, install on your device), privacy.`,
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
