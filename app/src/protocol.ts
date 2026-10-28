import type { Position } from "@tauri-apps/plugin-geolocation";

/** Server -> device. See docs/protocol.md. */
export type Ping = { type: "locate"; id: string; client_id: string };

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
