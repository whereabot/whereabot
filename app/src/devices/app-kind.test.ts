// Run: node --test app/src/devices/app-kind.test.ts
import assert from "node:assert/strict";
import test from "node:test";
import { appKindLabel, detectAppKind } from "./app-kind.ts";

test("native shell wins, then home-screen install, then browser", () => {
  assert.equal(detectAppKind(true, false), "app");
  assert.equal(detectAppKind(true, true), "app");
  assert.equal(detectAppKind(false, true), "pwa");
  assert.equal(detectAppKind(false, false), "browser");
});

test("labels", () => {
  assert.equal(appKindLabel("app"), "App");
  assert.equal(appKindLabel("pwa"), "Web app (PWA)");
  assert.equal(appKindLabel("browser"), "Browser");
  assert.equal(appKindLabel(null), "Unknown");
});
