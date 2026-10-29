// Smoke test for the landing's "Get started" tabs: the default tab per device, keyboard switching,
// the detected install path with "Other devices", copy buttons, install help, the no-JS fallback,
// and that the section fits one screen (390x844 and 1280x800) in every state.
// Usage: node scripts/landing-install-smoke.mjs   (needs Playwright; see scripts/screens/playwright-browser.mjs)
// SHOTS=<dir> also saves one viewport PNG per case and state.
import assert from 'node:assert/strict';
import { join } from 'node:path';
import { ROOT, launch } from './screens/playwright-browser.mjs';
import { withScene } from './screens/static-site-scene.mjs';

// localhost is a secure context, so navigator.clipboard (and the Copy buttons) exist.
const ORIGIN = 'http://localhost:4321';
const DIR = join(ROOT, 'site');
const UA = {
  iphoneSafari: 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1',
  iphoneChrome: 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/130.0.0.0 Mobile/15E148 Safari/604.1',
  androidChrome: 'Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Mobile Safari/537.36',
  desktopChrome: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36',
};
// `path`: the install path shown first; `fit`: check the one-screen fit (the 320 px case only checks overflow).
const CASES = [
  { name: 'iphone-safari', viewport: 'mobile', ua: UA.iphoneSafari, tab: 'Device', path: 'App Store', help: 'Add to Home Screen', fit: true },
  { name: 'iphone-chrome', viewport: 'narrow', ua: UA.iphoneChrome, tab: 'Device', path: 'App Store', help: 'Open in Safari to install' },
  { name: 'android-chrome', viewport: 'mobile', ua: UA.androidChrome, tab: 'Device', path: 'Google Play', help: 'Install on Android', fit: true },
  { name: 'desktop-chrome', viewport: 'desktop', ua: UA.desktopChrome, tab: 'Agent', path: 'On your phone, open', help: 'Install Whereabot', fit: true },
];

async function checkFit(page, c, state) {
  const [section, screen, over] = await page.evaluate(() => [document.getElementById('start').getBoundingClientRect().height, innerHeight, document.documentElement.scrollWidth - innerWidth]);
  const height = Math.ceil(section - screen);
  console.log(`${c.name} ${state}: section ${Math.ceil(section)} of ${screen} px`);
  assert.equal(over, 0, `${c.name} ${state}: no sideways scroll`);
  if (c.fit) assert.ok(height <= 0, `${c.name} ${state}: section is ${height} px taller than the screen`);
  if (process.env.SHOTS) {
    await page.locator('#start').evaluate((s) => s.scrollIntoView({ behavior: 'instant' }));
    await page.screenshot({ path: join(process.env.SHOTS, `landing-${c.name}-${state}.png`) });
  }
}

const browser = await launch();
try {
  for (const c of CASES) {
    await withScene(browser, { viewport: c.viewport, origin: ORIGIN, dir: DIR, userAgent: c.ua }, async (page) => {
      await page.goto(`${ORIGIN}/`);
      const tab = (name) => page.getByRole('tab', { name });
      const agent = page.getByRole('tabpanel', { name: 'Agent' });
      const device = page.getByRole('tabpanel', { name: 'Device' });
      await tab(c.tab).waitFor();
      assert.equal(await tab(c.tab).getAttribute('aria-selected'), 'true', `${c.name}: default tab ${c.tab}`);
      assert.equal(await page.getByRole('tabpanel').count(), 1, `${c.name}: one panel visible`);
      await checkFit(page, c, c.tab.toLowerCase());

      // Keyboard: arrows move selection and focus between tabs.
      await tab(c.tab).focus();
      await page.keyboard.press(c.tab === 'Agent' ? 'ArrowRight' : 'ArrowLeft');
      const other = c.tab === 'Agent' ? 'Device' : 'Agent';
      assert.equal(await tab(other).getAttribute('aria-selected'), 'true', `${c.name}: arrow key selects ${other}`);
      assert.ok(await tab(other).evaluate((t) => t === document.activeElement), `${c.name}: focus follows`);
      await checkFit(page, c, other.toLowerCase());

      await tab('Agent').click();
      assert.equal(await agent.getByRole('button', { name: /^Copy / }).count(), 2, `${c.name}: copy buttons`);
      for (const text of ['Claude.ai and Claude Desktop', 'Other MCP clients', 'OAuth']) assert.ok(await agent.getByText(text).isVisible(), `${c.name}: ${text}`);
      assert.ok(await agent.getByRole('link', { name: 'More in the docs' }).isVisible(), `${c.name}: docs link`);

      await tab('Device').click();
      assert.ok(await device.getByText(c.path).isVisible(), `${c.name}: ${c.path} first`);
      assert.ok(await device.getByRole('link', { name: 'Open the browser app' }).isVisible(), `${c.name}: link to /app/`);
      assert.equal(await device.locator('.install-options > li:visible').count(), 2, `${c.name}: only this device's paths`);
      await device.getByText('How to install from the browser').click();
      await device.locator('[data-install-help]').getByText(c.help).first().waitFor();
      await checkFit(page, c, 'device-steps');
      const others = device.getByRole('button', { name: 'Other devices' });
      await others.click();
      assert.equal(await others.getAttribute('aria-expanded'), 'true', `${c.name}: other devices expanded`);
      assert.equal(await device.locator('.install-options > li:visible').count(), c.tab === 'Agent' ? 4 : 3, `${c.name}: every path after Other devices`);
      await checkFit(page, c, 'device-all');

      // In-page links open their tab.
      await page.locator('#how a[href="#agent"]').click();
      assert.ok(await agent.isVisible(), `${c.name}: #agent link opens the Agent tab`);
    });
  }

  // No JS: the tab list stays hidden and both panels show with all their content.
  await withScene(browser, { viewport: 'desktop', origin: ORIGIN, dir: DIR }, async (page) => {
    await page.route('**/*.js', (route) => route.abort());
    await page.goto(`${ORIGIN}/`);
    assert.ok(!(await page.locator('#start [role="tablist"]').isVisible()), 'no-js: tab list hidden');
    for (const id of ['#agent', '#get-app']) assert.ok(await page.locator(id).isVisible(), `no-js: ${id} visible`);
    assert.equal(await page.locator('#get-app .install-options > li:visible').count(), 4, 'no-js: every install path');
  });
  console.log(`Landing install smoke passed: ${CASES.map((c) => c.name).join(', ')}, no-js`);
} finally {
  await browser.close();
}
