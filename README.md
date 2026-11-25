# DISPEL® Game File Extractor

This toolkit reads and writes DISPEL® game files in binary, INI and DB formats. It includes a CLI, a desktop GUI with 46 editor types, a hex editor and a mod packaging pipeline. The tools write each edit into the game file. They do not keep a separate database.

## Scope and notice

This project reads and writes data files for DISPEL®. It ships no game content. You provide the files you edit.

DISPEL® is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark owner.

The source code is licensed under the MIT License (LICENSE). Game content is not covered by this license.

## What's in this repository

This Cargo workspace has six crates. The workspace uses Rust 2024 and version `0.9.5`.

| Crate                     | Type       | Role                                                            |
| ------------------------- | ---------- | --------------------------------------------------------------- |
| `dispel-extractor` (root) | lib + bin  | Core library (`dispel_core`) + CLI binary                       |
| `dispel-encodings`        | lib        | Shared text-encoding registry used by macros and core           |
| `dispel-gui`              | bin        | Iced desktop GUI with 46 editor types                           |
| `dispel-macros`           | proc-macro | 7 derive macros (Extractor, Localizable, RecordPatcher, etc.)   |
| `gui-widgets`             | lib        | Reusable Iced widgets (ContextMenu, modal, ParagraphCache)      |
| `hexedit`                 | lib + bin  | Full hex editor                                                 |

### Key source directories (`src/` in root crate)

| Directory             | Contents                                                            |
| --------------------- | ------------------------------------------------------------------- |
| `src/references/`     | 51 source files with parsers for `.db`, `.ini` and `.ref` files      |
| `src/map/`            | `.map` parser + isometric tile/sprite/event renderer                |
| `src/sprite.rs`       | `.spr` sprite and animation parser (RGB 565)                        |
| `src/snf.rs`          | `.snf` audio parser (PCM to WAV conversion)                         |
| `src/modding/`        | Mod authoring pipeline (apply, patch, package, conflict resolution) |
| `src/commands/`       | CLI command implementations (11 subcommands)                        |
| `src/queries/`        | 118 SQL files (59 schema DDL + 59 parameterized INSERTs)            |
| `src/localization.rs` | Text export and import for translations                             |
| `src/database.rs`     | SQLite schema bootstrap                                             |

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

### INI and text files

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
| `.gtl` / `.btl` | Tilesets (ground / roof, RGB 565, 32x32 tiles) |
| `.spr`          | Character sprites and animations (RGB 565)     |
| `.snf`          | Sound effects (custom PCM to WAV)              |
| `.dlg` / `.pgp` | Dialogue scripts and text (EUC-KR / 1250)      |
| `.scr`          | Event scripts (1250)                           |

Format specifications are in `docs/files/`. That directory holds 36 per-format documents and `CROSS_REFERENCES.md`.

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

### Extract sprites and animations

```bash
cargo run -- sprite fixtures/Dispel/CharacterInGame/M_BODY1.SPR --mode sprite
cargo run -- sprite fixtures/Dispel/CharacterInGame/M_BODY1.SPR --mode animation
```

### Convert sound files

```bash
cargo run -- sound to-wav fixtures/Dispel/Sound/sample.snf output.wav
cargo run -- sound from-wav output.wav sample.snf
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

### SQLite database import

```bash
cargo run -- database import fixtures/Dispel/ db.sqlite
```

### Mod packaging

The CLI `mod-pack` command reads `Event*.scr` files from the game `Ref/` folder and writes JSON.

```bash
cargo run -- mod-pack --help
```

### Full CLI reference

```bash
cargo run -- --help
```

## GUI

Launch the desktop editor:

```bash
cargo run -p dispel-gui
```

The GUI has these features:

- 46 editor types with spreadsheet views for supported game data formats
- A three-pane layout with a sidebar for the file tree, main content for editors and a history panel for undo and redo
- Undo and redo with an edit history for each editor
- Mod recording that tracks every field change for the mod changelog
- Full-text search with nucleo-matcher fuzzy search across loaded files
- A hex editor from the `hexedit` crate
- A map viewer with isometric rendering, a viewport and an LRU cache
- SNF audio playback with rodio
- An audio viewer for read-only OGG and WAV playback with timeline and volume
- A BMP viewer for read-only image preview with zoom and fit to viewport
- A mod packager that stores work in `.dispel-mods`. It writes a zip with `manifest.json`, `changes.json`, `patches/` and `files/`.

## Project structure

```
dispel-extractor/
├── src/                      # dispel-core library + CLI binary
│   ├── references/           # 51 game data file parsers
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

## Development

The canonical build, format, lint and test commands live in `scripts/ci-commands.sh`. The script has 11 commands. CI and the Makefile call that script. Use the script or the matching `make` target.

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
