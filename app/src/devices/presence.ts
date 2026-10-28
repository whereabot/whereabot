/** A device is online when it refreshed last_seen_at within this window (heartbeat = 60 s). */
export const ONLINE_MS = 90_000;

type Seen = { last_seen_at: string };

export const isOnline = (d: Seen, now = Date.now()) => now - Date.parse(d.last_seen_at) < ONLINE_MS;

/** "Online", or "Last seen 5 min ago". */
export function seenText(d: Seen, now = Date.now()): string {
  if (isOnline(d, now)) return "Online";
  const min = Math.max(1, Math.round((now - Date.parse(d.last_seen_at)) / 60_000));
  if (min < 60) return `Last seen ${min} min ago`;
  const hours = Math.round(min / 60);
  if (hours < 48) return `Last seen ${hours} h ago`;
  return `Last seen ${Math.round(hours / 24)} days ago`;
}
