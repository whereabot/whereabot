import { execFileSync } from 'node:child_process';
import { existsSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { FAKE, OUT_DIR, ROOT, shoot } from './browser.mjs';
import { sessionInit, withScene } from './scene.mjs';
import { installTauri } from './tauri-stub.mjs';

const APP = join(ROOT, 'app');
const VITE = join(APP, 'node_modules', 'vite', 'bin', 'vite.js');
const ORIGIN = 'http://app.local';
const VALID_RELAY = 'wss://relay.example.test';
const CONNECTED = 'Connected. Waiting for pings.';
const builds = new Map();

/** Builds the real app with fake env values into a temp dir, outside the repo. */
function buildApp(relayUrl) {
  if (!existsSync(VITE)) throw new Error('App dependencies missing. Run `npm ci` in app/ first.');
  if (!builds.has(relayUrl)) {
    const outDir = mkdtempSync(join(tmpdir(), 'whereabouts-screens-'));
    execFileSync(process.execPath, [VITE, 'build', '--outDir', outDir, '--emptyOutDir'], {
      cwd: APP,
      stdio: 'inherit',
      env: {
        ...process.env,
        VITE_SUPABASE_URL: FAKE.supabaseUrl,
        VITE_SUPABASE_ANON_KEY: FAKE.anonKey,
        VITE_RELAY_URL: relayUrl,
        VITE_DEV_LOGIN: 'false',
      },
    });
    builds.set(relayUrl, outDir);
  }
  return builds.get(relayUrl);
}

const CASES = [
  { name: 'signed-out', status: 'Disconnected.', desc: 'Signed out. Shows the Google sign-in button.' },
  { name: 'permission-denied', session: true, tauri: { permission: 'denied' }, status: 'Error: Location permission denied', desc: 'Location permission refused. Shows the reason. No connection.' },
  { name: 'relay-url-invalid', session: true, relay: 'http://relay.example.test', status: 'Relay URL must start with wss:// (ws:// only for localhost, 10.x or 192.168.x)', desc: 'Relay URL is not secure. Connect is refused.' },
  { name: 'connecting', session: true, tauri: { connect: 'hang' }, status: 'Connecting…', desc: 'Signed in. Opening the connection to the relay.' },
  { name: 'connect-failed', session: true, tauri: { connect: 'reject', connectError: 'relay unreachable' }, status: 'Connect failed: relay unreachable. Retrying…', desc: 'Relay could not be reached. Status shows a retry.' },
  { name: 'connected', session: true, status: CONNECTED, desc: 'Connected to the relay and waiting for a locate ping.' },
  { name: 'ping-answered', session: true, ping: true, lastPing: true, desc: 'A locate ping was answered. Shows the time of the reply.' },
  { name: 'location-failed', session: true, tauri: { locationError: 'Location services are off' }, ping: true, status: 'Location failed: Location services are off. No reply sent.', desc: 'Location read failed for a ping. No reply is sent.' },
  { name: 'connection-lost', session: true, close: true, status: 'Connection lost. Retrying…', desc: 'The socket closed. Status shows a retry.' },
  { name: 'disconnected', session: true, click: '#disconnect', status: 'Disconnected.', desc: 'Disconnect pressed. Shows Disconnected.' },
];

async function waitStatus(page, text) {
  await page.waitForFunction((t) => document.getElementById('status').textContent === t, text);
}

/** Moves the app from its start state to the state the case shows. */
async function drive(page, c) {
  if (c.ping || c.close || c.click) await waitStatus(page, CONNECTED);
  if (c.ping) {
    await page.evaluate(() => window.__screensEmit({ type: 'Text', data: JSON.stringify({ type: 'locate', id: 'ping-1' }) }));
  }
  if (c.close) await page.evaluate(() => window.__screensEmit({ type: 'Close' }));
  if (c.click) await page.click(c.click);
  if (c.lastPing) {
    await page.waitForFunction(() => document.getElementById('last-ping').textContent !== '');
  } else {
    await waitStatus(page, c.status);
  }
}

// Supabase calls return an empty JSON body. No network leaves the machine.
const EMPTY_JSON = (route) => route.fulfill({ status: 200, contentType: 'application/json', body: '{}' });

/** Phone app at 390 x 844: every connection state the app can show. */
export async function captureApp(browser) {
  const shots = [];
  for (const c of CASES) {
    const file = `app-${c.name}-mobile.png`;
    const dir = buildApp(c.relay ?? VALID_RELAY);
    const tauri = { permission: 'granted', connect: 'ok', ...c.tauri };
    await withScene(browser, {
      viewport: 'mobile',
      origin: ORIGIN,
      dir,
      stubs: [[FAKE.supabaseUrl, EMPTY_JSON]],
      init: [...(c.session ? [sessionInit()] : []), [installTauri, tauri]],
    }, async (page) => {
      await page.goto(`${ORIGIN}/`);
      await drive(page, c);
      await shoot(page, join(OUT_DIR, file));
    });
    shots.push({ file, shows: c.desc });
  }
  return shots;
}
