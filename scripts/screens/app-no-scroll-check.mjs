// No-scroll check: every app screen at every phone, tablet and desktop size must fit the viewport.
// The page itself must never scroll, and nothing may stick out sideways. Long lists scroll inside their own box.
// Usage: node scripts/screens/app-no-scroll-check.mjs [outDir]   (needs Playwright; see playwright-browser.mjs)
// Writes one screenshot per screen and size to outDir (outside the repo by default). Exits 1 on any problem.
import { mkdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { FAKE, launch, shoot } from './playwright-browser.mjs';
import { sessionInit, withScene } from './static-site-scene.mjs';
import { installTauri } from './tauri-stub.mjs';
import { CASES, EMPTY_JSON, ORIGIN, VALID_RELAY, buildApp, drive } from './app-screenshots.mjs';
import { PWA_CASES } from './app-pwa-cases.mjs';
import { FIT_VIEWPORTS, measureFit } from './viewport-fit-measure.mjs';

const OUT = process.argv[2] ?? join(tmpdir(), 'whereabot-no-scroll');
const ALL = [...CASES, ...PWA_CASES];
const failures = [];

const browser = await launch();
try {
  for (const vp of FIT_VIEWPORTS) {
    mkdirSync(join(OUT, vp.name), { recursive: true });
    for (const c of ALL) {
      const tauri = c.pwa ? [] : [[installTauri, { permission: 'granted', connect: 'ok', ...c.tauri }]];
      await withScene(browser, {
        viewport: { width: vp.width, height: vp.height },
        userAgent: c.ua,
        permissions: c.grant,
        origin: ORIGIN,
        dir: buildApp(c.relay ?? VALID_RELAY),
        stubs: [[FAKE.supabaseUrl, EMPTY_JSON]],
        init: [...(c.session ? [sessionInit()] : []), ...tauri, ...(c.pageInit ? [[c.pageInit]] : [])],
      }, async (page) => {
        await page.goto(`${ORIGIN}/`);
        await drive(page, c);
        const problems = await page.evaluate(measureFit);
        await shoot(page, join(OUT, vp.name, `${c.name}.png`));
        for (const p of problems) failures.push(`${vp.name} ${c.name}: ${p}`);
      });
    }
  }
} finally {
  await browser.close();
}
console.log(`Checked ${ALL.length} screens x ${FIT_VIEWPORTS.length} sizes. Screenshots in ${OUT}`);
if (failures.length) {
  console.error(failures.join('\n'));
  process.exit(1);
}
