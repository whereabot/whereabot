import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { FAKE, OUT_DIR, ROOT, shoot } from './playwright-browser.mjs';
import { AUTH_API, AUTH_ID, APPROVED, CASES, HOLD, OK, ORIGIN } from './consent-cases.mjs';
import { sessionInit, withScene } from './static-site-scene.mjs';

const DIR = join(ROOT, 'site');
const CONSENT_DIR = join(DIR, 'oauth', 'consent');

/** Fills the deploy placeholders in the real supabase-config.js with fake values. */
async function fillConfig() {
  const text = await readFile(join(CONSENT_DIR, 'supabase-config.js'), 'utf8');
  const body = text
    .replaceAll('__SUPABASE_URL__', FAKE.supabaseUrl)
    .replaceAll('__SUPABASE_ANON_KEY__', FAKE.anonKey);
  return { status: 200, contentType: 'text/javascript', body };
}

function deferred() {
  let resolve;
  const promise = new Promise((r) => (resolve = r));
  return { promise, resolve };
}

function answer(route, spec) {
  if (spec === HOLD) return new Promise(() => {});
  return route.fulfill({ status: spec.status, contentType: 'application/json', body: JSON.stringify(spec.body) });
}

function stubsFor(c, hits) {
  const authStub = (route, request) => {
    const isConsent = request.url().endsWith('/consent');
    hits[isConsent ? 'consent' : 'details'].resolve();
    return answer(route, isConsent ? (c.consent ?? APPROVED) : (c.details ?? OK));
  };
  const stubs = [[AUTH_API, authStub]];
  if (c.configured !== false) {
    stubs.push([`${ORIGIN}/oauth/consent/supabase-config.js`, async (route) => route.fulfill(await fillConfig())]);
  }
  return stubs;
}

async function until(page, what, hits) {
  if (what in hits) return hits[what].promise;
  return page.waitForFunction((state) => document.body.dataset.state === state, what);
}

async function runCase(browser, c, viewport) {
  const hits = { details: deferred(), consent: deferred() };
  const file = `consent-${c.name}-${viewport}.png`;
  const url = c.query ? `${ORIGIN}/oauth/consent/?authorization_id=${AUTH_ID}` : `${ORIGIN}/oauth/consent/`;
  await withScene(browser, {
    viewport,
    origin: ORIGIN,
    dir: DIR,
    stubs: stubsFor(c, hits),
    init: c.session ? [sessionInit()] : [],
  }, async (page) => {
    await page.goto(url);
    await until(page, c.first, hits);
    if (c.click) {
      await page.click(`#${c.click}`);
      await until(page, c.then, hits);
    }
    await shoot(page, join(OUT_DIR, file));
  });
  return { file, shows: `${c.desc} (${viewport})` };
}

/** Consent page: every state it can show, at both widths. Network and config are stubbed. */
export async function captureConsent(browser) {
  const shots = [];
  for (const c of CASES) {
    for (const viewport of ['mobile', 'desktop']) {
      shots.push(await runCase(browser, c, viewport));
    }
  }
  return shots;
}
