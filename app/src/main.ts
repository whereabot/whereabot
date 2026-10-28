import { $ } from "./dom";
import { disconnect, start } from "./connection";

window.addEventListener("DOMContentLoaded", () => {
  const server = $<HTMLInputElement>("server");
  const token = $<HTMLInputElement>("token");
  server.value = localStorage.getItem("server") ?? "";
  // shortcut: token in localStorage, move to OS keychain (stronghold plugin) before 1.0.
  token.value = localStorage.getItem("token") ?? "";

  $("pair-form").addEventListener("submit", async (e) => {
    e.preventDefault();
    localStorage.setItem("server", server.value);
    localStorage.setItem("token", token.value);
    await start(server.value, token.value);
  });
  $("disconnect").addEventListener("click", disconnect);

  if (server.value && token.value) void start(server.value, token.value);
});
