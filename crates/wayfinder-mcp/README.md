# wayfinder-mcp

[![crates.io](https://img.shields.io/crates/v/wayfinder-mcp.svg)](https://crates.io/crates/wayfinder-mcp)

An [MCP](https://modelcontextprotocol.io) server exposing
[Archives of Nethys](https://2e.aonprd.com) Pathfinder 2e and Starfinder 2e game
data to LLM tools like Claude. Built on
[`wayfinder-core`](https://crates.io/crates/wayfinder-core), so it shares one AON
client (rustls + ring TLS, no OpenSSL/aws-lc) with the `wf` CLI.

It speaks JSON-RPC over stdio and provides three tools:

- **`search`** -- free-text query plus filters (category, traits, level range,
  source, rarity), with sort and result-limit control.
- **`get`** -- full rules text for one entry by exact `name` (legacy pre-remaster
  names resolve too) or by AoN `url`.
- **`list_categories`** -- live category names and entry counts for a game.

Every tool takes an optional `game`: `"pf2e"` (default) or `"sf2e"`.

## Install

```sh
cargo install wayfinder-mcp
```

## Configure an MCP client

It is a **stdio** server, so it works with local clients (Claude Desktop, Claude
Code, Codex CLI). Cloud clients (Claude.ai web/mobile, ChatGPT) need a remote
HTTP transport, which is not shipped yet.

Point your client at the installed binary. For Claude Desktop, use the
**absolute** path (GUI apps do not inherit your shell `PATH`) from
`which wayfinder-mcp`:

```json
{
  "mcpServers": {
    "wayfinder": {
      "command": "/Users/you/.cargo/bin/wayfinder-mcp"
    }
  }
}
```

With Claude Code, one command does it: `claude mcp add wayfinder -- wayfinder-mcp`.

Full per-client setup and the compatibility matrix are in
[docs/mcp-setup.md](https://github.com/jhheider/wayfinder/blob/main/docs/mcp-setup.md).

## License

MIT.
