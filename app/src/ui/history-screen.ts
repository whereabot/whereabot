import type { Agent } from "../agent-identity";
import { $ } from "../status-display-dom";
import { clearHistory, listAlwaysApproved, listRequests, removeAlwaysApproved } from "../request-history-store";
import { announce, el, setScreen } from "./screen-view-helpers";

type Entries = ReturnType<typeof listRequests>;

const OUTCOME: Record<string, string> = {
  approved: "Allowed",
  always: "Always allowed",
  rejected: "Rejected",
  timeout: "No answer",
  auto: "Shared automatically",
  elsewhere: "Answered on another device",
};

const stamp = (at: number) =>
  new Date(at).toLocaleString([], { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });

/** Draws both lists. Data comes in as arguments so screenshots can seed it. */
export function renderHistory(requests: Entries, always: Agent[], onRemove: (agent: Agent) => void) {
  const list = $("history-list");
  list.replaceChildren(...[...requests].sort((a, b) => b.at - a.at).map((r) => {
    const row = el("li", "row");
    const time = el("time", "muted", stamp(r.at));
    time.setAttribute("datetime", new Date(r.at).toISOString());
    const badge = el("span", `badge badge-${r.outcome}`, OUTCOME[r.outcome] ?? r.outcome);
    row.append(el("span", "row-name", r.agent.name), badge, time);
    return row;
  }));
  $("history-empty").hidden = requests.length > 0;
  $("always-empty").hidden = always.length > 0;
  $("always-list").replaceChildren(...always.map((a) => {
    const row = el("li", "row");
    const rm = el("button", "btn btn-secondary btn-small", "Remove");
    rm.setAttribute("type", "button");
    rm.setAttribute("aria-label", `Remove ${a.name} from always allowed`);
    rm.addEventListener("click", () => onRemove(a));
    row.append(el("span", "row-name", a.name), rm);
    return row;
  }));
}

function refresh() {
  renderHistory(listRequests(), listAlwaysApproved(), (agent) => {
    removeAlwaysApproved(agent);
    refresh();
  });
}

/** Also opened by a tap on a "checked your location" notification. */
export function openHistory() {
  refresh();
  setScreen("history");
  $("history-title").focus();
}

export function initHistory() {
  $("open-history").addEventListener("click", openHistory);
  $("history-clear").addEventListener("click", () => {
    clearHistory();
    refresh();
    announce("History cleared");
    $("history-title").focus();
  });
  $("history-back").addEventListener("click", () => {
    setScreen("home");
    $("open-history").focus();
  });
}
