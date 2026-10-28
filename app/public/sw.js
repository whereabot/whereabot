// Minimal worker for the browser build (/app): makes the page installable, caches nothing.
// shortcut: no offline shell; the app needs the relay anyway. Add caching if offline start matters.
self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (e) => e.waitUntil(self.clients.claim()));
self.addEventListener("fetch", () => {}); // network as usual
