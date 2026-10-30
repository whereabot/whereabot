# Roadmap

Shipped: OAuth 2.1 sign-in for MCP clients, the custom domain, APNs and Web Push wake-up, several devices per user, and the iOS Keychain session. Details: [architecture](architecture.md).

1. **Android push (FCM).** The Android app answers only while it is open. Web Push covers the browser app.
2. **Connected agents list.** Show granted agents in the app, with a revoke button.
3. **Delete account in the app.** Today deletion needs a request to the operator. See [privacy](privacy.md#retention-and-deletion).
4. **Keystore storage on Android.** Move the Supabase session from `localStorage` to the Android Keystore.
5. **End-to-end encryption.** Encrypt the fix on the phone with a key the relay never sees.
6. **Pause sharing.** One switch in the app that declines every request until you turn it back on.
