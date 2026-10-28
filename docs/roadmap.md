# Roadmap

1. **Wake a closed app.** iOS and Android stop background apps, and the WebSocket dies with them. The relay must send a push (APNs on iOS, FCM on Android) that wakes the app to read GPS and answer over HTTPS. Tauri has no official push plugin, so this needs native code. This is the main risk — spike it first.
2. **MCP server.** Expose `get_phone_location` as an MCP tool that calls `POST /v1/locate`, so agents can use it without curl.
3. **Secure token storage.** Move the device token from `localStorage` to the OS keychain.
4. **Several phones per relay.** Name each device; the agent picks one.
5. **End-to-end encryption.** Encrypt the fix on the phone with a key the relay never sees.
6. **Consent log in the app.** Show every ping and allow "pause sharing".
