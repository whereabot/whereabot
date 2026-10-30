// Run: node --test app/src/shell-kind.test.ts
import assert from "node:assert/strict";
import test from "node:test";
import { shellKind } from "./shell-kind.ts";

const mac = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15";

test("phones and iPads take the plugin path", () => {
  assert.equal(shellKind(true, "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X)", 5), "mobile");
  assert.equal(shellKind(true, "Mozilla/5.0 (Linux; Android 15; Pixel 9)", 5), "mobile");
  assert.equal(shellKind(true, mac, 5), "mobile"); // iPadOS with a Mac user agent
});

test("a Mac uses CoreLocation, other desktops the webview", () => {
  assert.equal(shellKind(true, mac, 0), "mac");
  assert.equal(shellKind(true, "Mozilla/5.0 (Windows NT 10.0; Win64; x64)", 0), "desktop");
  assert.equal(shellKind(true, "Mozilla/5.0 (X11; Linux x86_64)", 0), "desktop");
});

test("the browser build is never a shell", () => {
  assert.equal(shellKind(false, mac, 0), "web");
});
