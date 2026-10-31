# Listings

Where MCP users can find Whereabot, and what each directory needs. Checked October 2026.

The listing data lives in two files at the repo root:

- [`server.json`](https://github.com/whereabot/whereabot/blob/main/server.json): the [official MCP Registry](https://registry.modelcontextprotocol.io) entry. Name, description, remote URL and icon come from here.
- [`glama.json`](https://github.com/whereabot/whereabot/blob/main/glama.json): the GitHub users who may claim and edit the [Glama](https://glama.ai/mcp/servers) listing.

## Status

| Directory | How it lists a server | Status |
|-----------|-----------------------|--------|
| [Official MCP Registry](https://registry.modelcontextprotocol.io) | Owner publishes `server.json` with `mcp-publisher` | Ready to publish |
| [PulseMCP](https://www.pulsemcp.com) | Reads the official registry | Automatic after registry publish. Its own form is paused. |
| [Glama](https://glama.ai/mcp/servers) | Indexes public GitHub repos; owner claims with `glama.json` | Add the repo, then claim |
| [Glama connectors](https://glama.ai/mcp/connectors) | Remote endpoints, keyed by the registry name | Add after registry publish |
| [Smithery](https://smithery.ai) | Owner pastes the remote URL at smithery.ai/new and signs in once to scan | Owner step |
| [mcp.so](https://mcp.so) | GitHub issue in `chatmcp/mcpso` (free) or paid form | Owner step |
| [MCP Market](https://mcpmarket.com) | Form, "Remote MCP", free queue | Owner step |
| [mcpservers.org](https://mcpservers.org) | Form (also feeds `wong2/awesome-mcp-servers`, which takes no PRs) | Owner step |
| [Cline MCP Marketplace](https://github.com/cline/mcp-marketplace) | GitHub issue with a 400×400 PNG logo; Cline must install it from the README | Owner step |
| [punkpeye/awesome-mcp-servers](https://github.com/punkpeye/awesome-mcp-servers) | PR; CI wants the Glama score badge | After the Glama listing exists |
| [punkpeye/awesome-remote-mcp-servers](https://github.com/punkpeye/awesome-remote-mcp-servers) | PR; needs a Glama connector badge and a star on the repo | After the Glama connector exists |
| [Claude Connectors Directory](https://claude.com/docs/connectors/directory) | Developer portal, any paid Claude plan | Blocked: tools need `title` and `readOnlyHint` |
| `appcypher/awesome-mcp-servers` | — | Archived; skip |

## Rules

- One name everywhere: `io.whereabot/whereabot` (registry namespace verified by the domain, [`site/.well-known/mcp-registry-auth`](../site/.well-known/mcp-registry-auth)). Glama connectors and mcp.so key on it.
- Links to whereabot.io carry `?ref=<directory>`, so visits per directory show in the request logs.
- Change the description in `server.json` first, then copy it to the other directories.
- Bump `version` in `server.json` before each new `mcp-publisher publish`. The registry refuses a version it already has.

## Publish to the official registry

The registry checks that you own the `io.github.whereabot` namespace. You must be an **Owner** of the `whereabot` GitHub organization. The plain browser sign-in (`mcp-publisher login github`) cannot see organization roles. Sign in with a personal access token that has the `read:org` scope instead:

```bash
mcp-publisher login github --token "$TOKEN"
mcp-publisher publish
```

Check the result: `https://registry.modelcontextprotocol.io/v0/servers?search=whereabot`.

Details: [registry authentication](https://github.com/modelcontextprotocol/registry/blob/main/docs/modelcontextprotocol-io/authentication.mdx), [remote servers](https://github.com/modelcontextprotocol/registry/blob/main/docs/modelcontextprotocol-io/remote-servers.mdx).
