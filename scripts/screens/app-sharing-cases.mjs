// Cases for the Sharing screen and a shared request. Same shape as app-request-history-cases.mjs.
const MINE = [
  { id: 's1', grantee_email: 'maya.cohen@example.com', grantee_user_id: 'u2', owner_display_name: 'Ofek', created_at: '2026-10-01T10:00:00Z' },
  { id: 's2', grantee_email: 'a.very.long.address.for.wrapping@subdomain.example.org', grantee_user_id: null, owner_display_name: 'Ofek', created_at: '2026-10-02T10:00:00Z' },
];
const WITH_ME = [{ id: 's3', owner_email: 'dana@example.com', owner_display_name: 'Dana Levi', grantee_email: 'me@example.com', grantee_user_id: 'u1', created_at: '2026-10-03T10:00:00Z' }];

const show = (page, view) => page.evaluate((v) => { window.__ui.renderSharing(v); window.__ui.setScreen('sharing'); }, view);
const ready = (page, byMe, withMe) => show(page, { kind: 'ready', byMe, withMe });
const SHARED = { clientId: 'c1', name: "Dana's agent Claude", requesterId: 'u9' };

export const SHARING_CASES = [
  { name: 'sharing', session: true, desc: 'Sharing: email field, people I share with (one not signed in yet), people sharing with me.', run: (p) => ready(p, MINE, WITH_ME) },
  { name: 'sharing-empty', session: true, desc: 'Sharing with nobody and nobody sharing: both lists explain themselves.', run: (p) => ready(p, [], []) },
  { name: 'sharing-loading', session: true, desc: 'Sharing while the lists load.', run: (p) => show(p, { kind: 'loading' }) },
  { name: 'sharing-error', session: true, desc: 'Sharing lists could not load. Shows the reason and Try again.', run: (p) => show(p, { kind: 'error', message: 'Failed to fetch' }) },
  { name: 'sharing-invalid-email', session: true, desc: 'A malformed email is refused with a plain message and no network call.', run: async (p) => { await ready(p, MINE, []); await p.fill('#share-email', 'maya@'); await p.click('#share-submit'); await p.waitForFunction(() => document.getElementById('share-error').textContent !== ''); } },
  { name: 'sharing-stop', session: true, desc: 'Stop sharing asks for confirmation first.', run: async (p) => { await p.evaluate((m) => { window.__ui.renderSharing({ kind: 'ready', byMe: m, withMe: [] }, window.__ui.onStopSharing); window.__ui.setScreen('sharing'); }, MINE); await p.click('[data-id="s1"] button'); } },
  { name: 'shared-request', session: true, desc: "A shared request: \"Dana's agent Claude wants to know your location\" with Reject, Allow, Always allow.", run: (p) => p.evaluate((a) => { window.__ui.showRequest(a, new AbortController().signal); }, SHARED) },
];
