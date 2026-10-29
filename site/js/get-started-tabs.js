// Get started: "Agent | Device" tabs. The default tab follows the device: phones see Device, computers see Agent.
// Without JS the tab list stays hidden and both panels show.
import { phoneLike } from "./device-platform.js";

const list = document.querySelector('#start [role="tablist"]');
const tabs = [...(list?.querySelectorAll('[role="tab"]') ?? [])];
const panelOf = (tab) => document.getElementById(tab.getAttribute("aria-controls"));

function select(tab, focus = false) {
  for (const t of tabs) {
    const on = t === tab;
    t.setAttribute("aria-selected", String(on));
    t.tabIndex = on ? 0 : -1;
    panelOf(t).hidden = !on;
  }
  if (focus) tab.focus();
}

if (tabs.length) {
  for (const [i, tab] of tabs.entries()) {
    const panel = panelOf(tab);
    panel.setAttribute("role", "tabpanel");
    panel.setAttribute("aria-labelledby", tab.id);
    panel.tabIndex = 0;
    tab.addEventListener("click", () => select(tab));
    tab.addEventListener("keydown", (event) => {
      const next = { ArrowRight: i + 1, ArrowLeft: i - 1, Home: 0, End: tabs.length - 1 }[event.key];
      if (next === undefined) return;
      event.preventDefault();
      select(tabs[(next + tabs.length) % tabs.length], true);
    });
  }
  // In-page links (#agent, #get-app) open their tab first; the browser then scrolls to it.
  const tabFor = (hash) => tabs.find((t) => `#${t.getAttribute("aria-controls")}` === hash);
  document.addEventListener("click", (event) => {
    const tab = tabFor(event.target.closest?.('a[href^="#"]')?.hash);
    if (tab) select(tab);
  });
  select(tabFor(location.hash) ?? tabs[phoneLike ? 1 : 0]);
  list.hidden = false;
}
