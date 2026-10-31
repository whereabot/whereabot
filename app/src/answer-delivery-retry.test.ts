// Run: node --test app/src/answer-delivery-retry.test.ts
import assert from "node:assert/strict";
import test from "node:test";
import { deliverAnswer, type DeliveryDeps } from "./answer-delivery-retry.ts";

/** Fake clock: sleep advances time; post replays `replies` (a string = network error). */
function fake(replies: Array<number | string>) {
  let t = 0;
  const logs: string[] = [];
  const posts: string[] = [];
  const deps: DeliveryDeps = {
    post: async (body) => {
      posts.push(body);
      const r = replies.shift() ?? 503;
      if (typeof r === "string") throw new TypeError(r);
      return r;
    },
    now: () => t,
    sleep: async (ms) => void (t += ms),
    log: (m) => logs.push(m),
  };
  return { deps, logs, posts, clock: () => t };
}

test("acked on first try", async () => {
  const f = fake([204]);
  assert.equal(await deliverAnswer("r1", "{}", 10_000, f.deps), true);
  assert.deepEqual(f.logs, ["answer sent r1 attempt=1", "answer acked r1"]);
});

test("retries network errors and 5xx until acked", async () => {
  const f = fake(["Load failed", 503, 204]);
  assert.equal(await deliverAnswer("r2", "body", 10_000, f.deps), true);
  assert.deepEqual(f.posts, ["body", "body", "body"]);
  assert.equal(f.logs.at(-1), "answer acked r2");
});

test("401 retries: the next post reads a refreshed token", async () => {
  const f = fake([401, 204]);
  assert.equal(await deliverAnswer("r3", "{}", 10_000, f.deps), true);
});

test("gives up at the deadline", async () => {
  const f = fake([]);
  assert.equal(await deliverAnswer("r4", "{}", 3_000, f.deps), false);
  assert.ok(f.clock() < 3_000);
  assert.match(f.logs.at(-1)!, /^answer failed r4 status=503$/);
});

test("410 and permanent 4xx stop at once", async () => {
  for (const s of [410, 400, 403, 422]) {
    const f = fake([s]);
    assert.equal(await deliverAnswer("r5", "{}", 60_000, f.deps), false);
    assert.equal(f.posts.length, 1);
    assert.equal(f.logs.at(-1), `answer failed r5 status=${s}`);
  }
});

test("410 after a lost reply counts as delivered", async () => {
  for (const lost of ["Load failed", 502]) {
    const f = fake([lost, 410]);
    assert.equal(await deliverAnswer("r6", "{}", 60_000, f.deps), true);
    assert.match(f.logs.at(-1)!, /^answer probably acked r6 status=410/);
  }
});

test("410 after a lost reply then a refused try counts as delivered", async () => {
  const f = fake([502, 401, 410]);
  assert.equal(await deliverAnswer("r8", "{}", 60_000, f.deps), true);
});

test("410 after a refused try still fails", async () => {
  const f = fake([401, 410]);
  assert.equal(await deliverAnswer("r7", "{}", 60_000, f.deps), false);
});
