// Cases for the request and history screens. `run` drives the page after it is connected.
const AGENT = { clientId: 'c1', name: 'Claude on Ofek-laptop' };
const NOW = Date.now();
const SEED = [
  { at: NOW - 120e3, agent: AGENT, outcome: 'approved' },
  { at: NOW - 3600e3, agent: { clientId: 'c2', name: 'Hermes' }, outcome: 'rejected' },
  { at: NOW - 7200e3, agent: AGENT, outcome: 'always' },
  { at: NOW - 86400e3, agent: { clientId: 'c3', name: 'Cron agent' }, outcome: 'timeout' },
  { at: NOW - 90000e3, agent: { clientId: 'c3', name: 'Cron agent' }, outcome: 'auto' },
];

const ask = (page) => page.evaluate((a) => { window.__ui.showRequest(a, new AbortController().signal); }, AGENT);

const history = (page, entries, always) =>
  page.evaluate(([e, al]) => { window.__ui.renderHistory(e, al, () => {}); window.__ui.setScreen('history'); }, [entries, always]);

// Ages, not dates: the page clock is fixed, so the page turns them into timestamps.
const DEVICES = [
  { id: 'd1', name: 'iPhone', platform: 'ios', app_kind: 'app', created_at: 9e9, seen_ago: 20e3, revoked_at: null },
  { id: 'd2', name: 'Pixel 8 (work)', platform: 'android', app_kind: 'pwa', created_at: 8e9, seen_ago: 3 * 3600e3, revoked_at: null },
  { id: 'd3', name: 'Old Galaxy', platform: 'android', app_kind: 'browser', created_at: 7e9, seen_ago: 5 * 86400e3, revoked_at: null },
];
// d1 is "this device". The real actions are passed in, so Remove opens the real dialog.
const devices = (page, view) => page.evaluate((v) => {
  const at = (ms) => new Date(Date.now() - ms).toISOString();
  if (v.devices) v = { ...v, devices: v.devices.map((d) => ({ ...d, created_at: at(d.created_at), last_seen_at: at(d.seen_ago) })) };
  window.__ui.renderDevices(v, 'd1', window.__ui.deviceActions); window.__ui.setScreen('devices'); }, view);

// This iPhone (App), an older App install silent for 3 days (truly stale), and a Browser tab on the
// same iPhone silent for 28 h: a different install way, so it must not get the hint.
const TWINS = [
  DEVICES[0],
  { id: 'd4', name: 'iPhone', platform: 'ios', app_kind: 'app', created_at: 6e9, seen_ago: 3 * 86400e3, revoked_at: null },
  { id: 'd5', name: 'iPhone', platform: 'ios', app_kind: 'browser', created_at: 5e9, seen_ago: 28 * 3600e3, revoked_at: null },
];

export const UI_CASES = [
  { name: 'request', session: true, desc: 'An agent asks for the location. Three tiles: Reject, Allow, Always allow.', run: ask },
  { name: 'history', session: true, desc: 'Request history with items and always allowed agents.', run: (p) => history(p, SEED, [AGENT, { clientId: 'c3', name: 'Cron agent' }]) },
  { name: 'devices', session: true, desc: 'Devices page: this phone online, others with last seen, each tagged App, Web app (PWA) or Browser, a Computer coming soon row and the add-a-phone hint.', run: (p) => devices(p, { kind: 'ready', devices: DEVICES }) },
  { name: 'devices-old-install', session: true, desc: 'This iPhone at the top, an older App install of it with a one-line hint (no extra button), and a Browser row on the same iPhone without the hint.', run: (p) => devices(p, { kind: 'ready', devices: TWINS }) },
  { name: 'devices-empty', session: true, desc: 'Devices page with no devices yet.', run: (p) => devices(p, { kind: 'ready', devices: [] }) },
  { name: 'devices-rename', session: true, desc: 'Renaming a device inline: a name field with Save and Cancel.', run: async (p) => { await devices(p, { kind: 'ready', devices: DEVICES }); await p.click('[data-id="d2"] button:has-text("Rename")'); } },
  { name: 'devices-remove', session: true, desc: 'Removing another device asks for confirmation first.', run: async (p) => { await devices(p, { kind: 'ready', devices: DEVICES }); await p.click('[data-id="d3"] button:has-text("Remove")'); } },
  { name: 'devices-error', session: true, desc: 'Devices could not be loaded. Shows the reason and Try again.', run: (p) => devices(p, { kind: 'error', message: 'Failed to fetch' }) },
  { name: 'history-empty', session: true, desc: 'Request history with no items.', run: (p) => history(p, [], []) },
];

// Permission step flow. `wait` is the selector that shows when the state is ready.
export const PERM_CASES = [
  { session: true, name: 'perm-location', tauri: { permission: 'prompt', notifications: 'prompt' }, wait: '#perms', desc: 'First launch, step 1 of 2. Asks for location with one button.' },
  { session: true, name: 'perm-location-denied', tauri: { permission: 'denied', notifications: 'prompt' }, wait: '#perm-how', desc: 'Location refused. Numbered steps and an Open Settings button.' },
  { session: true, name: 'perm-notifications', tauri: { notifications: 'prompt' }, wait: '#perms', desc: 'Step 2 of 2. Asks for notifications. Skip for now is allowed.' },
  { session: true, name: 'perm-notifications-banner', tauri: { notifications: 'prompt' }, click: '#perm-skip', wait: '#notif-banner:not([hidden])', desc: 'Notifications skipped. Home shows a banner with a Fix button.' },
];
export { SHARING_CASES } from './app-sharing-cases.mjs';
