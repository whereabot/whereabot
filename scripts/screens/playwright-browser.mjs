import { createRequire } from 'node:module';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

export const ROOT = fileURLToPath(new URL('../..', import.meta.url));
// Audit runs (scripts/screens/audit-matrix.mjs) override these: SCREENS_OUT, SCREENS_MOBILE_WIDTH, SCREENS_SCHEME, SCREENS_ROOT_FONT.
export const OUT_DIR = process.env.SCREENS_OUT ?? join(ROOT, 'docs', 'screens');
export const VIEWPORTS = {
  desktop: { width: 1280, height: 800 },
  mobile: { width: Number(process.env.SCREENS_MOBILE_WIDTH ?? 390), height: 844 },
  narrow: { width: 320, height: 640 },
};

/** Fake values only. Every Supabase call is stubbed, so no real project is reached. */
export const FAKE = {
  supabaseUrl: 'https://fake-project.supabase.co',
  anonKey: 'fake-anon-key',
  sessionKey: 'sb-fake-project-auth-token',
};

export function loadPlaywright() {
  try {
    return createRequire(import.meta.url)('playwright');
  } catch {
    throw new Error('Playwright not found. Run `npm i -g playwright` or set NODE_PATH to a folder with it.');
  }
}

export function launch() {
  // Software raster and fixed font hinting keep pixels the same on every machine and run.
  const args = ['--font-render-hinting=none', '--disable-lcd-text', '--disable-gpu', '--disable-gpu-rasterization', '--disable-partial-raster'];
  return loadPlaywright().chromium.launch({ args });
}

/** Waits for fonts and images, then captures. Injecting CSS here made pixels vary between runs. */
export async function shoot(page, file, { fullPage = false } = {}) {
  await page.evaluate(() => Promise.all([
    document.fonts.ready,
    ...[...document.images].map((img) => (img.complete ? null : new Promise((done) => { img.onload = img.onerror = done; }))),
  ]));
  if (process.env.SCREENS_OUT) {
    // Audit runs: full page, and flag sideways scroll (wider than the viewport).
    const over = await page.evaluate(() => document.documentElement.scrollWidth - innerWidth);
    if (over > 0) console.warn(`OVERFLOW ${over}px ${file}`);
    fullPage = true;
  }
  await page.screenshot({ path: file, fullPage, animations: 'disabled', caret: 'hide' });
}
