# wayfinder-cli

[![crates.io](https://img.shields.io/crates/v/wayfinder-cli.svg)](https://crates.io/crates/wayfinder-cli)

`wf` — a fast, colorized terminal tool for searching and browsing
[Archives of Nethys](https://2e.aonprd.com) Pathfinder 2e and Starfinder 2e data.

```sh
cargo install wayfinder-cli      # installs the `wf` binary
```

```sh
wf search deity -f domain=Dragon
wf show spell Fireball
wf categories
wf --sf2e search class
wf --format json search spell --name Fireball
wf cache fetch spell
```

See the [project README](https://github.com/jhheider/wayfinder) for the full
tour, and [`wayfinder-core`](https://crates.io/crates/wayfinder-core) for the
library it's built on.

## License

MIT.
