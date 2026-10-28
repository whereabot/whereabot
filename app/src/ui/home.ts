import { openUrl } from "@tauri-apps/plugin-opener";
import { checkPermissions } from "@tauri-apps/plugin-geolocation";
import { $ } from "../dom";
import { relayUrl } from "../config";
import { requestPermissionsUpFront } from "../location";
import { supabase } from "../supabase";
import { announce } from "./view";

const ONBOARDED = "whereabouts.onboarded";
const mcpUrl = `${relayUrl.replace(/^ws/, "http")}/mcp`;

/** Home card for denied location. Re-checked when the user comes back from Settings. */
async function showPermissionCard() {
  try {
    $("perm-card").hidden = (await checkPermissions()).location !== "denied";
  } catch {
    $("perm-card").hidden = true;
  }
}

async function showAccount() {
  const { data } = await supabase.auth.getSession();
  $("account").textContent = data.session?.user.email ?? "";
  $("onboarding").hidden = !data.session || localStorage.getItem(ONBOARDED) === "1";
}

function initOnboarding() {
  $("allow-perms").addEventListener("click", async () => {
    const btn = $<HTMLButtonElement>("allow-perms");
    btn.disabled = true;
    try {
      const r = await requestPermissionsUpFront();
      announce(r.location === "granted" ? "Location allowed" : "Location not allowed");
    } finally {
      btn.disabled = false;
      void showPermissionCard();
    }
  });
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
  $("open-settings").addEventListener("click", () => void openUrl("app-settings:"));
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
    else if (screen() === "history") {
      document.body.dataset.screen = "home";
      $("open-history").focus();
    }
  });
}

export function initHome() {
  initOnboarding();
  initHelp();
  initBackButton();
  supabase.auth.onAuthStateChange(() => setTimeout(() => void showAccount(), 0));
  void showAccount();
  void showPermissionCard();
  document.addEventListener("visibilitychange", () => document.visibilityState === "visible" && void showPermissionCard());
}
