import { inTauri } from "../tauri-runtime";
import { $ } from "../status-display-dom";
import { installButton } from "./pwa-install-button";

// shortcut: no App Store listing yet, so this opens the landing's install section. Swap in the apps.apple.com URL when the app is live.
const APP_STORE_URL = "https://whereabot.io/#get-app";
const LIMIT = "In the browser version, Whereabot answers only after you tap the notification and open the app.";
const ALWAYS = "Always allow cannot answer in the background.";
const ua = navigator.userAgent;
// iPadOS 13+ reports a Mac user agent; touch points tell it apart.
const ios = /iPhone|iPad|iPod/.test(ua) || (/Macintosh/.test(ua) && navigator.maxTouchPoints > 1);
const iosSafari = ios && !/CriOS|FxiOS|EdgiOS|OPiOS|GSA\//.test(ua);
const standalone = matchMedia("(display-mode: standalone)").matches || (navigator as { standalone?: boolean }).standalone === true;

const el = <K extends keyof HTMLElementTagNameMap>(tag: K, props: Partial<HTMLElementTagNameMap[K]> = {}, ...kids: Node[]) => {
  const node = Object.assign(document.createElement(tag), props);
  node.append(...kids);
  return node;
};

const note = (text: string) => el("p", { className: "muted pwa-limit-note", textContent: text });

/** The full notice: what the browser version cannot do, and where to get the app that can. */
function limitsBanner() {
  const link = el("a", { href: APP_STORE_URL, textContent: "Get the App Store app", target: "_blank", rel: "noopener" });
  return el(
    "aside",
    { className: "banner pwa-limits" },
    el("p", { className: "banner-title", textContent: "Browser version" }),
    el("p", { textContent: LIMIT }),
    el("p", { textContent: `${ALWAYS} For automatic answers, install the App Store app.` }),
    link,
  );
}

/** Not installed yet. iPhone Safari: Home Screen steps (Web Push needs them, iOS 16.4+). Other iOS browsers: go to Safari. Elsewhere: the install button. */
function installBanner() {
  if (!ios) return installButton();
  if (!iosSafari) {
    return el(
      "aside",
      { className: "banner pwa-install" },
      el("p", { className: "banner-title", textContent: "Open in Safari to install" }),
      el("p", { textContent: "Open whereabot.io/app in Safari, then tap Share and Add to Home Screen." }),
    );
  }
  const steps = ["Tap Share.", "Tap Add to Home Screen, then tap Add.", "Open Whereabot from the Home Screen."];
  return el(
    "aside",
    { className: "banner pwa-install" },
    el("p", { className: "banner-title", textContent: "Add to Home Screen to get notifications (iOS 16.4+)" }),
    el("ol", {}, ...steps.map((t) => el("li", { textContent: t }))),
  );
}

/** Browser build only: shows the limits on sign-in, Home, the permission steps and the Always allow flow. */
export function initPwaLimitsNotice() {
  if (inTauri) return;
  if (!standalone) $("signed-out").prepend(installBanner());
  $("signed-out").append(limitsBanner());
  if (!standalone) $("notif-banner").before(installBanner());
  $("notif-banner").before(limitsBanner());
  $("perm-why").after(note(LIMIT));
  $("always-allow").parentElement?.after(note(`${ALWAYS} Open the app to answer.`));
  $("always-empty").after(note(`${ALWAYS} Open the app to answer.`));
}
