/**
 * Which Tauri shell runs the app. "mobile": the geolocation plugin works. "mac": CoreLocation
 * commands (macos-location.ts), since WKWebView denies navigator.geolocation. "desktop": the
 * webview's navigator.geolocation (the plugin is a stub there). iPadOS reports a Mac user agent;
 * touch points tell them apart.
 */
export function shellKind(inTauri: boolean, ua: string, maxTouchPoints: number): "web" | "mobile" | "mac" | "desktop" {
  if (!inTauri) return "web";
  const mac = /Macintosh/.test(ua);
  if (/iPhone|iPad|iPod|Android/.test(ua) || (mac && maxTouchPoints > 1)) return "mobile";
  return mac ? "mac" : "desktop";
}
