import { test } from "node:test";
import assert from "node:assert/strict";
import { oldInstallIds } from "./old-install-detection.ts";
import type { Device } from "./device-list-store";

const NOW = Date.parse("2026-10-31T12:00:00Z");
const ago = (h: number) => new Date(NOW - h * 3600_000).toISOString();
const dev = (id: string, over: Partial<Device> = {}): Device => ({
  id, name: "iPhone", platform: "ios", app_kind: "app", created_at: ago(1000), last_seen_at: ago(0), revoked_at: null, ...over,
});
const ids = (devices: Device[], thisId = "me") => [...oldInstallIds(devices, thisId, NOW)].sort();

test("flags an older silent row of the same platform and install way", () => {
  assert.deepEqual(ids([dev("me"), dev("old", { last_seen_at: ago(72) })]), ["old"]);
});

test("never flags the current device", () => {
  assert.deepEqual(ids([dev("me", { last_seen_at: ago(72) }), dev("other", { last_seen_at: ago(0) })]), []);
});

test("a Browser row and a PWA row never mark each other", () => {
  const rows = [dev("me", { app_kind: "pwa" }), dev("tab", { app_kind: "browser", last_seen_at: ago(28) })];
  assert.deepEqual(ids(rows), []);
  // The owner's case: this device is unknown here, so stale PWA/Browser rows do not flag each other.
  assert.deepEqual(ids([dev("pwa", { app_kind: "pwa", last_seen_at: ago(48) }), dev("tab", { app_kind: "browser", last_seen_at: ago(28) })]), []);
});

test("other platform, recent rows and unknown kinds are not flagged", () => {
  assert.deepEqual(ids([dev("me"), dev("droid", { platform: "android", last_seen_at: ago(72) })]), []);
  assert.deepEqual(ids([dev("me"), dev("fresh", { last_seen_at: ago(2) })]), []);
  assert.deepEqual(ids([dev("me"), dev("legacy", { app_kind: null, last_seen_at: ago(72) })]), []);
  assert.deepEqual(ids([dev("me", { app_kind: null }), dev("old", { app_kind: null, last_seen_at: ago(72) })]), []);
});
