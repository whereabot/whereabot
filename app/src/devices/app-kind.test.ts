// Run: node --test app/src/devices/app-kind.test.ts
import assert from "node:assert/strict";
import test from "node:test";
import { appKindLabel, detectAppKind, selectToleratingMissingKind } from "./app-kind.ts";

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

test("select retries without app_kind when the column is missing", async () => {
  const seen: string[] = [];
  const res = await selectToleratingMissingKind("id,app_kind,created_at", async (cols) => {
    seen.push(cols);
    return cols.includes("app_kind")
      ? { data: null, error: { code: "42703", message: "column devices.app_kind does not exist" } }
      : { data: [{ id: "d1" }], error: null };
  });
  assert.deepEqual(seen, ["id,app_kind,created_at", "id,created_at"]);
  assert.deepEqual(res.data, [{ id: "d1" }]);
});

test("select does not retry on other errors", async () => {
  let calls = 0;
  const res = await selectToleratingMissingKind("id,app_kind", async () => {
    calls++;
    return { data: null, error: { code: "42501", message: "denied" } };
  });
  assert.equal(calls, 1);
  assert.equal(res.error?.code, "42501");
});
