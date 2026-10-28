# Roadmap

1. **Wake a closed app.** iOS and Android stop background apps, and the WebSocket dies with them. The relay must send a push (APNs on iOS, FCM on Android) that wakes the app to read GPS and answer. Tauri has no official push plugin, so this needs native code. This is the main risk. Spike it first.
2. **Keychain storage for the session.** Move the Supabase session from `localStorage` to the OS keychain.
3. **Custom domain.** Serve the relay on a domain you own, instead of the container host's URL.
4. **Several devices per user.** Today one phone per user. Allow more, and let the agent pick one.
5. **End-to-end encryption.** Encrypt the fix on the phone with a key the relay never sees.
6. **Consent log in the app.** Show every ping and allow "pause sharing".
