/** What one delivery needs; injected so tests run without a relay. */
export type DeliveryDeps = {
  /** One POST /v1/device/answer; resolves with the HTTP status, rejects on a network error. */
  post: (body: string) => Promise<number>;
  now: () => number;
  sleep: (ms: number) => Promise<void>;
  log: (msg: string) => void;
};

// 400/403/415/422: the relay will refuse this body every time, so retrying is pointless.
const permanent = (status: number) => [400, 403, 415, 422].includes(status);

/**
 * Posts one answer until the relay acknowledges it (204) or the request deadline passes.
 * 410 means no request waits under this id any more (answered elsewhere or timed out).
 * Returns true only when this answer settled the request.
 */
export async function deliverAnswer(id: string, body: string, deadline: number, d: DeliveryDeps): Promise<boolean> {
  for (let attempt = 1; ; attempt++) {
    d.log(`answer sent ${id} attempt=${attempt}`);
    const status = await d.post(body).catch((e: unknown) => String(e));
    if (status === 204) {
      d.log(`answer acked ${id}`);
      return true;
    }
    const wait = Math.min(4000, 250 * 2 ** attempt);
    if (status === 410 || (typeof status === "number" && permanent(status)) || d.now() + wait >= deadline) {
      d.log(`answer failed ${id} status=${status}`);
      return false;
    }
    await d.sleep(wait);
  }
}
