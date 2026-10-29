// Landing: install steps for the browser app (/app/) that match this device.
// Inside /app/ the same help comes from app/src/ui/pwa-limits-notice.ts.
import { android, ios, iosSafari } from "./device-platform.js";

const SHARE_ICON = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M12 15V3m0 0L8 7m4-4 4 4M6 11H5v10h14V11h-1"/></svg>';
const ADD_ICON = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="4" y="4" width="16" height="16" rx="3"/><path d="M12 8v8m-4-4h8"/></svg>';

function banner(title, body) {
  const aside = Object.assign(document.createElement("aside"), { className: "banner" });
  aside.setAttribute("data-install-help", "");
  const heading = Object.assign(document.createElement("p"), { className: "banner-title", textContent: title });
  aside.append(heading);
  aside.insertAdjacentHTML("beforeend", body); // static markup only, no user data
  return aside;
}

function help() {
  if (iosSafari) {
    return banner(
      "Install on iPhone or iPad",
      `<ol class="install-steps">
        <li>Open <a href="app/">whereabot.io/app</a> in Safari.</li>
        <li>Tap ${SHARE_ICON} Share.</li>
        <li>Tap ${ADD_ICON} Add to Home Screen, then Add.</li>
      </ol><p>Notifications need iOS 16.4 or later.</p>`,
    );
  }
  if (ios) {
    return banner("Open in Safari to install", "<p>Open <strong>whereabot.io/app</strong> in Safari, then tap Share and Add to Home Screen.</p>");
  }
  if (android) {
    return banner(
      "Install on Android",
      `<ol class="install-steps">
        <li>Open <a href="app/">whereabot.io/app</a> in Chrome.</li>
        <li>Tap Install Whereabot, or open the browser menu and tap Install app.</li>
      </ol>`,
    );
  }
  return banner(
    "Install Whereabot",
    `<p>In Chrome or Edge, open the browser app and tap Install Whereabot.</p>
     <a class="btn btn-secondary" href="app/">Install Whereabot</a>`,
  );
}

document.getElementById("install-help")?.append(help());
