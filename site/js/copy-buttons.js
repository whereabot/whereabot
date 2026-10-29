// Adds a Copy button under each <code data-copy>. Without JS (or clipboard access) the code stays selectable.
if (navigator.clipboard) {
  for (const code of document.querySelectorAll("code[data-copy]")) {
    const button = Object.assign(document.createElement("button"), {
      type: "button",
      className: "btn btn-secondary btn-small copy-button",
      textContent: "Copy",
    });
    button.setAttribute("aria-label", `Copy ${code.textContent}`);
    button.addEventListener("click", async () => {
      await navigator.clipboard.writeText(code.textContent);
      button.textContent = "Copied";
      setTimeout(() => (button.textContent = "Copy"), 2000);
    });
    code.after(button);
  }
}
