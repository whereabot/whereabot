// Chrome and Edge (Android, desktop) offer install through beforeinstallprompt. Keep the event and show our own button.
type InstallPromptEvent = Event & { prompt(): Promise<unknown> };

let deferred: InstallPromptEvent | undefined;
const buttons: HTMLButtonElement[] = [];
const show = (visible: boolean) => buttons.forEach((b) => (b.hidden = !visible));

// Registered at import (app start), before the browser fires the event.
addEventListener("beforeinstallprompt", (e) => {
  e.preventDefault();
  deferred = e as InstallPromptEvent;
  show(true);
});
addEventListener("appinstalled", () => {
  deferred = undefined;
  show(false);
});

/** An "Install Whereabot" button, hidden until the browser allows install. */
export function installButton() {
  const button = Object.assign(document.createElement("button"), {
    type: "button",
    className: "btn btn-primary",
    textContent: "Install Whereabot",
    hidden: !deferred,
  });
  button.addEventListener("click", async () => {
    const event = deferred;
    deferred = undefined; // prompt() works once per event
    show(false);
    await event?.prompt();
  });
  buttons.push(button);
  return button;
}
