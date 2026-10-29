// Worker for the browser build (/app): makes the page installable and shows Web Push alerts.
// shortcut: no offline shell; the app needs the relay anyway. Add caching if offline start matters.
self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (e) => e.waitUntil(self.clients.claim()));
self.addEventListener("fetch", () => {}); // network as usual

// The relay sends the APNs alert text (server/src/push/web_push). A worker cannot read the
// location: the user opens the app, the relay resends the waiting request, the in-app prompt answers.
self.addEventListener("push", (e) => {
  let data = {};
  try {
    data = e.data ? e.data.json() : {};
  } catch {
    // Unreadable payload: still tell the user something is waiting.
  }
  const body = data.body || "An agent wants to know your location";
  // Same tag as the in-page notice, so one request shows one notification.
  const options = { body, tag: "whereabot-locate", renotify: true, data: { request_id: data.request_id } };
  e.waitUntil(self.registration.showNotification(data.title || "Whereabot", options));
});

self.addEventListener("notificationclick", (e) => {
  e.notification.close();
  const app = self.registration.scope; // https://whereabot.io/app/
  const focusOrOpen = self.clients.matchAll({ type: "window", includeUncontrolled: true }).then((wins) => {
    const open = wins.find((w) => w.url.startsWith(app));
    return open ? open.focus() : self.clients.openWindow(app);
  });
  e.waitUntil(focusOrOpen);
});
