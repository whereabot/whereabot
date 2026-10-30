import { el } from "./screen-view-helpers";

/** Built on first use. Native <dialog> gives the focus trap and Escape. */
function build() {
  const dialog = el("dialog", "") as HTMLDialogElement;
  dialog.setAttribute("aria-labelledby", "confirm-title");
  dialog.setAttribute("aria-describedby", "confirm-text");
  const form = el("form", "stack");
  form.setAttribute("method", "dialog");
  const actions = el("div", "actions");
  const cancel = el("button", "btn btn-secondary", "Cancel") as HTMLButtonElement;
  cancel.value = "cancel";
  cancel.autofocus = true; // the safe choice
  const remove = el("button", "btn btn-danger", "Remove") as HTMLButtonElement;
  remove.value = "remove";
  actions.append(cancel, remove);
  const title = el("h2", "");
  title.id = "confirm-title";
  const text = el("p", "");
  text.id = "confirm-text";
  form.append(title, text, actions);
  dialog.append(form);
  document.body.append(dialog);
  return { dialog, title, text };
}

let parts: ReturnType<typeof build> | undefined;

/** Modal "Remove device?" dialog. Resolves true on Remove, false on Cancel or Escape. */
export function confirmRemove(name: string, isThis: boolean): Promise<boolean> {
  const { dialog, title, text } = (parts ??= build());
  title.textContent = `Remove ${name}?`;
  text.textContent = isThis
    ? "You will be signed out on this device. Sign in again to add it back."
    : "It stops answering location requests and is signed out. Sign in on it again to add it back.";
  return new Promise((resolve) => {
    dialog.addEventListener("close", () => resolve(dialog.returnValue === "remove"), { once: true });
    dialog.returnValue = "";
    dialog.showModal();
  });
}
