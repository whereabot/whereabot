import type { Share } from "../sharing/location-shares-store";
import { el } from "./screen-view-helpers";

/** A person I share with: email, whether they joined yet, and Stop sharing. */
export function sharedByMeRow(s: Share, onStop: (s: Share) => void) {
  const row = el("li", "row");
  row.dataset.id = s.id;
  const stop = el("button", "btn btn-secondary btn-small", "Stop sharing");
  stop.setAttribute("type", "button");
  stop.setAttribute("aria-label", `Stop sharing with ${s.grantee_email}`);
  stop.addEventListener("click", () => onStop(s));
  row.append(
    el("span", "row-name", s.grantee_email),
    el("span", "row-meta", s.grantee_user_id ? "Their agents can ask where you are." : "Waiting for them to sign in to Whereabot."),
    stop,
  );
  return row;
}

/** Someone who shares with me. Read-only: only they can stop it. */
export function sharedWithMeRow(s: Share) {
  const row = el("li", "row");
  row.append(el("span", "row-name", s.owner_display_name || "Someone"), el("span", "row-meta", `${s.owner_email ? `${s.owner_email}. ` : ""}Your agent can ask where they are.`));
  return row;
}
