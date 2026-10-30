/**
 * Runs in the page before app scripts. Fakes the Tauri IPC bridge for the plugins the app calls.
 * Self-contained on purpose: Playwright serialises this function into the page.
 * cfg: { permission, notifications ('granted' by default), connect: 'ok' | 'hang' | 'reject', connectError, locationError }
 */
export function installTauri(cfg) {
  const callbacks = {};
  let nextId = 1;
  let connects = 0;
  let channel = null;
  let seq = 0;
  const position = {
    timestamp: 1767268800000,
    coords: { latitude: 52.52, longitude: 13.405, accuracy: 8, altitude: null, altitudeAccuracy: null, heading: null, speed: null },
  };
  const invoke = async (cmd, args) => {
    if (cmd === 'plugin:geolocation|check_permissions' || cmd === 'plugin:geolocation|request_permissions') {
      return { location: cfg.permission, coarseLocation: cfg.permission };
    }
    if (cmd === 'plugin:notification|is_permission_granted') return (cfg.notifications ?? 'granted') === 'granted';
    if (cmd === 'plugin:geolocation|get_current_position') {
      if (cfg.locationError) throw cfg.locationError;
      return position;
    }
    if (cmd === 'plugin:geolocation|watch_position') {
      if (cfg.locationError) throw cfg.locationError;
      // The fix is precise, so the app answers on the first reading.
      setTimeout(() => callbacks[args.channel.id]?.({ index: 0, message: position }), 0);
      return null;
    }
    if (cmd === 'plugin:websocket|connect') {
      connects += 1;
      // Only the first connect answers. Later retries hang, so the screen stays put.
      if (connects > 1 || cfg.connect === 'hang') return new Promise(() => {});
      if (cfg.connect === 'reject') throw cfg.connectError;
      channel = args.onMessage;
      return 1;
    }
    return null;
  };
  // The notification plugin reads window.Notification first.
  window.Notification = { permission: (cfg.notifications ?? 'granted') === 'granted' ? 'granted' : 'default', requestPermission: async () => 'default' };
  window.isTauri = true; // app/src/tauri-runtime.ts: take the native code paths
  // Play the phone app: desktop Tauri uses the browser location path (mobileShell). A Mac user
  // agent with touch points reads as iPadOS: mobile shell, but no iPhone-only background step.
  Object.defineProperty(navigator, 'userAgent', { get: () => 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)' });
  Object.defineProperty(navigator, 'maxTouchPoints', { get: () => 5 });
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } },
    transformCallback(cb) {
      const id = nextId++;
      callbacks[id] = cb;
      return id;
    },
    unregisterCallback(id) {
      delete callbacks[id];
    },
    convertFileSrc: (path) => path,
    invoke,
  };
  // Delivers one message from the relay to the app's WebSocket listener.
  window.__screensEmit = (message) => callbacks[channel.id]({ index: seq++, message });
}
