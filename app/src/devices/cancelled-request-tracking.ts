// Locate requests another device already answered. The relay sends {"type":"cancel","id"}.
const open = new Map<string, AbortController>();
const cancelled = new Set<string>();

/** Lets a cancel close the prompt for this request. Call `untrack` when it ends. */
export const track = (id: string, abort: AbortController) => void open.set(id, abort);
export const untrack = (id: string) => void open.delete(id);

/** True if the relay cancelled this request (also while it still waits in the queue). */
export const wasCancelled = (id: string) => cancelled.has(id);

export function cancelRequest(id: string) {
  cancelled.add(id);
  open.get(id)?.abort();
}
