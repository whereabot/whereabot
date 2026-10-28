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

export const UI_CASES = [
  { name: 'request', session: true, desc: 'An agent asks for the location. Three tiles: Reject, Allow, Always allow.', run: ask },
  { name: 'history', session: true, desc: 'Request history with items and always allowed agents.', run: (p) => history(p, SEED, [AGENT, { clientId: 'c3', name: 'Cron agent' }]) },
  { name: 'history-empty', session: true, desc: 'Request history with no items.', run: (p) => history(p, [], []) },
];

// Permission step flow. `wait` is the selector that shows when the state is ready.
export const PERM_CASES = [
  { session: true, name: 'perm-location', tauri: { permission: 'prompt', notifications: 'prompt' }, wait: '#perms', desc: 'First launch, step 1 of 2. Asks for location with one button.' },
  { session: true, name: 'perm-location-denied', tauri: { permission: 'denied', notifications: 'prompt' }, wait: '#perm-how', desc: 'Location refused. Numbered steps and an Open Settings button.' },
  { session: true, name: 'perm-notifications', tauri: { notifications: 'prompt' }, wait: '#perms', desc: 'Step 2 of 2. Asks for notifications. Skip for now is allowed.' },
  { session: true, name: 'perm-notifications-banner', tauri: { notifications: 'prompt' }, click: '#perm-skip', wait: '#notif-banner:not([hidden])', desc: 'Notifications skipped. Home shows a banner with a Fix button.' },
];
