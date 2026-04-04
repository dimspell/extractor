---
name: dispel-extractor
description: Helps to execute the dispel extractor CLI tool to extract, patch, and validate game data and assets for the Dispel game
---

# Dispel Extractor Implementation Skill

## Overview
This skill enables an AI agent to execute, debug, and validate the **Dispel Game File Extractor** (`dispel-extractor`) during implementation. It provides structured workflows for running the extractor, patching game files, and understanding its architecture.

---

## Purpose
- Automate the execution of `dispel-extractor` during development.
- Validate file parsing, extraction, and output generation.
- Patch (modify) game files from JSON for modding purposes.
- Provide debugging support for format compliance and error handling.
- Ensure adherence to the project's [legal guidelines](AGENTS.md).

---

## Running the Tool

### Build
```bash
cargo build --release
```

### Common Commands

```bash
# Extract game files to JSON (auto-detects type by filename)
cargo run -- extract -i <file> [-o <output.json>] [--pretty]
cargo run -- extract -i fixtures/Dispel/CharacterInGame/WeaponItem.db
cargo run -- extract -i fixtures/Dispel/MonsterInGame/Monster.db -o monsters.json
cargo run -- extract -i fixtures/Dispel/Map/cat1.map --pretty

# Extract with type override (for ambiguous files)
cargo run -- extract -i unknown_file --type weapons

# Patch game files from JSON
cargo run -- patch -i <input.json> -t <target-file> [--in-place] [--dry-run] [--no-backup]
cargo run -- patch -i weapons.json -t weaponItem.db --in-place
cargo run -- patch -i monsters.json -t Monster.db --dry-run

# Validate JSON against file format
cargo run -- validate -i <input.json> --type <type> [--verbose]
cargo run -- validate -i weapons.json --type weapons

# List supported file types
cargo run -- list
cargo run -- list --format json
cargo run -- list --filter monster

# Get JSON schema for a file type (AI-friendly)
cargo run -- schema --type weapons

# Get record template for a file type
cargo run -- template --type weapons [--pretty]

# Map operations
cargo run -- map tiles <input.gtl> [--output <dir>]
cargo run -- map atlas <input.gtl> <output.png>
cargo run -- map render --map <map> --btl <btl> --gtl <gtl> --output <output>
cargo run -- map from-db --map-id <id> --gtl-atlas <path> --btl-atlas <path> --output <out>
cargo run -- map to-db --database <db> --map <map.file>
cargo run -- map to-json --input <map.file> [--output <out.json>] [--pretty]
cargo run -- map sprites <input.map> [--output <dir>]

# Database operations
cargo run -- database import <game_path> <db_path>
cargo run -- database dialog-texts <game_path> <db_path>
cargo run -- database maps <game_path> <db_path>

# Sprite extraction
cargo run -- sprite <input.spr>
cargo run -- sprite <input.spr> --mode animation
cargo run -- sprite <input.spr> --info

# Extract sprite info via unified extract command (auto-detected by .spr extension)
cargo run -- extract -i <input.spr> [--pretty]

# Audio conversion
cargo run -- sound <input.snf> <output.wav>

# Test
cargo run -- test --message "hello"
```

### Deprecated Commands
The `ref` command is deprecated but still works with a migration hint:
```bash
# Old (deprecated, shows hint)
cargo run -- ref weapons fixtures/Dispel/CharacterInGame/weaponItem.db

# New (preferred)
cargo run -- extract -i fixtures/Dispel/CharacterInGame/weaponItem.db --type weapons
```

---

## Architecture

### Command Pattern (`src/commands/mod.rs`)
All commands implement the `Command` trait:
```rust
pub trait Command: Send + Sync {
    fn execute(&self) -> Result<(), Box<dyn Error>>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}
```

A `CommandFactory` provides dependency injection for creating commands.

### Module Structure

| Module | Purpose |
|--------|---------|
| `src/commands/mod.rs` | Command trait and factory |
| `src/commands/unified.rs` | Extract and Patch commands (unified reference file handling) |
| `src/commands/registry.rs` | File type registry with auto-detection (filename-based) |
| `src/commands/info.rs` | List, Validate, Schema, Template commands |
| `src/commands/map.rs` | Map parsing, rendering, and JSON export (`MapSubcommand` enum) |
| `src/commands/database.rs` | SQLite import/export (`DatabaseSubcommand` enum) |
| `src/commands/sprite.rs` | SPR sprite parsing |
| `src/commands/sound.rs` | SNF to WAV conversion |
| `src/map/` | Binary map format parsing, rendering, JSON serialization |
| `src/references/` | Game reference file parsers (with `Extractor` trait: `read_file` + `save_file`) |
| `src/queries/` | SQL templates for database operations |

### Supported File Formats

**Maps:** `.map`, `.gtl` (ground tiles), `.btl` (building tiles)
**Sprites:** `.spr` (static or animated)
**Audio:** `.snf` (raw PCM)
**Reference Files:** `.ini`, `.db`, `.ref`, `.dlg`, `.pgp`, `.scr`

### Supported Reference File Types (31 total)

| Type Key | Extensions | Description | Patchable |
|----------|-----------|-------------|-----------|
| `all_maps` | .ini | Master map list | Yes |
| `map_ini` | .ini | Map properties | Yes |
| `extra_ini` | .ini | Interactive object types | Yes |
| `event_ini` | .ini | Script/event mappings | Yes |
| `monster_ini` | .ini | Monster visual refs | Yes |
| `npc_ini` | .ini | NPC visual refs | Yes |
| `wave_ini` | .ini | Audio/SNF references | Yes |
| `weapons` | .db | Weapons & armor | Yes |
| `monsters` | .db | Monster stats | Yes |
| `magic` | .db | Magic spells | Yes |
| `store` | .db | Shop inventories | Yes |
| `misc_item` | .db | Generic items | Yes |
| `heal_item` | .db | Consumables | Yes |
| `event_item` | .db | Quest items | Yes |
| `edit_item` | .db | Modifiable items | Yes |
| `party_level` | .db | EXP tables | Yes |
| `party_ini` | .db | Party NPC metadata | Yes |
| `chdata` | .db | Character data | Yes |
| `party_ref` | .ref | Character definitions | Yes |
| `draw_item` | .ref | Map placements | Yes |
| `npc_ref` | .ref | NPC placements | Yes |
| `monster_ref` | .ref | Monster placements | Yes |
| `extra_ref` | .ref | Special object placements | Yes |
| `event_npc_ref` | .ref | Event-specific NPC placements | Yes |
| `dialog` | .dlg | Dialogue scripts | Yes |
| `dialog_text` | .pgp | Dialogue text packages | Yes |
| `quest` | .scr | Quest definitions | Yes |
| `message` | .scr | Game messages | Yes |
| `map_file` | .map | Map geometry, sprites, events, tiles | **No** |
| `gtl` | .gtl | Ground tile layer | **No** |
| `btl` | .btl | Building tile layer | **No** |
| `sprite` | .spr | Sprite/animation file | **No** |

### Auto-Detection

File types are detected by **filename** (case-insensitive). For example:
- `WeaponItem.db` → `weapons`
- `Monster.db` → `monsters`
- `Store.db` → `store`
- `cat1.map` → `map_file`
- `cat1.gtl` → `gtl`
- `PartyRef.ref` → `party_ref`

Use `--type <key>` to override auto-detection.

---

## Adding New Features

### Adding a new reference parser
1. Create parser struct in `src/references/` implementing `Extractor` trait (`read_file` + `save_file`)
2. Add `Deserialize` derive to the struct (required for patch support)
3. Add entry to `FILE_TYPES` in `src/commands/registry.rs` with `DetectKind` for auto-detection
4. Add SQL templates in `src/queries/` (optional, for database import)
5. No changes needed in `main.rs` — the registry handles routing automatically

### Adding a new map subcommand
1. Add variant to `MapCommands` enum in `src/main.rs`
2. Add variant to `MapSubcommand` in `src/commands/map.rs`
3. Implement in `src/map/` modules
4. Wire up in `main.rs` match statement

### Adding a new extract-only file type (no patch)
1. Add entry to `FILE_TYPES` in `src/commands/registry.rs`
2. Use `patch_not_supported` as the `patch_fn`
3. Implement a custom `extract_fn` that returns `serde_json::Value`

---

## Testing
```bash
cargo test
```

---

## Error Handling

| Error | Recovery |
|-------|----------|
| File not found | Check input path |
| Invalid format | Compare with known-good files |
| Parse error | Check binary structure against format docs |
| Database error | Check SQL syntax in `src/queries/` |
| JSON validation error | Check field types against schema |
| Patch file type mismatch | Use `--type` flag to override auto-detection |
| Patch not supported | File type is extract-only (e.g., `.map`, `.gtl`, `.btl`) |

---

## References
- **Extractor CLI**: `cargo run -- --help`
- **File Formats**: `docs/file_formats.md`
- **Cross-Reference Guide**: [CROSS_REFERENCES.md](docs/files/CROSS_REFERENCES.md)
- **INI Files**:
  - [AllMap.ini.md](docs/files/AllMap.ini.md)
  - [Event.ini.md](docs/files/Event.ini.md)
  - [Extra.ini.md](docs/files/Extra.ini.md)
  - [Npc.ini.md](docs/files/Npc.ini.md)
  - [Wave.ini.md](docs/files/Wave.ini.md)
- **Database Files**:
  - [CharacterInGame/](docs/files/CharacterInGame/)
    - [ChData.db.md](docs/files/CharacterInGame/ChData.db.md)
    - [EditItem.db.md](docs/files/CharacterInGame/EditItem.db.md)
    - [EventItem.db.md](docs/files/CharacterInGame/EventItem.db.md)
    - [HealItem.db.md](docs/files/CharacterInGame/HealItem.db.md)
    - [MiscItem.db.md](docs/files/CharacterInGame/MiscItem.db.md)
    - [Monster.ini.md](docs/files/CharacterInGame/Monster.ini.md)
    - [Store.db.md](docs/files/CharacterInGame/Store.db.md)
    - [WeaponItem.db.md](docs/files/CharacterInGame/WeaponItem.db.md)
  - [Ref/](docs/files/Ref/)
    - [DrawItem.db.md](docs/files/Ref/DrawItem.db.md)
    - [Magic.db.md](docs/files/Ref/Magic.db.md)
    - [Map.ini.md](docs/files/Ref/Map.ini.md)
    - [PartyRef.ref.md](docs/files/Ref/PartyRef.ref.md)
  - [MonsterInGame/](docs/files/MonsterInGame/)
    - [MondunMonmap.ref.md](docs/files/MonsterInGame/MondunMonmap.ref.md)
    - [Monster.db.md](docs/files/MonsterInGame/Monster.db.md)
  - [NpcInGame/](docs/files/NpcInGame/)
    - [DlgMapFiles.dlg.md](docs/files/NpcInGame/DlgMapFiles.dlg.md)
    - [Eventnpc.ref.md](docs/files/NpcInGame/Eventnpc.ref.md)
    - [NpcMapFiles.ref.md](docs/files/NpcInGame/NpcMapFiles.ref.md)
    - [PgpMapFiles.pgp.md](docs/files/NpcInGame/PgpMapFiles.pgp.md)
    - [PrtLevel.db.md](docs/files/NpcInGame/PrtLevel.db.md)
  - [ExtraInGame/](docs/files/ExtraInGame/)
    - [Extra.ref.md](docs/files/ExtraInGame/Extra.ref.md)
    - [Message.scr.md](docs/files/ExtraInGame/Message.scr.md)
    - [Quest.scr.md](docs/files/ExtraInGame/Quest.scr.md)
  - [Map/](docs/files/Map/)
    - [Map.btl.md](docs/files/Map/Map.btl.md)
    - [Map.gtl.md](docs/files/Map/Map.gtl.md)
    - [Map.map.md](docs/files/Map/Map.map.md)
    - [MapModule.md](docs/files/Map/MapModule.md)
    - [Sprites.spr.md](docs/files/Map/Sprites.spr.md)

---

## Legal Compliance Checklist
- [ ] Test files are synthetic or documented
- [ ] Outputs use generic identifiers (e.g., `monster_id` not `"Goblin"`)
- [ ] No copyrighted content in outputs
- [ ] Focus on technical specs, not creative works
- [ ] Patch operations only modify game files for personal modding — no distribution of modified assets

See [AGENTS.md](AGENTS.md) for full guidelines.
