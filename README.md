# wayfinder

[![CI](https://github.com/jhheider/wayfinder/actions/workflows/ci.yml/badge.svg)](https://github.com/jhheider/wayfinder/actions/workflows/ci.yml)
[![Coverage Status](https://coveralls.io/repos/github/jhheider/wayfinder/badge.svg?branch=main)](https://coveralls.io/github/jhheider/wayfinder?branch=main)
[![crates.io](https://img.shields.io/crates/v/wayfinder-cli.svg)](https://crates.io/crates/wayfinder-cli)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](#license)

**Search Archives of Nethys from your terminal.**

`wf` is a fast, colorized lookup tool for [Archives of Nethys](https://2e.aonprd.com)
Pathfinder 2e and Starfinder 2e data. Search by category and filters, show a full
rendered document, and cache results locally so repeat lookups are instant -- all
without leaving the shell.

```sh
# broad search across everything
wf search fireball

# scoped to a category, with filters
wf search deity -f domain=Dragon
wf search spell --name Fireball --level 3

# show a full document, rendered for the terminal
wf show spell Fireball

# discover what you can search
wf categories
wf fields deity

# Starfinder 2e instead of Pathfinder 2e
wf --sf2e search class

# raw JSON for scripting, or raw AON markdown
wf --format json search spell --name Fireball
wf --format md show feat Power_Attack

# inspect or clean the local cache
wf cache status
wf cache purge
```

## Install

```sh
# the `wf` CLI
cargo install wayfinder-cli
brew install jhheider/tap/wf

# the MCP server
cargo install wayfinder-mcp
brew install jhheider/tap/wayfinder-mcp
```

Each binary is packaged separately. Prebuilt archives for macOS, Linux (musl),
and Windows are attached to every
[release](https://github.com/jhheider/wayfinder/releases).

Before the first tagged release lands on crates.io / Homebrew, install straight
from source:

```sh
cargo install --git https://github.com/jhheider/wayfinder wayfinder-cli
cargo install --git https://github.com/jhheider/wayfinder wayfinder-mcp
```

## What's in the box

This is a Cargo workspace with three crates -- one AON client library and two
frontends:

- **[`wayfinder-core`](crates/wayfinder-core)** -- the library: AON Elasticsearch
  client, SQLite cache with TTLs, unified search, and an HTML/markdown renderer.
  Depend on it directly to build your own AON-backed tools.
- **[`wayfinder-cli`](crates/wayfinder-cli)** -- the `wf` binary built on top of it.
- **[`wayfinder-mcp`](crates/wayfinder-mcp)** -- an [MCP](https://modelcontextprotocol.io)
  server exposing AON data (`search`, `get`, `list_categories`) to LLM tools like
  Claude (see below).

## Use it from your AI assistant (MCP)

`wayfinder-mcp` exposes AON search to LLM tools as three MCP tools: `search`,
`get`, and `list_categories` (Pathfinder 2e by default, `sf2e` optional). Ask
Claude things like *"What does the Grab an Edge action do in PF2e?"* and it
looks them up live.

It is a **stdio** server, so it works today with **Claude Desktop, Claude Code,
and Codex CLI**. Cloud clients (Claude.ai web/mobile, ChatGPT) need a remote
HTTP server, which is not shipped yet. Full per-client instructions and the
compatibility matrix are in **[docs/mcp-setup.md](docs/mcp-setup.md)**.

Quick start with Claude Code:

```sh
cargo install wayfinder-mcp
claude mcp add wayfinder -- wayfinder-mcp
```

For Claude Desktop, use the **absolute** path (GUI apps do not inherit your
shell `PATH`) from `which wayfinder-mcp` in `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "wayfinder": { "command": "/Users/you/.cargo/bin/wayfinder-mcp" }
  }
}
```

## Data sources

| System | Index | Docs | Base |
| --- | --- | --- | --- |
| Pathfinder 2e | `aon70` | ~39k | <https://2e.aonprd.com> |
| Starfinder 2e | `aonsf10` | ~6k | <https://2e.aonsrd.com> |

Both query AON's public Elasticsearch backend at `elasticsearch.aonprd.com`.
Documents you look up are cached locally (SQLite) with per-category TTLs as a
side effect of searching; use `wf cache` to inspect or purge that store. The
tool sends an identifying `User-Agent` and honors `Retry-After` backoff, and it
does not bulk-mirror AON: results are capped and cached only as you query them.

TLS is rustls with the ring provider -- no OpenSSL or aws-lc, so release builds
cross-compile cleanly to musl and aarch64.

## Output formats

- `--format pretty` (default) -- colorized terminal output with icons.
- `--format json` -- raw JSON, for piping and scripting.
- `--format md` -- raw AON markdown.

## Credits

Game data comes from [Archives of Nethys](https://2e.aonprd.com), the official
Pathfinder 2e / Starfinder 2e online reference. Pathfinder and Starfinder are
trademarks of Paizo Inc.; game mechanics and rules text are used under Paizo's
[Community Use Policy](https://paizo.com/community/communityuse) and the ORC /
OGL where applicable. This project is not published, endorsed by, or affiliated
with Paizo Inc. or Archives of Nethys.

## License

MIT. See [LICENSE](LICENSE). Applies to this tool's own code, not to the
game-data content it retrieves.
