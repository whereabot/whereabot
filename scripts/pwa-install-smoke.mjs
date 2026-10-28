// Smoke test for the browser app (PWA): serves app/dist-web at /app/ and checks the manifest and service worker.
// Usage: (cd app && npm run build:web) && node scripts/pwa-install-smoke.mjs   (needs Playwright; see scripts/screens/playwright-browser.mjs)
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize } from 'node:path';
import assert from 'node:assert/strict';
import { ROOT, launch } from './screens/playwright-browser.mjs';

const DIST = join(ROOT, 'app', 'dist-web');
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.webmanifest': 'application/manifest+json' };

const server = createServer(async (req, res) => {
  const path = new URL(req.url, 'http://x').pathname;
  if (!path.startsWith('/app/')) return res.writeHead(404).end();
  const rel = normalize(path.slice('/app/'.length) || 'index.html');
  if (rel.startsWith('..')) return res.writeHead(400).end();
  try {
    const body = await readFile(join(DIST, rel));
    res.writeHead(200, { 'content-type': TYPES[extname(rel)] ?? 'application/octet-stream' }).end(body);
  } catch {
    res.writeHead(404).end();
  }
});
await new Promise((ok) => server.listen(0, '127.0.0.1', ok));
const base = `http://localhost:${server.address().port}`;

const browser = await launch();
try {
  const page = await browser.newPage();
  // No relay or Supabase in this test: refuse everything that leaves the local server.
  await page.route((url) => !url.href.startsWith(base), (route) => route.abort());
  await page.goto(`${base}/app/`);
  const href = await page.locator('link[rel="manifest"]').getAttribute('href');
  assert.equal(href, '/app/app.webmanifest');
  const manifest = await (await page.request.get(base + href)).json();
  assert.equal(manifest.start_url, '/app/');
  assert.equal(manifest.display, 'standalone');
  assert.ok(manifest.icons.some((i) => i.sizes === '512x512'), 'needs a 512 px icon');
  assert.ok(await page.locator('link[rel="apple-touch-icon"]').count(), 'apple-touch-icon');
  assert.equal(await page.locator('meta[name="apple-mobile-web-app-capable"]').getAttribute('content'), 'yes');
  const scope = await page.evaluate(() => navigator.serviceWorker.ready.then((r) => r.scope));
  assert.equal(scope, `${base}/app/`);
  console.log('PWA smoke passed: manifest ok, service worker active at', scope);
} finally {
  await browser.close();
  server.close();
}
