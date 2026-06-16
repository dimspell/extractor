# Dispel Extractor Overview

A modding toolkit for **DISPEL®** game files. Written in Rust as a 5-crate Cargo workspace, this project provides a CLI for extraction/patching, a desktop GUI for editing game data, and a hex editor with Lua scripting. All edits are persisted directly to game files — no intermediate database.

**Not affiliated with, endorsed by, or sponsored by the trademark owner.**

---

## Project Structure

The workspace consists of five crates:

| Crate | Type | Role |
|---|---|---|
| `dispel-extractor` (root) | lib + bin | Core library (`dispel_core`) + CLI binary |
| `dispel-gui` | bin | Iced 0.14 desktop GUI |
| `dispel-macros` | proc-macro | 5 derive macros (Extractor, Localizable, RecordPatcher, etc.) |
| `gui-widgets` | lib | Reusable Iced widgets (ContextMenu, modal, ParagraphCache) |
| `hexedit` | lib + bin | Hex editor with Lua scripting |

### Key source directories (`src/`)

- **`src/main.rs`** — CLI entry point via `clap`
- **`src/cli.rs`** — Subcommand definitions
- **`src/database.rs`** — SQLite schema bootstrap
- **`src/localization.rs`** — Text export/import for translations
- **`src/references/`** — 30+ game data file parsers (`.db`, `.ini`, `.ref`)
- **`src/map/`** — `.map` parser + isometric renderer (tiles, sprites, events)
- **`src/modding/`** — Mod authoring pipeline (apply, patch, package, resolve conflicts)
- **`src/commands/`** — CLI command implementations
- **`src/queries/`** — 70 SQL files (schema DDL + INSERTs)
- **`src/sprite.rs`** — `.spr` sprite/ animation parser
- **`src/snf.rs`** — `.snf` audio parser

---

## CLI Commands

```
# Extract game data to JSON
cargo run -- extract -i "Monster.db"

# Patch game files from JSON
cargo run -- patch -i changes.json -t "Monster.db" --in-place

# Validate JSON against expected schema
cargo run -- validate -i weapons.json --type weapons

# List supported file types
cargo run -- list

# Generate JSON schema for a file type
cargo run -- schema --type weapons

# Generate a minimal JSON template
cargo run -- template --type weapons --pretty

# Extract sprites or animations
cargo run -- sprite "M_BODY1.SPR" --mode sprite

# Convert SNF audio to WAV
cargo run -- sound --input sample.snf --output output.wav

# Map operations (tiles, atlas, render)
cargo run -- map tiles "cat1.gtl" --output out/tiles/
cargo run -- map atlas "cat1.btl" cat1_atlas.png
cargo run -- map render --map cat1.map --btl cat1.btl --gtl cat1.gtl --output map.png

# Dialogue parsing
cargo run -- dialog "DlgMapFiles.dlg"

# SQLite database import
cargo run -- database import "path/to/Dispel/" db.sqlite

# Mod packaging
cargo run -- mod-pack ...
```

Full subcommand reference: `cargo run -- --help`.

---

## Workflow

1. **Parsing** — Binary/INI formats are read using `byteorder` and encoding-aware text readers (`WINDOWS-1250` or `EUC-KR`).
2. **Processing** — Raw data (RGB 565 colors, PCM audio, tile indexes) is converted to usable representations.
3. **Editing** — The GUI provides 38+ editor types for viewing and modifying records, with undo/redo, mod recording, and auto-save drafts.
4. **Modding** — Changes can be packaged into `.dispel-mod` archives for distribution, with conflict detection and binary patching.
5. **Output** — Images (PNG), audio (WAV), JSON (extraction), or direct in-place file patching.
