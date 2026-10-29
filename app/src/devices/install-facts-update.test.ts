// Run: node --test app/src/devices/install-facts-update.test.ts
import assert from "node:assert/strict";
import test from "node:test";
import { sendInstallFacts } from "./install-facts-update.ts";

test("app_kind still saves when the screen_hint column is missing", async () => {
  const saved: Record<string, string>[] = [];
  await sendInstallFacts(async (fields) => {
    if ("screen_hint" in fields) throw Object.assign(new Error("column missing"), { code: "42703" });
    saved.push(fields);
  }, { app_kind: "pwa", screen_hint: "390x844@3" });
  assert.deepEqual(saved, [{ app_kind: "pwa" }]);
});
