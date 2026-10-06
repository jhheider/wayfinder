# wayfinder-mcp

[![crates.io](https://img.shields.io/crates/v/wayfinder-mcp.svg)](https://crates.io/crates/wayfinder-mcp)

An [MCP](https://modelcontextprotocol.io) server exposing
[Archives of Nethys](https://2e.aonprd.com) Pathfinder 2e and Starfinder 2e game
data to LLM tools like Claude. Built on
[`wayfinder-core`](https://crates.io/crates/wayfinder-core), so it shares one AON
client (rustls + ring TLS, no OpenSSL/aws-lc) with the `wf` CLI.

It speaks JSON-RPC over stdio and provides three tools:

- **`search`** -- free-text query plus filters (category, traits, level range,
  source, rarity, and exact field values such as `{"tradition": "arcane"}`),
  with sort, `limit` and `offset` paging.
- **`get`** -- full rules text for one entry, as markdown that keeps AON's
  structure (stat blocks, action costs, heightening), by exact `name` or by AoN
  `url`. Legacy pre-remaster names resolve too. When several entries share a
  name ("Shield" is a spell, an implement and a weapon group), it returns the
  likeliest and lists the rest.
- **`list_categories`** -- live category names and entry counts for a game.

Every tool takes an optional `game`: `"pf2e"` (default) or `"sf2e"`. Results
follow the Remaster: an entry it replaced appears as its remastered version
unless `legacy: true`. Categories are checked against the game's live list
(case and plurals are forgiven), and mistakes come back as tool errors with a
suggestion, so the model can correct itself.

## Install

```sh
cargo install wayfinder-mcp
```

## Configure an MCP client

It is a **stdio** server, so it works with local clients (Claude Desktop, Claude
Code, Codex CLI). Cloud clients (Claude.ai web/mobile, ChatGPT) are
remote-only; serving a public HTTP endpoint is out of scope by design, since
without a client identity in the transport it is an open relay.

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
