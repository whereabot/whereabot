import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { OUT_DIR, ROOT, VIEWPORTS, shoot } from './playwright-browser.mjs';
import { withScene } from './static-site-scene.mjs';

const ORIGIN = 'http://site.local';
const DIR = join(ROOT, 'docs-site', '.vitepress', 'dist');
const PAGES = [
  { slug: 'home', path: '/docs/', shows: 'docs home: hero and three feature cards' },
  { slug: 'mcp-tool-reference', path: '/docs/reference/mcp-tools', shows: 'the generated MCP tool reference' },
];

/** Docs pages at both widths. Needs the docs built first: `cd docs-site && npm ci && npm run build`. */
export async function captureDocs(browser) {
  if (!existsSync(join(DIR, 'index.html'))) {
    throw new Error('Docs are not built. Run `cd docs-site && npm ci && npm run build` first.');
  }
  const shots = [];
  for (const { slug, path, shows } of PAGES) {
    for (const viewport of ['desktop', 'mobile']) {
      const file = `docs-${slug}-${viewport}.png`;
      await withScene(browser, { viewport, origin: ORIGIN, dir: DIR, mount: '/docs' }, async (page) => {
        await page.goto(`${ORIGIN}${path}`);
        await shoot(page, join(OUT_DIR, file), { fullPage: true });
      });
      shots.push({ file, shows: `Docs at ${VIEWPORTS[viewport].width} px: ${shows}.` });
    }
  }
  return shots;
}
