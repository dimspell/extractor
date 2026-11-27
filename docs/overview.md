# Dispel Extractor overview

This toolkit reads and writes DISPEL® game files. It is a Rust workspace with six crates. The workspace has a CLI for extraction and patching, a desktop GUI for editing game data and a hex editor. The tools write edits into the game files. They do not keep a separate database.

## Project structure

The workspace has six crates:

| Crate | Type | Role |
|---|---|---|
| `dispel-extractor` (root) | lib + bin | Core library (`dispel_core`) + CLI binary |
| `dispel-encodings` | lib | Single source of truth for text-encoding spellings and labels |
| `dispel-gui` | bin | Iced 0.15 desktop GUI with 46 editor types, built on the `dimspell/iced` fork pinned at rev `c0ffdd8e7352df9283496f3db334731c00ef6d0d` |
| `dispel-macros` | proc-macro | 7 derive macros (Extractor, Localizable, RecordPatcher, etc.) |
| `gui-widgets` | lib | Reusable Iced widgets (ContextMenu, modal, ParagraphCache) |
| `hexedit` | lib + bin | Hex editor |

### Key source directories (`src/`)

- `src/main.rs`: CLI entry point with `clap`
- `src/cli.rs`: subcommand definitions
- `src/database.rs`: SQLite schema bootstrap
- `src/localization.rs`: text export and import for translations
- `src/references/`: 51 game data file parsers (`.db`, `.ini`, `.ref`)
- `src/map/`: `.map` parser and isometric renderer for tiles, sprites and events
- `src/modding/`: mod authoring pipeline (apply, patch, package, resolve conflicts)
- `src/commands/`: CLI command implementations
- `src/queries/`: 118 SQL files (59 schema DDL + 59 INSERTs)
- `src/sprite.rs`: `.spr` sprite and animation parser
- `src/snf.rs`: `.snf` audio parser

## CLI commands

```
# Extract game data to JSON
cargo run -- extract -i "Monster.db"

# Patch game files from JSON
cargo run -- patch -i changes.json -t "Monster.db" --in-place

# Validate JSON against an expected schema
cargo run -- validate -i weapons.json --type weapons

# List supported file types
cargo run -- list

# Generate a JSON schema for a file type
cargo run -- schema --type weapons

# Generate a minimal JSON template
cargo run -- template --type weapons --pretty

# Extract sprites or animations
cargo run -- sprite "M_BODY1.SPR" --mode sprite

# Convert SNF audio to WAV and WAV audio to SNF
cargo run -- sound to-wav sample.snf output.wav
cargo run -- sound from-wav output.wav sample.snf

# Map operations (tiles, atlas, render)
cargo run -- map tiles "cat1.gtl" --output out/tiles/
cargo run -- map atlas "cat1.btl" cat1_atlas.png
cargo run -- map render --map cat1.map --btl cat1.btl --gtl cat1.gtl --output map.png

# SQLite database import
cargo run -- database import "path/to/Dispel/" db.sqlite

# Batch-export event scripts to JSON
cargo run -- mod-pack --game-path "path/to/Dispel" --output mod-pack/
```

Full subcommand reference: `cargo run -- --help`.

## Workflow

1. Parsing: The tools read binary and INI files with `byteorder` and encoding-aware text readers. The readers use `WINDOWS-1250` or `EUC-KR`.
2. Conversion: The tools convert RGB 565 colors, PCM audio and tile indexes to other representations.
3. GUI editing: The GUI has 46 editor types and a fallback for unknown file types. It supports undo, redo and mod recording.
4. Mod packaging: The CLI `mod-pack` command exports event scripts to JSON. The GUI mod packager writes a zip with `manifest.json`, `changes.json`, `patches/` and `files/`. It stores work in `.dispel-mods`.
5. Output: The tools write PNG images, WAV audio and extracted JSON. They can also patch files in place.
