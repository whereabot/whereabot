import { backgroundAnswerSupported, takeAutoShared } from "./background-answer-bridge";
import { recordRequest } from "./request-history-store";
import { openHistory } from "./ui/history-screen";

/** Adds the native handler's automatic answers to History; opens History after a "checked" tap. */
async function collect() {
  const { shared, openHistory: open } = await takeAutoShared();
  for (const s of shared) recordRequest({ clientId: s.clientId, name: s.name }, "auto", s.at);
  if (open) openHistory();
}

/** Call once at start, after initHistory. */
export function initAutoSharedHistory() {
  if (!backgroundAnswerSupported) return;
  void collect();
  document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && void collect());
}
