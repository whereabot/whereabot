# Roadmap

Each line links the PR or file that owns the details.

## Shipped

- Install from the browser (PWA): [#1](https://github.com/whereabot/whereabot/pull/1), icon refresh [#11](https://github.com/whereabot/whereabot/pull/11).
- Operations docs: [#2](https://github.com/whereabot/whereabot/pull/2).
- Location sharing between people: [#3](https://github.com/whereabot/whereabot/pull/3).
- Landing "Get started" for agent and device: [#4](https://github.com/whereabot/whereabot/pull/4), [#19](https://github.com/whereabot/whereabot/pull/19).
- One-screen app layout: [#5](https://github.com/whereabot/whereabot/pull/5).
- Web Push wake-up for the browser app: [#6](https://github.com/whereabot/whereabot/pull/6), on in prod [#12](https://github.com/whereabot/whereabot/pull/12).
- Brand icon: [#7](https://github.com/whereabot/whereabot/pull/7).
- App lint baseline: [#9](https://github.com/whereabot/whereabot/pull/9).
- Device kind (app, PWA, browser) in the device list: [#10](https://github.com/whereabot/whereabot/pull/10).
- Deploys wait for full CI: [#13](https://github.com/whereabot/whereabot/pull/13); hardened workflows: [#20](https://github.com/whereabot/whereabot/pull/20).
- A reinstalled app reuses its device row: [#17](https://github.com/whereabot/whereabot/pull/17).
- Android AAB to Google Play internal testing: [#18](https://github.com/whereabot/whereabot/pull/18).
- MIT license fields and notices: [#21](https://github.com/whereabot/whereabot/pull/21).
- Community files and [privacy](privacy.md) statement: [#22](https://github.com/whereabot/whereabot/pull/22).
- Agent tokens kept out of phone paths: [#23](https://github.com/whereabot/whereabot/pull/23).
- Earlier: OAuth 2.1 for MCP clients, custom domain, APNs wake-up, several devices per user, iOS Keychain session. See [architecture](architecture.md).

## In review

- Automatic app release on merge: [#14](https://github.com/whereabot/whereabot/pull/14).
- Submit iOS builds to App Store review: [#15](https://github.com/whereabot/whereabot/pull/15).
- Per-PR previews of site, docs and `/app`: [#16](https://github.com/whereabot/whereabot/pull/16).
- Rate limit on locate and a cap on pending requests: [#28](https://github.com/whereabot/whereabot/pull/28).

## Now

- **App CSP.** The Tauri app sets `csp: null`. See [tauri.conf.json](../app/src-tauri/tauri.conf.json).
- **Pause sharing.** One switch in the app that declines every request until you turn it back on.
- **Keystore storage on Android.** Move the Supabase session from `localStorage` to the Android Keystore.

## Next

- **End-to-end encryption.** Encrypt the fix on the phone with a key the relay never sees.
- **Remove device ends its session.** Today the removed device stays signed in.
- **Two same-size phones on one account.** They can share one device row. See [reinstall-device-match.ts](../app/src/devices/reinstall-device-match.ts).

## Later

- **MCP rate limit.** The MCP path of [#28](https://github.com/whereabot/whereabot/pull/28) has no own test. `/mcp` calls that send no push have no limit.
- **Delete account in the app.** Today deletion needs a request to the operator. See [privacy](privacy.md#retention-and-deletion).
- **Connected agents list.** Show granted agents in the app, with a revoke button.
- **Android push (FCM).** The Android app answers only while open. It needs `google-services.json`, the gms plugin and an FCM sender in the relay.
- **Store listings.** App Store and Google Play listings, and store links on the site, are not live.
