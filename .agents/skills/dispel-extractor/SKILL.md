---
name: dispel-extractor
description: Helps to execute the dispel extractor CLI tool to extract data and assets from the Dispel game
---

# Dispel Extractor Implementation Skill

## Overview
This skill enables an AI agent to execute, debug, and validate the **Dispel Game File Extractor** (`dispel-extractor`) during implementation. It provides structured workflows for running the extractor and understanding its architecture.

---

## Purpose
- Automate the execution of `dispel-extractor` during development.
- Validate file parsing, extraction, and output generation.
- Provide debugging support for format compliance and error handling.
- Ensure adherence to the project's [legal guidelines](AGENTS.md).

---

## Running the Tool

### Build
```bash
cargo build --release
```

### Run with Debug
```bash
cargo run -- -vvv map tiles input.gtl
```

### Common Commands

```bash
# Map operations
cargo run -- map tiles <input.gtl> [--output <dir>]
cargo run -- map atlas <input.gtl> <output.png>
cargo run -- map render --map <map> --btl <btl> --gtl <gtl> --output <output>
cargo run -- map from-db --map-id <id> --gtl-atlas <path> --btl-atlas <path> --output <out>
cargo run -- map to-db --database <db> --map <map.file>

# Reference files to JSON
cargo run -- ref monster <Monster.ini>
cargo run -- ref weapons <weaponItem.db>
cargo run -- ref npc <Npc.ini>
cargo run -- ref dialog <file.dlg>
cargo run -- ref map <Map.ini>
cargo run -- ref all-maps <AllMap.ini>

# Database operations
cargo run -- database import <game_path> <db_path>
cargo run -- database dialog-texts <game_path> <db_path>
cargo run -- database maps <game_path> <db_path>

# Sprite extraction
cargo run -- sprite <input.spr>
cargo run -- sprite <input.spx> --mode animation

# Audio conversion
cargo run -- sound <input.snf> <output.wav>

# Test
cargo run -- test --message "hello"
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
| `src/commands/map.rs` | Map parsing and rendering (`MapSubcommand` enum) |
| `src/commands/ref_command.rs` | INI/DB/REF file parsers (`RefSubcommand` enum) |
| `src/commands/database.rs` | SQLite import/export (`DatabaseSubcommand` enum) |
| `src/commands/sprite.rs` | SPR/SPX sprite parsing |
| `src/commands/sound.rs` | SNF to WAV conversion |
| `src/map/` | Binary map format parsing |
| `src/references/` | Game reference file parsers |
| `src/queries/` | SQL templates for database operations |

### Supported File Formats

**Maps:** `.map`, `.gtl` (ground tiles), `.btl` (building tiles)
**Sprites:** `.spr` (static), `.spx` (animated)
**Audio:** `.snf` (raw PCM)
**Reference Files:** `.ini`, `.db`, `.ref`, `.dlg`, `.pgp`, `.scr`

---

## Adding New Features

### Adding a new reference parser
1. Add variant to `RefCommands` enum in `src/main.rs`
2. Add variant to `RefSubcommand` in `src/commands/ref_command.rs`
3. Create parser struct in `src/references/`
4. Add SQL templates in `src/queries/`
5. Wire up in `main.rs` match statement

### Adding a new map subcommand
1. Add variant to `MapCommands` enum in `src/main.rs`
2. Add variant to `MapSubcommand` in `src/commands/map.rs`
3. Implement in `src/map/` modules
4. Wire up in `main.rs` match statement

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
    - [Sprites.spr.md](docs/files/Map/Sprites.spr.md)

---

## Legal Compliance Checklist
- [ ] Test files are synthetic or documented
- [ ] Outputs use generic identifiers (e.g., `monster_id` not `"Goblin"`)
- [ ] No copyrighted content in outputs
- [ ] Focus on technical specs, not creative works

See [AGENTS.md](AGENTS.md) for full guidelines.
