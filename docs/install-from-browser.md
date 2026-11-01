# Install from the browser

You can use Whereabot without the App Store. Open [whereabot.io/app](https://whereabot.io/app/) and add it to the Home Screen.

## Install

1. On iPhone, open `https://whereabot.io/app/` in Safari.
2. Tap Share, then **Add to Home Screen**, then **Add**.
3. Open Whereabot from the Home Screen and sign in with Apple or Google.
4. Allow location and notifications when Whereabot asks.

In another browser on iPhone, open the address in Safari first. On Android or a computer, open the same address in Chrome or Edge and tap **Install Whereabot** (or choose **Install app** in the browser menu). The landing page shows the steps for your device ([`site/js/landing-install-help.js`](../site/js/landing-install-help.js)).

## Limits

The browser app is the same code as the iOS app ([`app/src`](../app/src)). Where a native plugin is missing, it uses a web API instead ([`app/src/tauri-runtime.ts`](../app/src/tauri-runtime.ts)).

- **It answers only after you open it.** A request sends a Web Push notification: "Your agent X wants to know your location". Tap it to open Whereabot; the in-app prompt then answers. A service worker cannot read the location, so the notification has no Allow button. The App Store app can answer from the notification.
- **Web Push needs setup.** On iPhone it needs iOS 16.4 or later and the Home Screen install. Allow notifications in the app. The app stores the subscription after sign-in and removes it on sign-out ([`app/src/push/web-push-subscription.ts`](../app/src/push/web-push-subscription.ts)).
- **No "Always" location.** A web page reads the location only while it is in front.
- **Settings.** If you denied location, change it in the browser settings (on iPhone: Settings > Apps > Safari > Location). The app cannot open Settings for you.
- **The relay token.** A browser cannot set an `Authorization` header on a WebSocket. The browser app sends the Supabase token as a WebSocket subprotocol instead ([`app/src/browser-relay-socket.ts`](../app/src/browser-relay-socket.ts), [`server/src/devices/websocket_subprotocol_bearer.rs`](../server/src/devices/websocket_subprotocol_bearer.rs)).

The app shows these limits too: on the sign-in screen, on Home, in the permission steps and next to Always allow ([`app/src/ui/pwa-limits-notice.ts`](../app/src/ui/pwa-limits-notice.ts)).

## How it ships

The site deploy builds `app/` with `npm run build:web` and serves it at `/app/` ([`.github/workflows/site-cloudflare-deploy.yml`](../.github/workflows/site-cloudflare-deploy.yml)). Supabase allows `https://whereabot.io/app/` as a sign-in redirect ([`infra/modules/whereabouts/supabase.tf`](../infra/modules/whereabouts/supabase.tf)).
