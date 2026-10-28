import { el } from "./screen-view-helpers";

/** Built on first use. Native <dialog> gives the focus trap and Escape. */
function build() {
  const dialog = el("dialog", "") as HTMLDialogElement;
  dialog.setAttribute("aria-labelledby", "stop-title");
  dialog.setAttribute("aria-describedby", "stop-text");
  const form = el("form", "stack");
  form.setAttribute("method", "dialog");
  const actions = el("div", "actions");
  const cancel = el("button", "btn btn-secondary", "Cancel") as HTMLButtonElement;
  cancel.value = "cancel";
  cancel.autofocus = true; // the safe choice
  const stop = el("button", "btn btn-danger", "Stop sharing") as HTMLButtonElement;
  stop.value = "stop";
  actions.append(cancel, stop);
  const title = el("h2", "");
  title.id = "stop-title";
  const text = el("p", "", "Their agents can no longer ask where you are. You can share again any time.");
  text.id = "stop-text";
  form.append(title, text, actions);
  dialog.append(form);
  document.body.append(dialog);
  return { dialog, title };
}

let parts: ReturnType<typeof build> | undefined;

/** Modal "Stop sharing?" dialog. Resolves true on Stop sharing, false on Cancel or Escape. */
export function confirmStopSharing(email: string): Promise<boolean> {
  const { dialog, title } = (parts ??= build());
  title.textContent = `Stop sharing with ${email}?`;
  return new Promise((resolve) => {
    dialog.addEventListener("close", () => resolve(dialog.returnValue === "stop"), { once: true });
    dialog.returnValue = "";
    dialog.showModal();
  });
}
