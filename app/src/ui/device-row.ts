import { isOnline, seenText } from "../devices/device-online-status";
import type { Device } from "../devices/device-list-store";
import { appKindLabel } from "../devices/app-kind";
import { el } from "./screen-view-helpers";

export type RowActions = {
  onRename: (d: Device, name: string) => void;
  onRemove: (d: Device) => void;
};

// Stroke icons (24 px grid). The platform is also written as text, so the icon is decoration.
const PHONE = '<rect x="7" y="2" width="10" height="20" rx="2"/><path d="M11 18h2"/>';
const COMPUTER = '<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/>';
const PLATFORM = { ios: "iPhone", android: "Android", desktop: "Computer" };

function button(label: string, aria: string, onClick: () => void, kind = "secondary") {
  const b = el("button", `btn btn-${kind} btn-small`, label);
  b.setAttribute("type", "button");
  b.setAttribute("aria-label", aria);
  b.addEventListener("click", onClick);
  return b;
}

/** Swaps the title for a name field. Enter saves, Escape or Cancel restores. */
function edit(title: HTMLElement, actions: HTMLElement, d: Device, save: RowActions["onRename"]) {
  const form = el("form", "field-inline");
  const input = el("input", "") as HTMLInputElement;
  input.id = `rename-${d.id}`;
  input.value = d.name;
  input.maxLength = 64;
  input.required = true;
  const label = el("label", "sr-only", `Name for ${d.name}`) as HTMLLabelElement;
  label.htmlFor = input.id;
  const stop = () => { form.replaceWith(title); actions.hidden = false; };
  form.addEventListener("keydown", (e) => e.key === "Escape" && stop());
  form.addEventListener("submit", (e) => {
    e.preventDefault();
    if (input.value.trim()) save(d, input.value.trim());
  });
  const ok = el("button", "btn btn-primary btn-small", "Save");
  const cancel = button("Cancel", `Cancel renaming ${d.name}`, stop, "ghost");
  form.append(label, input, ok, cancel);
  title.replaceWith(form);
  actions.hidden = true;
  input.select();
  input.focus();
}

export function deviceRow(d: Device, isThis: boolean, act: RowActions, oldInstall = false) {
  const row = el("li", "row device");
  row.dataset.id = d.id;
  const icon = el("span", "device-icon");
  icon.setAttribute("aria-hidden", "true");
  icon.innerHTML = `<svg viewBox="0 0 24 24">${d.platform === "desktop" ? COMPUTER : PHONE}</svg>`;
  const title = el("div", "device-title");
  title.append(el("span", "row-name", d.name), el("span", "badge", appKindLabel(d.app_kind)));
  if (isThis) title.append(el("span", "badge is-accent", "This device"));
  const meta = el("span", `device-meta${isOnline(d) ? " status-dot is-success" : ""}`, `${PLATFORM[d.platform]} · ${seenText(d)}`);
  const actions = el("div", "row-actions");
  actions.append(
    button("Rename", `Rename ${d.name}`, () => edit(title, actions, d, act.onRename)),
    button("Remove", `Remove ${d.name}`, () => act.onRemove(d)),
  );
  const main = el("div", "device-main");
  main.append(title, meta);
  // One line, no button: the row's own Remove is the action.
  if (oldInstall) main.append(el("span", "device-meta", "Older install of this device."));
  row.append(icon, main, actions);
  return row;
}

/** Placeholder for the future computer client. Not a device: no actions. */
export function comingSoonRow() {
  const row = el("li", "row device row-muted");
  const icon = el("span", "device-icon");
  icon.setAttribute("aria-hidden", "true");
  icon.innerHTML = `<svg viewBox="0 0 24 24">${COMPUTER}</svg>`;
  const main = el("div", "device-main");
  main.append(el("span", "row-name", "Computer"), el("span", "device-meta", "Coming soon"));
  row.append(icon, main);
  return row;
}
