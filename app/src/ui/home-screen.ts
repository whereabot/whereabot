import { $ } from "../status-display-dom";
import { relayUrl } from "../build-config";
import { supabase } from "../supabase-client";
import { announce } from "./screen-view-helpers";

const ONBOARDED = "whereabouts.onboarded";
const mcpUrl = `${relayUrl.replace(/^ws/, "http")}/mcp`;

async function showAccount() {
  const { data } = await supabase.auth.getSession();
  $("account").textContent = data.session?.user.email ?? "";
  $("onboarding").hidden = !data.session || localStorage.getItem(ONBOARDED) === "1";
}

async function copy(text: string, done: string) {
  try {
    await navigator.clipboard.writeText(text);
    announce(done);
  } catch {
    announce("Copy failed. Select the text and copy it by hand.");
  }
}

function initOnboarding() {
  $("copy-prompt").addEventListener("click", () => void copy($("first-prompt").textContent ?? "", "Question copied"));
  $("onb-done").addEventListener("click", () => ($("onboarding").hidden = true));
}

/** The first answered request ends onboarding: the card says it worked, and Done hides it. */
export function showFirstAnswer() {
  if ($("onboarding").hidden) return;
  localStorage.setItem(ONBOARDED, "1");
  $("onb-title").textContent = "It works";
  $("onb-works").hidden = $("onb-done").hidden = false;
  $("onb-steps").hidden = $("copy-prompt").hidden = true;
  announce("It works. Your agent asked and you answered.");
}

function initHelp() {
  $("cmd-add").textContent = `claude mcp add --transport http whereabot ${mcpUrl}`;
  $("copy-url").addEventListener("click", () => void copy(mcpUrl, "MCP URL copied"));
}

/** Android back button: history goes home, the request screen stays. Uses history entries, which the webview maps to back. */
function initBackButton() {
  const screen = () => document.body.dataset.screen;
  const pushed = () => history.state?.screen as string | undefined;
  new MutationObserver(() => {
    if (screen() !== "home" && !pushed()) history.pushState({ screen: screen() }, "");
    else if (screen() === "home" && pushed()) history.back();
  }).observe(document.body, { attributes: true, attributeFilter: ["data-screen"] });
  window.addEventListener("popstate", () => {
    if (screen() === "request") history.pushState({ screen: "request" }, "");
    else if (["history", "devices", "sharing"].includes(screen() ?? "")) {
      const opener = `open-${screen()}`;
      document.body.dataset.screen = "home";
      $(opener).focus();
    }
  });
}

export function initHome() {
  initOnboarding();
  initHelp();
  initBackButton();
  supabase.auth.onAuthStateChange(() => setTimeout(() => void showAccount(), 0));
  void showAccount();
}
