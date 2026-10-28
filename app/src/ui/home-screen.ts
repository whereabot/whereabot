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

function initOnboarding() {
  $("onb-done").addEventListener("click", () => {
    localStorage.setItem(ONBOARDED, "1");
    $("onboarding").hidden = true;
  });
}

function initHelp() {
  $("cmd-add").textContent = `claude mcp add --transport http whereabot ${mcpUrl}`;
  $("copy-url").addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(mcpUrl);
      announce("MCP URL copied");
    } catch {
      announce("Copy failed. Select the command and copy it by hand.");
    }
  });
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
    else if (screen() === "history" || screen() === "devices") {
      const opener = screen() === "history" ? "open-history" : "open-devices";
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
