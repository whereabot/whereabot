/** Reconnect delay: 1 s doubling to a 30 s cap, with jitter so phones do not retry in lockstep. */
export function backoffMs(attempt: number): number {
  return Math.min(30_000, 1000 * 2 ** attempt) * (0.5 + Math.random() / 2);
}
