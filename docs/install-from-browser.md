# Install from the browser

You can use Whereabot without the App Store. Open [whereabot.io/app](https://whereabot.io/app/) and add it to the Home Screen.

## Install

1. On iPhone, open `https://whereabot.io/app/` in Safari.
2. Tap Share, then **Add to Home Screen**, then **Add**.
3. Open Whereabot from the Home Screen and sign in with Google.
4. Allow location when Whereabot asks.

On Android or a computer, open the same address in Chrome and choose **Install app** (or **Add to Home screen**).

## Limits

The browser app is the same code as the iOS app ([`app/src`](../app/src)). Where a native plugin is missing, it uses a web API instead ([`app/src/tauri-runtime.ts`](../app/src/tauri-runtime.ts)).

- **It answers only while it is open.** iOS stops a web app in the background. A request that arrives while Whereabot is closed times out. The App Store app can answer from the notification.
- **No push notifications.** The browser app does not register for Web Push. Keep it open to see requests. Web Push on iPhone needs iOS 16.4 or later and a Home Screen install; Whereabot does not use it yet.
- **No "Always" location.** A web page reads the location only while it is in front.
- **Settings.** If you denied location, change it in the browser settings (on iPhone: Settings > Apps > Safari > Location). The app cannot open Settings for you.
- **The relay token.** A browser cannot set an `Authorization` header on a WebSocket. The browser app sends the Supabase token as a WebSocket subprotocol instead ([`app/src/browser-relay-socket.ts`](../app/src/browser-relay-socket.ts), [`server/src/devices/websocket_subprotocol_bearer.rs`](../server/src/devices/websocket_subprotocol_bearer.rs)).

The app shows these limits too: on the sign-in screen, on Home, in the permission steps and next to Always allow ([`app/src/ui/pwa-limits-notice.ts`](../app/src/ui/pwa-limits-notice.ts)).

## How it ships

The site deploy builds `app/` with `npm run build:web` and serves it at `/app/` ([`.github/workflows/site-cloudflare-deploy.yml`](../.github/workflows/site-cloudflare-deploy.yml)). Supabase allows `https://whereabot.io/app/` as a sign-in redirect ([`infra/modules/whereabouts/supabase.tf`](../infra/modules/whereabouts/supabase.tf)).
