import { readFile } from 'node:fs/promises';
import { extname, join, normalize, sep } from 'node:path';
import { FAKE, VIEWPORTS } from './browser.mjs';

const FIXED_TIME = new Date('2026-01-15T12:00:00Z');
const TYPES = {
  '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.mjs': 'text/javascript',
  '.svg': 'image/svg+xml', '.json': 'application/json', '.png': 'image/png',
};

/** Runs in the page before any app script. Seeds a signed-in Supabase session. */
function seedSessionInPage(arg) {
  localStorage.setItem(arg.key, arg.value);
}

const SESSION = {
  access_token: 'fake-access-token',
  token_type: 'bearer',
  expires_in: 3600,
  expires_at: 4102444800, // 2100, so no refresh call is made
  refresh_token: 'fake-refresh-token',
  user: {
    id: '00000000-0000-4000-8000-000000000001',
    aud: 'authenticated',
    email: 'user@example.test',
    app_metadata: {},
    user_metadata: {},
    created_at: '2026-01-01T00:00:00Z',
  },
};

export function sessionInit() {
  return [seedSessionInPage, { key: FAKE.sessionKey, value: JSON.stringify(SESSION) }];
}

async function serveFile(route, dir, pathname) {
  let file = normalize(join(dir, decodeURIComponent(pathname)));
  if (!file.startsWith(dir + sep) && file !== dir) return route.fulfill({ status: 403, body: 'forbidden' });
  if (pathname.endsWith('/')) file = join(file, 'index.html');
  try {
    const body = await readFile(file);
    return route.fulfill({ status: 200, contentType: TYPES[extname(file)] ?? 'application/octet-stream', body });
  } catch {
    return route.fulfill({ status: 404, body: 'not found' });
  }
}

/** Stubs match by URL prefix first. Own origin is served from disk. Anything else is aborted. */
async function routeRequest(route, { origin, dir, stubs }) {
  const href = route.request().url();
  const stub = stubs.find(([prefix]) => href.startsWith(prefix));
  if (stub) return stub[1](route, route.request());
  if (href.startsWith(origin + '/')) return serveFile(route, dir, new URL(href).pathname);
  return route.abort();
}

/** One isolated context per state. Runs `run(page)`, then closes the context. */
export async function withScene(browser, { viewport, origin, dir, stubs = [], init = [] }, run) {
  const context = await browser.newContext({
    viewport: VIEWPORTS[viewport],
    deviceScaleFactor: 1,
    colorScheme: 'light',
    reducedMotion: 'reduce',
    locale: 'en-US',
    timezoneId: 'UTC',
    serviceWorkers: 'block',
  });
  try {
    await context.route('**/*', (route) => routeRequest(route, { origin, dir, stubs }));
    const page = await context.newPage();
    await page.clock.setFixedTime(FIXED_TIME);
    for (const [fn, arg] of init) await page.addInitScript(fn, arg);
    await run(page);
  } finally {
    await context.close();
  }
}
