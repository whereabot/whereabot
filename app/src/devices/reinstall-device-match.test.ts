// Run: node --test app/src/devices/reinstall-device-match.test.ts
import assert from "node:assert/strict";
import test from "node:test";
import { type OwnDeviceRow, rowToTakeOver, screenHint } from "./reinstall-device-match.ts";

const hint = screenHint(390, 844, 3);
const me = { id: "new", platform: "ios", appKind: "pwa", screenHint: hint };
const row = (o: Partial<OwnDeviceRow>): OwnDeviceRow => ({
  id: "old", platform: "ios", app_kind: "pwa", screen_hint: hint,
  last_seen_at: "2026-10-29T10:00:00Z", revoked_at: null, ...o,
});

test("a reinstalled PWA takes over its old row", () => {
  assert.equal(rowToTakeOver([row({})], me), "old");
});

test("the newest of several matches wins", () => {
  const rows = [row({ id: "a" }), row({ id: "b", last_seen_at: "2026-10-29T12:00:00Z" })];
  assert.equal(rowToTakeOver(rows, me), "b");
});

test("an install that already has a row keeps it, even a removed one", () => {
  assert.equal(rowToTakeOver([row({}), row({ id: "new", revoked_at: "2026-10-29T11:00:00Z" })], me), null);
});

test("no match across install way, platform, removed rows or another screen", () => {
  assert.equal(rowToTakeOver([row({ app_kind: "app" })], me), null);
  assert.equal(rowToTakeOver([row({ app_kind: null })], me), null);
  assert.equal(rowToTakeOver([row({ platform: "android" })], me), null);
  assert.equal(rowToTakeOver([row({ revoked_at: "2026-10-29T11:00:00Z" })], me), null);
  assert.equal(rowToTakeOver([row({ screen_hint: screenHint(430, 932, 3) })], me), null);
});

test("rows from before hints existed still match", () => {
  assert.equal(rowToTakeOver([row({ screen_hint: null })], me), "old");
});

test("browser tabs never take over a row", () => {
  assert.equal(rowToTakeOver([row({ app_kind: "browser" })], { ...me, appKind: "browser" }), null);
});

test("hint ignores orientation", () => {
  assert.equal(screenHint(844, 390, 3), hint);
});
