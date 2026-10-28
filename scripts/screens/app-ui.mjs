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

async function dragAllow(page, over) {
  const a = await page.locator('#allow').boundingBox();
  const r = await page.locator('#reject').boundingBox();
  await page.mouse.move(a.x + a.width / 2, a.y + a.height / 2);
  await page.mouse.down();
  const targetX = over ? r.x + r.width * 0.5 : (a.x + a.width / 2 + r.x) / 2;
  await page.mouse.move(a.x + a.width / 2 + 30, a.y + a.height / 2, { steps: 3 });
  await page.mouse.move(targetX, r.y + r.height / 2, { steps: 8 });
}

const history = (page, entries, always) =>
  page.evaluate(([e, al]) => { window.__ui.renderHistory(e, al, () => {}); window.__ui.setScreen('history'); }, [entries, always]);

export const UI_CASES = [
  { name: 'request', session: true, desc: 'An agent asks for the location. Two big buttons: Allow and Reject.', run: ask },
  { name: 'request-dragging', session: true, desc: 'Allow is being dragged. Reject turns into the dashed Always allow drop zone.', run: async (p) => { await ask(p); await dragAllow(p, false); } },
  { name: 'request-over', session: true, desc: 'Allow is over the drop zone. Releasing now always allows the agent.', run: async (p) => { await ask(p); await dragAllow(p, true); } },
  { name: 'history', session: true, desc: 'Request history with items and always allowed agents.', run: (p) => history(p, SEED, [AGENT, { clientId: 'c3', name: 'Cron agent' }]) },
  { name: 'history-empty', session: true, desc: 'Request history with no items.', run: (p) => history(p, [], []) },
];
