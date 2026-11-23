# DISPEL® Game File Extractor

A modding toolkit for the **DISPEL®** RPG game. Reads and writes game files in native binary, INI, and DB formats. Includes a CLI, a desktop GUI with 45 editor types, a hex editor, and a full mod packaging pipeline. All edits are persisted directly to game files — no intermediate database.

**Not affiliated with, endorsed by, or sponsored by the trademark owner.**

---

## What's in this repository

This is a 6-crate Cargo workspace (Rust 2024, version `0.9.3`):

| Crate                     | Type       | Role                                                            |
| ------------------------- | ---------- | --------------------------------------------------------------- |
| `dispel-extractor` (root) | lib + bin  | Core library (`dispel_core`) + CLI binary                       |
| `dispel-encodings`        | lib        | Shared text-encoding registry used by macros and core           |
| `dispel-gui`              | bin        | Iced desktop GUI with 45 editor types                           |
| `dispel-macros`           | proc-macro | 7 derive macros (Extractor, Localizable, RecordPatcher, etc.)   |
| `gui-widgets`             | lib        | Reusable Iced widgets (ContextMenu, modal, ParagraphCache)      |
| `hexedit`                 | lib + bin  | Full hex editor                                                 |

### Key source directories (`src/` in root crate)

| Directory             | Contents                                                            |
| --------------------- | ------------------------------------------------------------------- |
| `src/references/`     | 30+ parsers for `.db`, `.ini`, `.ref` game data files               |
| `src/map/`            | `.map` parser + isometric tile/sprite/event renderer                |
| `src/sprite.rs`       | `.spr` sprite and animation parser (RGB 565)                        |
| `src/snf.rs`          | `.snf` audio parser (PCM → WAV conversion)                          |
| `src/modding/`        | Mod authoring pipeline (apply, patch, package, conflict resolution) |
| `src/commands/`       | CLI command implementations (12 subcommands)                        |
| `src/queries/`        | 118 SQL files (59 schema DDL + 59 parameterized INSERTs)            |
| `src/localization.rs` | Text export/import for translations                                 |
| `src/database.rs`     | SQLite schema bootstrap                                             |

---

## Supported file formats

### Binary DB files (`.db`)

| File               | Description             |
| ------------------ | ----------------------- |
| `WeaponItem.db`    | Weapons                 |
| `HealItem.db`      | Healing items           |
| `EditItem.db`      | Edit items              |
| `EventItem.db`     | Event items             |
| `MiscItem.db`      | Misc items              |
| `Monster.db`       | Monster combat stats    |
| `Magic.db`         | Magic spells            |
| `Store.db`         | Shop inventories        |
| `ChData.db`        | Character leveling data |
| `DrawItem.db`      | Draw items              |
| `PartyLevelNpc.db` | Party level NPC data    |

### INI / text files

| File          | Description                |
| ------------- | -------------------------- |
| `AllMap.ini`  | Map list and configuration |
| `Map.ini`     | Per-map configuration      |
| `Monster.ini` | Monster visual data        |
| `Npc.ini`     | NPC visual data            |
| `Event.ini`   | Event definitions          |
| `Extra.ini`   | Extra entity data          |
| `Wave.ini`    | Wave definitions           |
| `Npc.ini`     | NPC data                   |

### Reference files (`.ref`)

| File          | Description             |
| ------------- | ----------------------- |
| `Mondun*.ref` | Monster placements      |
| `Npccat*.ref` | NPC placements          |
| `Extdun*.ref` | Extra entity placements |

### Other formats

| Extension       | Description                                    |
| --------------- | ---------------------------------------------- |
| `.map`          | Map geometry (tiles, sprites, events)          |
| `.gtl` / `.btl` | Tilesets (ground / roof, RGB 565, 32×32 tiles) |
| `.spr`          | Character sprites and animations (RGB 565)     |
| `.snf`          | Sound effects (custom PCM → WAV)               |
| `.dlg` / `.pgp` | Dialogue scripts and text (EUC-KR / 1250)      |
| `.scr`          | Event scripts (1250)                           |

Full format specs are in `docs/files/` (36 per-format documents plus `CROSS_REFERENCES.md`).

---

## CLI usage

Build the project first:

```bash
cargo build --release
```

### Extract game data to JSON

```bash
# Extract a specific file
cargo run -- extract -i fixtures/Dispel/Monster.ini
cargo run -- extract -i fixtures/Dispel/CharacterInGame/weaponItem.db --pretty
```

### Patch game files from JSON

```bash
cargo run -- patch -i changes.json -t fixtures/Dispel/CharacterInGame/weaponItem.db --in-place
```

### Validate JSON against schema

```bash
cargo run -- validate -i weapons.json --type weapons
```

### List supported file types

```bash
cargo run -- list
cargo run -- list --filter monster
```

### Generate JSON schema or template

```bash
cargo run -- schema --type weapons
cargo run -- template --type weapons --pretty
```

### Sprite and animation extraction

```bash
cargo run -- sprite fixtures/Dispel/CharacterInGame/M_BODY1.SPR --mode sprite
cargo run -- sprite fixtures/Dispel/CharacterInGame/M_BODY1.SPR --mode animation
```

### Sound conversion

```bash
cargo run -- sound --input fixtures/Dispel/Sound/sample.snf --output output.wav
```

### Map operations

```bash
# Render a map to PNG
cargo run -- map render --map fixtures/Dispel/Map/cat1.map \
    --btl fixtures/Dispel/Map/cat1.btl \
    --gtl fixtures/Dispel/Map/cat1.gtl \
    --output map_render.png

# Extract tiles from a tileset
cargo run -- map tiles fixtures/Dispel/Map/cat1.gtl --output out/tiles/

# Generate a sprite atlas
cargo run -- map atlas fixtures/Dispel/Map/cat1.btl cat1_atlas.png

# Extract sprites used in a map
cargo run -- map sprites fixtures/Dispel/Map/cat1.map --output out/cat1_sprites/
```

### Dialogue parsing

```bash
cargo run -- dialog fixtures/Dispel/Map/DlgMapFiles.dlg
```

### SQLite database import

```bash
cargo run -- database import fixtures/Dispel/ db.sqlite
```

### Mod packaging

```bash
cargo run -- mod-pack --help
```

### Full CLI reference

```bash
cargo run -- --help
```

---

## GUI

Launch the desktop editor:

```bash
cargo run -p dispel-gui
```

The GUI provides:

- **45 editor types** — spreadsheet views for every supported game data format
- **3-pane layout** — sidebar (file tree), main content (editors), history panel (undo/redo)
- **Undo/redo** with full edit history per editor
- **Mod recording** — tracks every field change for mod changelog generation
- **Full-text search** — nucleo-matcher fuzzy search across all loaded files
- **Hex editor** — built-in hex editor (from `hexedit` crate)
- **Map viewer** — isometric rendering with viewport + LRU cache
- **SNF audio playback** — rodio-based sound preview
- **DbViewer** — SQLite browser for imported databases
- **Mod packager** — package changes into `dispel-mod` archives

---

## Project structure

```
dispel-extractor/
├── src/                      # dispel-core library + CLI binary
│   ├── references/           # 30+ game data file parsers
│   ├── map/                  # .map parser + isometric renderer
│   ├── modding/              # Mod authoring pipeline
│   ├── commands/             # CLI command implementations
│   ├── queries/              # 118 SQL files (59 DDL + 59 INSERTs)
│   ├── sprite.rs             # .spr parser
│   ├── snf.rs                # .snf audio parser
│   ├── localization.rs       # Text export/import
│   ├── database.rs           # SQLite schema bootstrap
│   ├── cli.rs                # clap subcommand definitions
│   └── main.rs               # CLI entry point
├── dispel-encodings/         # Shared text-encoding registry
├── dispel-gui/               # Iced desktop GUI (binary only)
├── dispel-macros/            # Proc-macro crate (7 derive macros)
├── gui-widgets/              # Reusable Iced widgets library
├── hexedit/                  # Hex editor library + standalone binary
├── docs/                     # Documentation (overview, format specs, cross-references)
├── fixtures/                 # Local-only, git-ignored test game files
├── Makefile                  # Dev targets (build, test, extract, render)
└── scripts/
    ├── ci-commands.sh        # Canonical build / fmt / clippy / test commands
    └── release.sh            # Automated release script
```

---

## Development

The canonical build, format, lint and test commands live in
`scripts/ci-commands.sh` (11 commands). CI and the Makefile delegate to that
script, so use it or the matching `make` target rather than an inline command:

```bash
bash scripts/ci-commands.sh --list   # show all canonical commands
bash scripts/ci-commands.sh build
bash scripts/ci-commands.sh test
bash scripts/ci-commands.sh clippy
bash scripts/ci-commands.sh fmt
```

```bash
make build        # same as scripts/ci-commands.sh build
make cargo_test   # same as scripts/ci-commands.sh test (software Iced backend)
make clippy       # same as scripts/ci-commands.sh clippy
make fmt          # same as scripts/ci-commands.sh fmt
```

### Run the GUI

```bash
cargo run -p dispel-gui
```

### Makefile targets

| Target                        | Description                   |
| ----------------------------- | ----------------------------- |
| `make build`                  | Build all crates              |
| `make cargo_test`             | Run all tests                 |
| `make iced_test`              | Run Iced UI simulation tests  |
| `make clippy`                 | Run linter                    |
| `make fmt`                    | Format all code               |
| `make extract-file FILE=path` | Extract a single file to JSON |
| `make map-render map_id=cat1` | Render a map to PNG           |
| `make database-import`        | Import game data to SQLite    |
| `make sound`                  | Convert SNF to WAV            |
| `make sprite-sprite`          | Extract sprite sheet          |
| `make sprite-animation`       | Extract animations            |
| `make mod-pack`               | Package a mod                 |
| `make help`                   | Show all Makefile targets     |

---

## Legal

This project is **not affiliated with, endorsed by, or sponsored by** the DISPEL® trademark owner.

This tool is for **independent file-format interoperability research based on observable program behavior** only. It does not redistribute copyrighted game files and does not circumvent technological protection measures.

### License

The source code is licensed under the **[MIT License](LICENSE)**. Game content, assets, and proprietary formats are not covered by this license.
