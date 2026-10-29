// Browser (PWA) cases: no Tauri stub, so the app takes the web paths and shows the PWA limits notice.
// `ua` swaps in an iPhone Safari user agent, which adds the Add to Home Screen steps.
const IPHONE_SAFARI = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
// Signed-in cases grant location and fake granted notifications (the test origin is not secure,
// so Chromium refuses a real grant). Home then shows instead of the permission steps.
const GRANT = ['geolocation'];
const notificationsOn = () => Object.defineProperty(Notification, 'permission', { get: () => 'granted' });

export const PWA_CASES = [
  { name: 'pwa-signed-out', pwa: true, wait: '#signed-out .pwa-limits', desc: 'Browser version, signed out: sign-in button and the browser limits notice.' },
  { name: 'pwa-ios-install', pwa: true, ua: IPHONE_SAFARI, wait: '#signed-out .pwa-install', desc: 'iPhone Safari, not installed: Add to Home Screen steps, then the limits notice.' },
  { name: 'pwa-home', pwa: true, session: true, grant: GRANT, pageInit: notificationsOn, wait: '#home .pwa-limits', desc: 'Browser version, signed in: the first screen after sign-in with the limits notice.' },
  { name: 'pwa-ios-home', pwa: true, session: true, grant: GRANT, pageInit: notificationsOn, ua: IPHONE_SAFARI, wait: '#home .pwa-install', desc: 'iPhone Safari, signed in: the first screen after sign-in with install steps and limits notice.' },
];
