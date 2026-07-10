# Using wayfinder-mcp with your AI client

`wayfinder-mcp` is a **stdio** MCP server: your client launches it as a local
process and talks to it over stdin/stdout. That means it works today with local
desktop and CLI clients. Cloud clients (Claude.ai web/mobile, ChatGPT) connect
to MCP servers from the vendor's servers over the public internet and need a
**remote HTTP** server, which wayfinder-mcp does not provide yet.

## Compatibility at a glance

| Client | MCP support today | Transport it needs | Works with wayfinder-mcp now? |
| --- | --- | --- | --- |
| Claude Desktop (macOS/Windows) | Yes | stdio (local) | Yes |
| Claude Code (CLI) | Yes | stdio (local) | Yes |
| Codex CLI (OpenAI) | Yes | stdio (local) | Yes |
| Claude.ai web | Yes, remote-only | Remote HTTPS | No, needs remote transport |
| Claude mobile (iOS/Android) | Yes, remote-only | Remote HTTPS | No, needs remote transport |
| ChatGPT web (Developer Mode) | Yes, remote-only | Remote HTTPS | No, needs remote transport |
| ChatGPT desktop / mobile | No custom MCP | Remote HTTPS | No |

Three clients work out of the box today, all local/CLI. Every web and mobile
surface, for both Claude and ChatGPT, is blocked on adding a remote HTTP
transport (see [Adding a remote transport](#adding-a-remote-transport)).

## Install

```sh
cargo install wayfinder-mcp        # or: brew install jhheider/tap/wayfinder-mcp
which wayfinder-mcp                 # e.g. /Users/you/.cargo/bin/wayfinder-mcp
```

Use that absolute path in the configs below. **Desktop GUI apps do not inherit
your shell `PATH`, so a bare `"wayfinder-mcp"` will often fail to launch. Always
use the full path in Claude Desktop.**

## Claude Desktop (supported, stdio)

Edit your Claude Desktop config file:

- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`

(Or: Claude menu, Settings, Developer, Edit Config.)

```json
{
  "mcpServers": {
    "wayfinder": {
      "command": "/Users/you/.cargo/bin/wayfinder-mcp"
    }
  }
}
```

Replace the path with your `which wayfinder-mcp` output, then fully quit and
reopen Claude Desktop. "wayfinder" appears under the connectors menu, exposing
`search`, `get`, and `list_categories`. Try: *"Use wayfinder to show me the
Fireball spell in PF2e."*

## Claude Code (supported, stdio)

One command (stdio is the default transport; `--` separates Claude Code's args
from the server command):

```sh
claude mcp add wayfinder -- wayfinder-mcp
```

To share it with a project (writes `.mcp.json` to the repo):

```sh
claude mcp add --scope project wayfinder -- wayfinder-mcp
```

Verify with `claude mcp list`. A bare `wayfinder-mcp` is fine here because Claude
Code inherits your shell `PATH`.

## Codex CLI (supported, stdio)

Codex CLI reads stdio MCP servers from `~/.codex/config.toml`:

```sh
codex mcp add wayfinder -- wayfinder-mcp
```

or edit `~/.codex/config.toml` directly:

```toml
[mcp_servers.wayfinder]
command = "wayfinder-mcp"
```

This config is shared with the Codex IDE extension.

## Claude.ai web (not supported yet, needs a remote transport)

Claude.ai supports custom connectors via remote MCP (Settings, Connectors, Add
custom connector), but it connects from Anthropic's cloud to a public HTTPS URL.
It cannot launch a local stdio binary, so wayfinder-mcp cannot be added here yet.

## Claude mobile / iOS / Android (not supported yet)

Claude mobile can use remote MCP connectors, but only ones you added on the
Claude.ai website. Same blocker: remote HTTPS only, no stdio.

## ChatGPT web, Developer Mode (not supported yet)

ChatGPT's Developer Mode supports MCP, but only remote MCP servers over HTTPS,
not local stdio servers. Availability is gated by plan (Plus/Pro/Business/
Enterprise/Edu on the web; full tool use is Business/Enterprise/Edu). A local
stdio server can be bridged to HTTPS with a tool like `mcp-remote`, but that is
a user-side workaround, not something wayfinder ships.

## ChatGPT desktop and mobile (not supported)

Custom MCP connectors are a web-only feature; the ChatGPT desktop and mobile
apps do not offer custom connector setup, and it would be remote-only anyway.

## Adding a remote transport

To reach Claude.ai web, Claude mobile, and ChatGPT, wayfinder-mcp needs a
Streamable HTTP endpoint on a public HTTPS URL. `rmcp` (already a dependency)
ships a streamable-http server transport, so this is additive rather than a
rewrite: a feature flag or second binary mode serving the same server. Because
this server is read-only over public game data with no secrets, hosting risk is
low. This is tracked as a post-launch enhancement.
