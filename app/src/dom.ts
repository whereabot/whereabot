export const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

export function setStatus(text: string) {
  $("status").textContent = text;
}
