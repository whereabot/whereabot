import { helloMessage } from "./identity";
import { startHeartbeat, stopHeartbeat } from "./heartbeat";
import { signOutRemoved } from "./removed";

export { stopHeartbeat };

/** A socket just opened: name this device to the relay and start the presence heartbeat. */
export function onOpen(socket: { send: (message: string) => Promise<void> }) {
  void socket.send(helloMessage()).catch(() => {});
  startHeartbeat();
}

/** A Close frame arrived. True when it was "revoked": this device is signed out, no retry. */
export function onCloseFrame(code?: number, reason?: string): boolean {
  stopHeartbeat();
  if (code !== 4000 || reason !== "revoked") return false;
  void signOutRemoved();
  return true;
}
