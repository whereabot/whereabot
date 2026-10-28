// Renders PNG screenshots of every page and visible state into docs/screens/.
// Usage: node scripts/screens.mjs   (needs Playwright and its Chromium; see scripts/screens/playwright-browser.mjs)
import { mkdirSync, readdirSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { OUT_DIR, launch } from './screens/playwright-browser.mjs';
import { captureApp } from './screens/app-screenshots.mjs';
import { captureConsent } from './screens/consent-screenshots.mjs';
import { captureSite } from './screens/site-screenshots.mjs';
import { writeReadme } from './screens/screens-readme-writer.mjs';

mkdirSync(OUT_DIR, { recursive: true });
for (const file of readdirSync(OUT_DIR)) {
  if (file.endsWith('.png')) rmSync(join(OUT_DIR, file));
}

const browser = await launch();
const shots = [];
try {
  shots.push(...(await captureSite(browser)));
  shots.push(...(await captureConsent(browser)));
  shots.push(...(await captureApp(browser)));
} finally {
  await browser.close();
}
writeReadme(OUT_DIR, shots);
console.log(`Wrote ${shots.length} screenshots to ${OUT_DIR}`);
