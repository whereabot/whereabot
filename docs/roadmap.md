# Roadmap

What comes next. Each line links the PR or file that owns the details. For what already shipped, see the [merged pull requests](https://github.com/whereabot/whereabot/pulls?q=is%3Apr+is%3Amerged).

## Now

- **App CSP.** The Tauri app sets `csp: null`. See [tauri.conf.json](../app/src-tauri/tauri.conf.json).
- **Pause sharing.** One switch in the app that declines every request until you turn it back on.
- **Keystore storage on Android.** Move the Supabase session from `localStorage` to the Android Keystore.

## Next

- **App Store and Google Play listings.** Store pages and store links on the site. Releases already run on every app change.
- **End-to-end encryption.** Encrypt the fix on the phone with a key the relay never sees.
- **Remove device ends its session.** Today a removed device stays signed in.
- **Two same-size phones on one account.** They can share one device row. See [reinstall-device-match.ts](../app/src/devices/reinstall-device-match.ts).

## Later

- **Delete account in the app.** Today deletion needs a request to the operator. See [privacy](privacy.md#retention-and-deletion).
- **Connected agents list.** Show granted agents in the app, with a revoke button.
- **Android push (FCM).** The Android app answers only while open. It needs `google-services.json`, the gms plugin and an FCM sender in the relay.
- **Limits on all MCP calls.** Calls that send no push, such as listing who shares with you, have no rate limit yet.
