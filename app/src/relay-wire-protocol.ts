import type { Position } from "@tauri-apps/plugin-geolocation";
import type { Requester } from "./agent-identity";

/** Server -> device. See docs/protocol.md. */
export type Ping = { type: "locate"; id: string; client_id?: string; expires_ms?: number; requester?: Requester | null };

/** Device -> server. */
export type LocationReply = {
  type: "location";
  id: string;
  latitude: number;
  longitude: number;
  accuracy_m: number;
  captured_at_ms: number;
};

export function answer(ping: Ping, pos: Position): LocationReply {
  return {
    type: "location",
    id: ping.id,
    latitude: pos.coords.latitude,
    longitude: pos.coords.longitude,
    accuracy_m: pos.coords.accuracy,
    captured_at_ms: Math.round(pos.timestamp),
  };
}

export const denied = (ping: Ping, reason: "user" | "no_answer") =>
  JSON.stringify({ type: "denied", id: ping.id, reason });

export const unavailable = (ping: Ping, reason: "permission" | "no_fix" | "error") =>
  JSON.stringify({ type: "unavailable", id: ping.id, reason });

/** Step report for relay logs while debugging. Never include location data. */
export const log = (msg: string) => JSON.stringify({ type: "log", msg });
