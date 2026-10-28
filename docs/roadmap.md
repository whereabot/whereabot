# Roadmap

1. **OAuth 2.1 for MCP clients (coming next).** The MCP client signs in through the browser. Users never create or paste tokens. The relay validates short-lived tokens and publishes the MCP authorization metadata. Agent keys stay as a headless fallback only.
2. **Wake a closed app.** iOS and Android stop background apps, and the WebSocket dies with them. The relay must send a push (APNs on iOS, FCM on Android) that wakes the app to read GPS and answer. Tauri has no official push plugin, so this needs native code. This is the main risk. Spike it first.
3. **Keychain storage for the session.** Move the Supabase session from `localStorage` to the OS keychain.
4. **Custom domain.** Serve the relay on a domain you own, instead of the container host's URL.
5. **Several devices per user.** Today one phone per user. Allow more, and let the agent pick one.
6. **End-to-end encryption.** Encrypt the fix on the phone with a key the relay never sees.
7. **Consent log in the app.** Show every ping and allow "pause sharing".
