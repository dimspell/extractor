# CLI Redesign Plan: Extract + Patch Commands

## Overview

Redesign the CLI to support a **round-trip workflow**: extract game files to JSON, edit JSON, then patch game files back from JSON. The redesign focuses on:

1. **User-friendly command structure** — consistent, discoverable, self-documenting
2. **AI-generation friendly** — predictable patterns, structured output, explicit flags
3. **Patch/mod support** — write JSON back to binary/text game files using existing `save_file` implementations

---

## Design Principles

### Command Structure

```
dispel-extractor <action> <file-type> [options]
```

Where:

- **action**: `extract` | `patch` | `list` | `validate`
- **file-type**: auto-detected or explicitly specified
- **options**: `--input`, `--output`, `--format`, `--dry-run`, etc.

### Why this structure?

- **AI-friendly**: Actions are verbs, file types are nouns — predictable token patterns
- **User-friendly**: `extract` and `patch` are symmetric operations
- **Extensible**: New actions (`validate`, `diff`, `merge`) slot in naturally
- **Auto-detection**: File type inferred from extension or content when possible

---

## New Command Hierarchy

### Top-Level Commands

```
dispel-extractor
├── extract          # Read game file → output JSON/text
├── patch            # Read JSON/text → write game file
├── validate         # Validate JSON against file format schema
├── list             # List supported file types with descriptions
├── map              # Map-specific operations (tiles, atlas, render) [keep existing]
├── sprite           # Sprite extraction [keep existing]
├── sound            # Audio conversion [keep existing]
├── database         # SQLite operations [keep existing, refactor args]
└── test             # Test command [keep existing]
```

### `extract` Command

```bash
# Extract any supported reference file to JSON
dispel-extractor extract --input path/to/file.db --output output.json

# Extract to stdout (default when no --output)
dispel-extractor extract --input path/to/file.ini

# Pretty-print JSON (default for terminal, compact for pipes)
dispel-extractor extract --input file.db --output out.json --pretty

# Extract with auto-detection of file type
dispel-extractor extract --input Monster.db

# Extract specific type (override auto-detection)
dispel-extractor extract --input unknown_file --type monster
```

**Flags:**
| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--input` | `-i` | _(required)_ | Path to game file |
| `--output` | `-o` | stdout | Output file path |
| `--type` | `-t` | auto | File type override |
| `--pretty` | `-p` | auto | Pretty-print JSON |
| `--format` | `-f` | json | Output format (json) |

### `patch` Command

```bash
# Patch a game file from JSON
dispel-extractor patch --input modified.json --output path/to/file.db

# Patch in-place (with automatic backup)
dispel-extractor patch --input modified.json --target path/to/file.db --in-place

# Dry run: validate JSON without writing
dispel-extractor patch --input modified.json --target file.db --dry-run

# Patch with type override
dispel-extractor patch --input data.json --target file.db --type weapons
```

**Flags:**
| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--input` | `-i` | _(required)_ | Path to JSON file |
| `--target` | `-t` | _(required)_ | Path to game file to patch |
| `--output` | `-o` | same as target | Output path (if different from target) |
| `--type` | | auto | File type override |
| `--dry-run` | `-n` | false | Validate without writing |
| `--in-place` | | false | Patch target directly, create `.bak` backup |
| `--no-backup` | | false | Skip backup creation (with --in-place) |

### `validate` Command

```bash
# Validate JSON against file format
dispel-extractor validate --input data.json --type weapons

# Validate and show errors
dispel-extractor validate --input data.json --type monster --verbose
```

### `list` Command

```bash
# List all supported file types
dispel-extractor list

# List with machine-readable output (AI-friendly)
dispel-extractor list --format json

# List types matching a pattern
dispel-extractor list --filter "monster"
```

**JSON output structure (for AI consumption):**

```json
{
  "types": [
    {
      "name": "weapons",
      "description": "Weapons & armor database",
      "extensions": [".db"],
      "typical_paths": ["CharacterInGame/weaponItem.db"],
      "record_type": "WeaponItem",
      "fields": ["id", "name", "description", "base_price", ...]
    }
  ]
}
```

---

## File Type Registry

All 29 reference types with their auto-detection patterns:

| Type Key        | Extensions | Detection Pattern               | Read Function                         | Save Function             |
| --------------- | ---------- | ------------------------------- | ------------------------------------- | ------------------------- |
| `all_maps`      | .ini       | header comment + CSV format     | `all_map_ini::read_all_map_ini`       | `AllMapIni::save_file`    |
| `map_ini`       | .ini       | 9-column CSV with ref filenames | `map_ini::read_map_ini`               | `MapIni::save_file`       |
| `extra_ini`     | .ini       | extra object type definitions   | `extra_ini::read_extra_ini`           | `ExtraIni::save_file`     |
| `event_ini`     | .ini       | event/script mappings           | `event_ini::read_event_ini`           | `EventIni::save_file`     |
| `monster_ini`   | .ini       | monster visual refs             | `monster_ini::read_monster_ini`       | `MonsterIni::save_file`   |
| `npc_ini`       | .ini       | NPC visual refs                 | `npc_ini::read_npc_ini`               | `NpcIni::save_file`       |
| `wave_ini`      | .ini       | audio/SNF references            | `wave_ini::read_wave_ini`             | `WaveIni::save_file`      |
| `weapons`       | .db        | weapons and armour records      | `weapons_db::read_weapons_db`         | `WeaponItem::save_file`   |
| `monsters`      | .db        | monster attributes              | `monster_db::read_monster_db`         | `Monster::save_file`      |
| `magic`         | .db        | magic spell records             | `magic_db::read_magic_db`             | `Magic::save_file`        |
| `store`         | .db        | shop inventory records          | `store_db::read_store_db`             | `StoreItem::save_file`    |
| `misc_item`     | .db        | generic item records            | `misc_item_db::read_misc_item_db`     | `MiscItem::save_file`     |
| `heal_item`     | .db        | consumable records              | `heal_item_db::read_heal_item_db`     | `HealItem::save_file`     |
| `event_item`    | .db        | quest item records              | `event_item_db::read_event_item_db`   | `EventItem::save_file`    |
| `edit_item`     | .db        | modifiable item records         | `edit_item_db::read_edit_item_db`     | `EditItem::save_file`     |
| `party_level`   | .db        | EXP table records               | `party_level_db::read_party_level_db` | `PartyLevel::save_file`   |
| `party_ini`     | .db        | party NPC metadata              | `party_ini_db::read_party_ini_db`     | `PartyIniDb::save_file`   |
| `chdata`        | .db        | character data records          | `chdata_db::read_chdata`              | `ChData::save_file`       |
| `party_ref`     | .ref       | 8-column CSV, ghost_face        | `party_ref::read_part_refs`           | `PartyRef::save_file`     |
| `draw_item`     | .ref       | map placement records           | `draw_item::read_draw_items`          | `DrawItem::save_file`     |
| `npc_ref`       | .ref       | NPC placement records           | `npc_ref::read_npc_ref`               | `NpcRef::save_file`       |
| `monster_ref`   | .ref       | monster placement records       | `monster_ref::read_monster_ref`       | `MonsterRef::save_file`   |
| `extra_ref`     | .ref       | special object placements       | `extra_ref::read_extra_ref`           | `ExtraRef::save_file`     |
| `event_npc_ref` | .ref       | event NPC placements            | `event_npc_ref::read_event_npc_ref`   | `EventNpcRef::save_file`  |
| `dialog`        | .dlg       | dialogue script CSV             | `dialog::read_dialogs`                | `Dialog::save_file`       |
| `dialog_text`   | .pgp       | dialogue text package           | `dialogue_text::read_dialogue_texts`  | `DialogueText::save_file` |
| `quest`         | .scr       | quest definitions               | `quest_scr::read_quests`              | `Quest::save_file`        |
| `message`       | .scr       | diary game messages             | `message_scr::read_messages`          | `Message::save_file`      |

---

## Implementation Plan

### Phase 1: Core Infrastructure

#### 1.1 Create File Type Registry (`src/commands/registry.rs`)

```rust
pub struct FileType {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub extensions: &'static [&'static str],
    pub detect_fn: fn(&Path) -> bool,
    pub extract_fn: fn(&Path) -> Result<serde_json::Value, Box<dyn Error>>,
    pub patch_fn: fn(&serde_json::Value, &Path) -> Result<(), Box<dyn Error>>,
}
```

- Single source of truth for all file types
- Auto-detection by extension + content sniffing
- Maps each type to its extract/patch functions

#### 1.2 Create Unified Extract/Patch Command (`src/commands/unified.rs`)

```rust
pub struct ExtractCommand {
    input: PathBuf,
    output: Option<PathBuf>,
    file_type: Option<String>,
    pretty: bool,
}

pub struct PatchCommand {
    input: PathBuf,       // JSON file
    target: PathBuf,      // Game file to patch
    file_type: Option<String>,
    dry_run: bool,
    in_place: bool,
    no_backup: bool,
}
```

- Uses registry to resolve file type
- Handles JSON serialization/deserialization
- Implements backup logic for `--in-place`

#### 1.3 Create List/Validate Commands (`src/commands/info.rs`)

```rust
pub struct ListCommand {
    format: OutputFormat,  // Text | Json
    filter: Option<String>,
}

pub struct ValidateCommand {
    input: PathBuf,
    file_type: String,
    verbose: bool,
}
```

### Phase 2: Refactor main.rs

#### 2.1 New CLI Structure

```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract game file data to JSON
    Extract(ExtractArgs),
    /// Patch game files from JSON data
    Patch(PatchArgs),
    /// Validate JSON data against file format
    Validate(ValidateArgs),
    /// List supported file types
    List(ListArgs),
    /// Map operations (tiles, atlas, render)
    Map(MapArgs),           // Keep existing
    /// Sprite extraction
    Sprite(SpriteArgs),     // Keep existing, flatten to struct
    /// Audio conversion
    Sound(SoundArgs),       // Keep existing, flatten to struct
    /// Database operations
    Database(DatabaseArgs), // Keep existing
    /// Test command
    Test(TestArgs),         // Keep existing
}
```

#### 2.2 Remove Old RefCommands

- Remove `RefArgs` and `RefCommands` enum entirely
- Remove `ref_command.rs` or repurpose as registry builder
- Update `CommandFactory` to remove `create_ref_command`

### Phase 3: Backward Compatibility

#### 3.1 Aliases for Old Commands

Provide `--legacy` flag or alias system:

```bash
# Old command still works
dispel-extractor ref weapons file.db

# Maps to new command internally
dispel-extractor extract --input file.db --type weapons
```

#### 3.2 Migration Guide in `--help`

When user runs old-style commands, show migration hint:

```
Note: 'ref' command is deprecated. Use 'extract' instead:
  dispel-extractor extract --input file.db --type weapons
```

### Phase 4: AI-Generation Optimizations

#### 4.1 Structured JSON Output

All extract output includes metadata:

```json
{
  "_meta": {
    "file_type": "weapons",
    "record_count": 150,
    "version": 1,
    "fields": ["id", "name", "description", "base_price", ...]
  },
  "records": [
    { "id": 0, "name": "Short Sword", ... },
    ...
  ]
}
```

#### 4.2 JSON Schema Output

```bash
dispel-extractor schema --type weapons
```

Returns JSON Schema for validation — useful for AI tools generating patches.

#### 4.3 Patch Templates

```bash
dispel-extractor template --type weapons --id 5
```

Returns a minimal JSON template for a single record — useful for AI generating specific modifications.

---

## File Changes Summary

### New Files

| File                       | Purpose                                   |
| -------------------------- | ----------------------------------------- |
| `src/commands/registry.rs` | File type registry with auto-detection    |
| `src/commands/unified.rs`  | Extract and Patch command implementations |
| `src/commands/info.rs`     | List, Validate, Schema, Template commands |

### Modified Files

| File                          | Changes                                                 |
| ----------------------------- | ------------------------------------------------------- |
| `src/main.rs`                 | New CLI structure, remove RefCommands, add new commands |
| `src/commands/mod.rs`         | Add new modules, remove ref_command, update factory     |
| `src/commands/ref_command.rs` | **DELETE** or repurpose as registry builder             |

### Unchanged Files

| File                       | Reason                                                |
| -------------------------- | ----------------------------------------------------- |
| `src/commands/map.rs`      | Map operations are separate concern                   |
| `src/commands/sprite.rs`   | Sprite extraction is separate concern                 |
| `src/commands/sound.rs`    | Audio conversion is separate concern                  |
| `src/commands/database.rs` | Database import is separate concern                   |
| `src/commands/test.rs`     | Test command is separate concern                      |
| `src/commands/services.rs` | Service container unchanged                           |
| `src/references/*.rs`      | All parsers and `save_file` implementations unchanged |

---

## Example Workflows

### Workflow 1: Mod a weapon's stats

```bash
# 1. Extract weapons database
dispel-extractor extract -i fixtures/Dispel/CharacterInGame/weaponItem.db -o weapons.json

# 2. Edit weapons.json (manually or with AI)
# Change weapon at id=5: attack from 10 to 50

# 3. Validate the modified JSON
dispel-extractor validate -i weapons.json --type weapons

# 4. Patch the game file (creates weaponItem.db.bak)
dispel-extractor patch -i weapons.json -t fixtures/Dispel/CharacterInGame/weaponItem.db --in-place
```

### Workflow 2: Change map starting positions

```bash
# 1. Extract map configuration
dispel-extractor extract -i fixtures/Dispel/Ref/Map.ini -o map_config.json

# 2. Edit map_config.json to change spawn points

# 3. Write back
dispel-extractor patch -i map_config.json -t fixtures/Dispel/Ref/Map.ini --in-place
```

### Workflow 3: AI-assisted batch modding

```bash
# 1. Get schema for AI context
dispel-extractor schema --type monster --format json > monster_schema.json

# 2. Get list of all types for AI context
dispel-extractor list --format json > file_types.json

# 3. Extract data to modify
dispel-extractor extract -i Monster.db -o monsters.json

# 4. AI generates modified_monsters.json using schema

# 5. Validate AI output
dispel-extractor validate -i modified_monsters.json --type monster

# 6. Apply patch
dispel-extractor patch -i modified_monsters.json -t Monster.db --in-place
```

---

## Error Handling

### Extract Errors

- File not found → clear message with suggested paths
- Unknown file type → list closest matches
- Parse failure → line/record number + raw bytes context

### Patch Errors

- JSON validation failure → field-level error messages
- Record count mismatch → warning with option to proceed
- File permission denied → clear message
- Backup creation failure → abort with message

### Validation Errors

```json
{
  "valid": false,
  "errors": [
    {
      "record_index": 5,
      "field": "attack",
      "expected": "i16",
      "got": "string",
      "value": "high"
    }
  ]
}
```

---

## Notes

1. **All 29 file types already have `save_file` implemented** via the `Extractor` trait — no parser changes needed
2. The `Extractor` trait already defines the read/write contract: `read_file()` and `save_file()`
3. Binary formats (.db) use `BufWriter` with `byteorder` — already handles endianness correctly
4. Text formats (.ini, .ref, .dlg, .scr) use proper encoding (EUC-KR, WINDOWS-1250) — already handled
5. The `id` field is auto-generated during read (record index) — during patch, it should be ignored or validated
