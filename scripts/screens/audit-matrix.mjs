// Layout audit: reruns scripts/screens.mjs at 320, 390 and 1280 px, light and dark, and with large text.
// Usage: node scripts/screens/audit-matrix.mjs [outDir]   Writes outside the repo by default; never commits.
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const SCREENS = fileURLToPath(new URL('../screens.mjs', import.meta.url));
const OUT = process.argv[2] ?? join(tmpdir(), 'whereabot-screens-audit');

// The "mobile" viewport takes each width; desktop pages stay at 1280 in every run.
const RUNS = [];
for (const width of [320, 390, 1280]) {
  for (const scheme of ['light', 'dark']) RUNS.push({ width, scheme, font: '' });
}
RUNS.push({ width: 320, scheme: 'light', font: '20px' }, { width: 390, scheme: 'dark', font: '20px' });

for (const { width, scheme, font } of RUNS) {
  const dir = join(OUT, `${width}-${scheme}${font ? `-text${font}` : ''}`);
  console.log(`== ${dir}`);
  execFileSync(process.execPath, [SCREENS], {
    stdio: 'inherit',
    env: { ...process.env, SCREENS_OUT: dir, SCREENS_MOBILE_WIDTH: String(width), SCREENS_SCHEME: scheme, SCREENS_ROOT_FONT: font },
  });
}
console.log(`Audit screenshots in ${OUT}`);
