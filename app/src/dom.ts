export const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

export function setStatus(text: string) {
  $("status").textContent = text;
}

export function showView(view: "signed-in" | "signed-out") {
  $("signed-in").hidden = view !== "signed-in";
  $("signed-out").hidden = view !== "signed-out";
}
