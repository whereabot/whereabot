// Install on your device: show only this device's install path first; "Other devices" reveals the rest.
// Without JS every path shows.
import { platform } from "./device-platform.js";

const panel = document.getElementById("get-app");
const items = [...(panel?.querySelectorAll("[data-platform]") ?? [])];
const mine = (item) => item.dataset.platform.split(" ").includes(platform);
// The computer-only hint ("On your phone, open ...") means nothing on a phone, so it stays hidden there.
const others = items.filter((item) => !mine(item) && item.dataset.platform !== "desktop");
const button = panel?.querySelector(".other-devices");

if (button) {
  for (const item of items) item.hidden = !mine(item);
  button.hidden = false;
  button.addEventListener("click", () => {
    const open = button.getAttribute("aria-expanded") !== "true";
    for (const item of others) item.hidden = !open;
    button.setAttribute("aria-expanded", String(open));
  });
}
