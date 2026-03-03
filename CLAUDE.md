# Wayfinder — PF2e / SF2e Data Tool

## Project Structure
Rust workspace with three crates:
- **wayfinder-core** — library: AON Elasticsearch client, SQLite cache, search, rendering, domain types
- **wayfinder** (bin: `wf`) — CLI for searching/browsing AON data
- **waybuilder** (bin: `wb`) — TUI character builder

## Build & Test
```
cargo build --workspace
cargo clippy --workspace
cargo test --workspace
cargo run -p wayfinder -- search deity -f domain=Dragon
cargo run -p wayfinder -- show spell Fireball
cargo run -p wayfinder -- categories
cargo run -p wayfinder -- fields deity
cargo run -p wayfinder -- --sf2e search class
cargo run -p wayfinder -- --format json search spell --name Fireball
cargo run -p wayfinder -- cache fetch spell
cargo run -p wayfinder -- cache status
```

## Code Style
- `rustfmt` and `clippy` always, default settings
- Edition 2024, workspace dep inheritance
- Tests in per-crate `tests/` directories (not inline)
- Target ~100-120 lines per file; split into submodules when larger
- Use `cargo add` for new dependencies (ensures latest versions)

## Data Sources
- **PF2e**: `POST https://elasticsearch.aonprd.com/aon/_search` → index `aon70` (39k docs, 93 categories)
- **SF2e**: `POST https://elasticsearch.aonprd.com/aonsf/_search` → index `aonsf10` (6k docs, 51 categories)
- Category field is `keyword` type (use `term` queries, lowercase singular: `spell`, `deity`)

## CLI Output Formats
- `--format pretty` (default): colorized terminal with emoji, styled text
- `--format json`: raw JSON for piping/scripting
- `--format md`: raw AON markdown

## Key Modules
- `aon::client` — `AonClient` + `GameSystem` enum (PF2e/SF2e)
- `aon::query` — `SearchQuery` builder for Elasticsearch
- `aon::models` — `Document` serde struct with `#[serde(flatten)]` for unknown fields
- `aon::categories` — 93 known categories, grouped hierarchy, filterable fields, fuzzy suggestion
- `cache::store` — `CacheStore` SQLite layer with TTL
- `render::terminal` — AON HTML/markdown → `Vec<Span>` (no ANSI; consumers colorize)
- `search` — unified search (cache + client), category fetch
- `types` — domain enums (Rarity, Tradition, Size, Sanctification)

## Waybuilder Architecture

### Data Flow
`BuildChoices` (serializable user decisions) → `recalculate()` → `Character` (derived state).
All user state lives in `BuildChoices`. `Character` is recomputed from scratch on every change.

### Key Directories
- `model/` — domain types: `Character`, `Equipment` (Weapon/Armor/Shield with rune enums), `Abilities`, `Proficiencies`, `Spell`
- `build/choices.rs` — all user decisions (selections, boosts, feats, spells, equipment)
- `build/recalculate.rs` — stateless function: choices → character (HP, AC, attacks, saves, spells)
- `build/mechanics/` — AON document extractors: `ancestry.rs`, `background.rs`, `class.rs`, `equipment.rs`, `boosts.rs`, `skills.rs`, `subclass.rs`
- `build/rules/` — game math: `combat.rs` (AC/saves/attacks), `abilities.rs`, `hp.rs`, `proficiency.rs`
- `build/progression.rs` — level-by-level build slots (ancestry→heritage→class→feats→boosts)
- `ui/app.rs` — `App` state, `Modal` enum, `Screen`/`Focus`/`DetailTab` enums
- `ui/events.rs` — all keyboard input handling, modal apply logic, equipment cursor
- `ui/panels/` — render functions: `build_selector`, `character_info`, `detail_tabs` (Equipment/Spells/Skills/Feats)
- `ui/modal/` — `selection` (AON search), `boosts`, `skills`, `runes`, `info`, `text_input`
- `persistence/` — `save.rs` (JSON save/load), `export.rs` (Pathbuilder format)
- `data/` — async AON loader with channel-based communication to UI

### Equipment System
Equipment stats (damage_die, ac_bonus, dex_cap, weapon_category, etc.) are extracted from AON documents at selection time and stored on the model structs. No static lookup tables — all 483 weapons, 55 armor, 20 shields from AON are supported. Runes (potency, striking, resilient) are user-editable via the rune modal.

### AON Document Fields (via `doc.extra` HashMap)
- **Weapons**: `damage_die` (u64), `damage_type` (array of strings), `weapon_category`, `weapon_group`, `weapon_type` ("Melee"/"Ranged"), `hands`, `trait_raw` (array), `range`
- **Armor**: `ac` (i64), `armor_category`, `armor_group`, `dex_cap` (i64), `check_penalty`, `speed_penalty`, `strength`
- **Shields**: `ac` (i64), `hardness` (i64), `hp` (i64)

### Adding New AON Extractors
Follow the pattern in `build/mechanics/equipment.rs`: take `&Document`, read fields from `doc.extra`, return a model struct. Register in `mechanics/mod.rs`. Wire into `ui/events.rs` apply logic.

## Roadmap (Priority Order)

1. **Focus Spells & Focus Points** — track pool, grant from class/feats. Infrastructure exists (`focus_spells` on SpellCaster, `focusPoints` in export) but not wired.
2. **Class Feature Choice Resolution** — features shown in progression but many grant choices (druid orders, ranger edges) not tracked.
3. **Languages** — `Vec<String>` from ancestry + INT modifier. Trivial, high table utility.
4. **Property Runes** — UI for selecting from AON. Storage exists (`property_runes: Vec<String>`), needs search modal + damage integration.
5. **Spontaneous Casters** — sorcerer/bard spell repertoire (spells known) vs prepared. Different model needed.
6. **Conditions & Temporary Modifiers** — toggles for frightened, flat-footed, etc. Modify displayed stats.
7. **Special Senses & Movement Modes** — darkvision, fly/climb from ancestry/feats.
8. **Feat Prerequisite Validation** — gray out feats whose prereqs aren't met.

## References
`references/` (.gitignored) contains:
- `category_fields.json` — filterable fields per category
- `sample_documents.json` — exemplar docs for 14 key categories
- `explore_aon.py` — Python script for AON exploration
- `aon_search.js` — AON frontend search JS
- `aon_search_page.html` — AON search page HTML
- `aonsrd_homepage.html` — SF2E site homepage
- `sf2e_sample_class.json` — SF2E class exemplar
