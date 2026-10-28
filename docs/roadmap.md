# Roadmap

1. **OAuth 2.1 for MCP clients.** Live: the MCP client signs in through the browser and users never paste tokens. Agent keys stay as a headless fallback only.
2. **Push notifications (APNs / FCM).** iOS and Android stop background apps, and the WebSocket dies with them. The relay sends a push that wakes the app to read GPS and answer. Tauri has no official push plugin, so this needs native code. This is the main risk. Spike it first.
3. **Connected agents list.** Show granted agents in the app, with a revoke button.
4. **Keychain storage for the session.** Move the Supabase session from `localStorage` to the OS keychain.
5. **Custom domain.** Serve the relay on a domain you own, instead of the container host's URL.
6. **Several devices per user (multi-phone).** Today one phone per user. Allow more, and let the agent pick one.
7. **End-to-end encryption.** Encrypt the fix on the phone with a key the relay never sees.
8. **Consent log in the app.** Show every ping and allow "pause sharing".
