import { deliverAnswer } from "./answer-delivery-retry";
import { relayUrl } from "./build-config";
import { getAccessToken } from "./supabase-client";

const answerUrl = `${relayUrl.replace(/^ws/, "http")}/v1/device/answer`;

const sleep = async (ms: number): Promise<void> => {
  await new Promise<void>((resolve) => {
    globalThis.setTimeout(resolve, ms);
  });
};

/** One acknowledged POST; a fresh token each time, so supabase-js can refresh between tries. */
async function post(body: string): Promise<number> {
  const token = await getAccessToken();
  if (!token) throw new Error("signed out");
  const res = await fetch(answerUrl, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
    body,
    signal: AbortSignal.timeout(10_000),
  });
  return res.status;
}

/**
 * Sends an answer (fix, denied, unavailable) over HTTP, not the socket: a dying socket
 * dropped approved fixes with no error. Retries until acked or `deadline`; throws if not acked.
 */
export async function deliver(id: string, body: string, deadline: number, step: (m: string) => void) {
  const log = (m: string) => {
    console.warn(m); // shows in the Safari Web Inspector log for the phone
    step(m);
  };
  if (!(await deliverAnswer(id, body, deadline, { post, now: Date.now, sleep, log }))) {
    throw new Error("answer not delivered");
  }
}
