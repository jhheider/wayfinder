# wayfinder-mcp

[![crates.io](https://img.shields.io/crates/v/wayfinder-mcp.svg)](https://crates.io/crates/wayfinder-mcp)

An [MCP](https://modelcontextprotocol.io) server exposing
[Archives of Nethys](https://2e.aonprd.com) Pathfinder 2e and Starfinder 2e game
data to LLM tools like Claude. Built on
[`wayfinder-core`](https://crates.io/crates/wayfinder-core), so it shares one AON
client (rustls + ring TLS, no OpenSSL/aws-lc) with the `wf` CLI.

It speaks JSON-RPC over stdio and provides three tools:

- **`search`** — free-text query plus filters (category, traits, level range,
  source, rarity), with sort and result-limit control.
- **`get`** — full rules text for one entry by exact `name` (legacy pre-remaster
  names resolve too) or by AoN `url`.
- **`list_categories`** — live category names and entry counts for a game.

Every tool takes an optional `game`: `"pf2e"` (default) or `"sf2e"`.

## Install

```sh
cargo install wayfinder-mcp
```

## Configure an MCP client

Point your client at the installed binary. For example, in a Claude MCP config:

```json
{
  "mcpServers": {
    "wayfinder": {
      "command": "wayfinder-mcp"
    }
  }
}
```

See the [project README](https://github.com/jhheider/wayfinder) for the full
workspace.

## License

MIT.
