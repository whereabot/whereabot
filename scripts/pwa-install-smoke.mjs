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
  // The browser version states its limits in the app, before sign-in.
  await page.locator('#signed-out .pwa-limits').getByText('answers only after you tap the notification').waitFor();
  // iPhone Safari, not installed: the Add to Home Screen steps show.
  const iphone = await browser.newPage({ userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 Safari/604.1' });
  await iphone.route((url) => !url.href.startsWith(base), (route) => route.abort());
  await iphone.goto(`${base}/app/`);
  await iphone.locator('#signed-out .pwa-install').getByText('Add to Home Screen to get notifications (iOS 16.4+)').waitFor();
  assert.equal(await page.locator('.pwa-install').count(), 0, 'desktop shows no iPhone install steps');
  // Chrome/Edge: the Install Whereabot button appears once the browser fires beforeinstallprompt.
  await page.evaluate(() => dispatchEvent(Object.assign(new Event('beforeinstallprompt', { cancelable: true }), { prompt: async () => {} })));
  await page.locator('#signed-out').getByRole('button', { name: 'Install Whereabot' }).waitFor();
  // Other iOS browsers cannot add to the Home Screen the Safari way: send the user to Safari.
  const iosChrome = await browser.newPage({ userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 CriOS/130.0 Mobile/15E148 Safari/604.1' });
  await iosChrome.route((url) => !url.href.startsWith(base), (route) => route.abort());
  await iosChrome.goto(`${base}/app/`);
  await iosChrome.locator('#signed-out .pwa-install').getByText('Open in Safari to install').waitFor();
  console.log('PWA smoke passed: manifest, service worker at', scope, 'the in-app limits notice and install help');
} finally {
  await browser.close();
  server.close();
}
