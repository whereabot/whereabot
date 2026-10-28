import { $ } from "../status-display-dom";
import { deviceId } from "../devices/device-identity";
import { signOutRemoved } from "../devices/removed-device-signout";
import { listDevices, removeDevice, renameDevice, type Device } from "../devices/device-list-store";
import { oldInstallIds } from "../devices/old-install-detection";
import { confirmRemove } from "./remove-device-confirm-dialog";
import { comingSoonRow, deviceRow, type RowActions } from "./device-row";
import { setScreen } from "./screen-view-helpers";

export type DevicesView =
  | { kind: "loading" }
  | { kind: "error"; message: string }
  | { kind: "ready"; devices: Device[] };

const say = (text: string) => ($("devices-status").textContent = text);
const why = (e: unknown) => (e instanceof Error ? e.message : String(e));

/** Draws one state of the page. Data comes in as arguments so screenshots can seed it. */
export function renderDevices(view: DevicesView, thisId: string, actions: RowActions) {
  const devices = view.kind === "ready" ? view.devices : [];
  const sorted = [...devices].sort((a, b) => Number(b.id === thisId) - Number(a.id === thisId));
  const old = oldInstallIds(devices);
  $("devices-list").replaceChildren(...sorted.map((d) => deviceRow(d, d.id === thisId, actions, old.has(d.id))), comingSoonRow());
  $("devices-empty").hidden = view.kind !== "ready" || devices.length > 0;
  $("devices-retry").hidden = view.kind !== "error";
  say(view.kind === "loading" ? "Loading devices…" : view.kind === "error" ? `Could not load devices: ${view.message}` : "");
}

async function refresh(quiet = false) {
  if (!quiet) renderDevices({ kind: "loading" }, deviceId(), deviceActions);
  try {
    renderDevices({ kind: "ready", devices: await listDevices() }, deviceId(), deviceActions);
  } catch (e) {
    renderDevices({ kind: "error", message: why(e) }, deviceId(), deviceActions);
  }
}

export const deviceActions: RowActions = {
  async onRename(d, name) {
    try {
      await renameDevice(d.id, name);
      await refresh(true);
      say(`Renamed to ${name}.`);
      document.querySelector<HTMLElement>(`[data-id="${d.id}"] .btn`)?.focus();
    } catch (e) {
      say(`Rename failed: ${why(e)}`);
    }
  },
  async onRemove(d) {
    const isThis = d.id === deviceId();
    if (!(await confirmRemove(d.name, isThis))) return;
    try {
      await removeDevice(d.id);
    } catch (e) {
      return say(`Remove failed: ${why(e)}`);
    }
    if (isThis) return void signOutRemoved();
    await refresh(true);
    say(`${d.name} removed.`);
    $("devices-title").focus();
  },
};

export function initDevices() {
  $("open-devices").addEventListener("click", () => {
    void refresh();
    setScreen("devices");
    $("devices-title").focus();
  });
  $("devices-back").addEventListener("click", () => {
    setScreen("home");
    $("open-devices").focus();
  });
  $("devices-retry").addEventListener("click", () => void refresh());
  // Keeps "Online" and "Last seen" current while the page stays open.
  window.setInterval(() => document.body.dataset.screen === "devices" && !$("devices-list").querySelector("input") && void refresh(true), 30_000);
}
