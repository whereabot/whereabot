import { join } from 'node:path';
import { OUT_DIR, ROOT, VIEWPORTS, shoot } from './playwright-browser.mjs';
import { withScene } from './static-site-scene.mjs';

const ORIGIN = 'http://repo.local';
const REPO = ROOT.replace(/\/$/, ''); // serveFile needs a dir without a trailing slash

/** Design-system gallery (design-system/component-gallery.html) at 320, 390 and 1280 px, light and dark. */
export async function captureGallery(browser) {
  const shots = [];
  for (const viewport of ['narrow', 'mobile', 'desktop']) {
    for (const colorScheme of ['light', 'dark']) {
      const file = `gallery-${viewport}-${colorScheme}.png`;
      await withScene(browser, { viewport, colorScheme, origin: ORIGIN, dir: REPO }, async (page) => {
        await page.goto(`${ORIGIN}/design-system/component-gallery.html`);
        await page.keyboard.press('Tab'); // shows the focus ring on the first button
        await shoot(page, join(OUT_DIR, file), { fullPage: true });
      });
      shots.push({ file, shows: `Component gallery at ${VIEWPORTS[viewport].width} px, ${colorScheme}: every shared component and state.` });
    }
  }
  return shots;
}
