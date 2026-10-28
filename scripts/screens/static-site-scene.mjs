import { readFile } from 'node:fs/promises';
import { extname, join, normalize, sep } from 'node:path';
import { FAKE, ROOT, VIEWPORTS } from './playwright-browser.mjs';

// The site deploy copies design-system/ into the published site; serve it the same way here.
const DESIGN_SYSTEM = join(ROOT, 'design-system');

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

async function serveFile(route, dir, pathname, mount) {
  // `mount` is the URL folder that `dir` stands for, e.g. "/docs" for the built docs.
  const inside = mount && pathname.startsWith(mount) ? pathname.slice(mount.length) : pathname;
  let file = normalize(join(dir, decodeURIComponent(inside)));
  if (!file.startsWith(dir + sep) && file !== dir) return route.fulfill({ status: 403, body: 'forbidden' });
  if (pathname.endsWith('/')) file = join(file, 'index.html');
  else if (!extname(file)) file += '.html'; // clean URLs: /protocol -> protocol.html
  try {
    const body = await readFile(file);
    return route.fulfill({ status: 200, contentType: TYPES[extname(file)] ?? 'application/octet-stream', body });
  } catch {
    return route.fulfill({ status: 404, body: 'not found' });
  }
}

/** Stubs match by URL prefix first. Own origin is served from disk. Anything else is aborted. */
async function routeRequest(route, { origin, dir, stubs, mount }) {
  const href = route.request().url();
  const stub = stubs.find(([prefix]) => href.startsWith(prefix));
  if (stub) return stub[1](route, route.request());
  const { pathname } = new URL(href);
  if (href.startsWith(origin + '/design-system/')) return serveFile(route, DESIGN_SYSTEM, pathname, '/design-system');
  if (href.startsWith(origin + '/')) return serveFile(route, dir, pathname, mount);
  return route.abort();
}

/** One isolated context per state. Runs `run(page)`, then closes the context. */
export async function withScene(browser, { viewport, origin, dir, mount = '', stubs = [], init = [], colorScheme }, run) {
  const context = await browser.newContext({
    viewport: VIEWPORTS[viewport],
    deviceScaleFactor: 1,
    colorScheme: colorScheme ?? process.env.SCREENS_SCHEME ?? 'light',
    reducedMotion: 'reduce',
    locale: 'en-US',
    timezoneId: 'UTC',
    serviceWorkers: 'block',
  });
  try {
    await context.route('**/*', (route) => routeRequest(route, { origin, dir, stubs, mount }));
    const page = await context.newPage();
    await page.clock.setFixedTime(FIXED_TIME);
    for (const [fn, arg] of init) await page.addInitScript(fn, arg);
    const rootFont = process.env.SCREENS_ROOT_FONT; // large-text audit, e.g. "20px"
    if (rootFont) await page.addInitScript((size) => addEventListener('DOMContentLoaded', () => { document.documentElement.style.fontSize = size; }), rootFont);
    await run(page);
  } finally {
    await context.close();
  }
}
