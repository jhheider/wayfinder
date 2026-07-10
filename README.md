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

# warm the local cache for a whole category
wf cache fetch spell
wf cache status
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

## What's in the box

This is a Cargo workspace with three crates -- one AON client, three frontends:

- **[`wayfinder-core`](crates/wayfinder-core)** -- the library: AON Elasticsearch
  client, SQLite cache with TTLs, unified search, and an HTML/markdown renderer.
  Depend on it directly to build your own AON-backed tools.
- **[`wayfinder-cli`](crates/wayfinder-cli)** -- the `wf` binary built on top of it.
- **[`wayfinder-mcp`](crates/wayfinder-mcp)** -- an [MCP](https://modelcontextprotocol.io)
  server exposing AON data (`search`, `get`, `list_categories`) to LLM tools like
  Claude. `cargo install wayfinder-mcp`, then point your MCP client at the
  `wayfinder-mcp` binary.

## Data sources

| System | Index | Docs | Base |
| --- | --- | --- | --- |
| Pathfinder 2e | `aon70` | ~39k | <https://2e.aonprd.com> |
| Starfinder 2e | `aonsf10` | ~6k | <https://2e.aonsrd.com> |

Both query AON's public Elasticsearch backend at `elasticsearch.aonprd.com`.
Documents are cached locally (SQLite) with per-category TTLs; use `wf cache` to
inspect, warm, or purge that store.

TLS is rustls with the ring provider -- no OpenSSL or aws-lc, so release builds
cross-compile cleanly to musl and aarch64.

## Output formats

- `--format pretty` (default) -- colorized terminal output with icons.
- `--format json` -- raw JSON, for piping and scripting.
- `--format md` -- raw AON markdown.

## License

MIT. See [LICENSE](LICENSE).
