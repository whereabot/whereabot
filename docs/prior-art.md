# Prior art

Checked October 2026. The question: can an agent ask a **phone** for its current location, on demand, through an open-source stack you host yourself?

| Project | Phone app | On-demand ping | Self-hosted server | Gap |
|---------|-----------|----------------|--------------------|-----|
| [Commit451/phonehome-agent](https://github.com/Commit451/phonehome-agent) | Yes, closed source | Yes, end-to-end encrypted | No — hosted API | Only the agent CLI/MCP is open. |
| [OwnTracks](https://owntracks.org) | Yes, open source | Yes, `reportLocation` cmd, **MQTT only** ([docs](https://owntracks.org/booklet/tech/json/)) | Yes (MQTT broker / Recorder) | No agent API; the MCP bridges below do not use the ping. |
| [de-niji/OwnTracksHermes](https://github.com/de-niji/OwnTracksHermes), [danielsuguimoto/location-tracker](https://github.com/danielsuguimoto/location-tracker), [michelgrolet/tars-location-mcp](https://github.com/michelgrolet/tars-location-mcp) | OwnTracks | No — read the last pushed fix | Yes | Passive history, not "where are you now". |
| [Location MCP Server](https://mcp.so/server/location-mcp-server) | No | — | — | Reads the **computer's** location, not the phone's. |

## Why Whereabouts

No open-source project found gives the full path agent → own relay → ping phone → fresh fix. Whereabouts does it with one Rust binary and one Tauri app. How it works: [architecture](architecture.md).
