# Extra.ini Documentation

## File Information

### Overview

Text file that defines interactive-object definitions with visual assets, activation-frame settings, and descriptions.

### File Structure

**Location**: `Extra.ini`
**Encoding**: EUC-KR (Korean character encoding)
**Format**: CSV (Comma-Separated Values) with comments
**Total Entries**: 182 interactive object definitions

### Format Specification

```ini
; Comment line explaining field structure
id,sprite_filename,activation_sprite_frame_mode,description
0,null,0,null
1,object1.spr,0,Object description
2,object2.spr,1,Special object description
...
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| id | i32 | Unique interactive object identifier. |
| sprite_filename | string | SPR filename or "null" |
| activation_sprite_frame_mode | i32 | Selects the sprite frame used after activation for the object handlers that use this setting. |
| description | string | Editor-facing description or "null". The executable loader does not copy this fourth column into its runtime definition table. |

### Activation-frame mode

The game loads this field into the runtime object definition and copies it to
each placed instance. In the observed activation path for object handlers 5, 6,
and 8, values greater than `1` choose activated sprite frame `1`; values `0`
and `1` choose frame `0`. The shipped file contains `0`, `1`, and `2`.

The executable does not support interpreting this column as a standard/special
or quest flag.

### Text-format details

- **"null"**: Literal string indicating no sprite or description
- **;**: Lines starting with semicolon are comments
- **Empty lines**: Ignored during processing

### Example Format

```ini
; Default entry
0,null,0,null

; Container template
1,container.spr,0,Storage object
2,container.spr,2,Loot container

; Object using the alternate activated sprite frame
10,special.spr,2,Interactive object
11,unique.spr,1,Interactive object
```

### Technical Details

**Encoding**: EUC-KR (Extended Unix Code Korea)
- Supports Korean characters in descriptions
- Requires proper encoding handling for reading/writing

**File Processing**:
- Comments (lines starting with ";") are ignored
- Empty lines are skipped
- CSV format with comma delimiter
- "null" literal used for missing fields

**Database Integration**:
- Processed by `Extra` struct in the codebase
- Stored in database with all field mappings
- Linked to object placement files (Ext*.ref)
- Referenced by interaction and puzzle systems

### Object Type System

The codebase defines the structure for interactive objects:

```rust
pub struct Extra {
    id: i32,                    // Object ID
    sprite_filename: Option<String>, // Visual asset
    activation_sprite_frame_mode: i32, // Activation-frame selector (0, 1, or 2 in shipped data)
    description: Option<String>, // Object description
}
```

### Usage in Game

1. **Environment Interaction**: Defines objects players can interact with
2. **Visual Mapping**: Links object IDs to sprite files
3. **Activation visuals**: Selects an activated sprite frame for supported object handlers
4. **Puzzle Systems**: Objects used in environmental puzzles
5. **Map Placement**: Referenced by Ext*.ref placement files

### Object Function Analysis

**Container System:**
- IDs 1-5: Various chest and storage types
- Linked to loot and inventory systems
- Standard interaction patterns

**Navigation System:**
- Doors, ladders, ropes for movement
- Teleportation objects for fast travel
- Special transition objects

**Information System:**
- Signs and markers provide guidance
- Shop signs indicate services
- Quest objects provide story context

### File Characteristics

- **Fixture entry count**: 151 object definitions
- **Fixture ID range**: 0-150 (ID 0 = default/null entry)
- **Activation-frame modes**: Values 0, 1, and 2 occur in the shipped data
- **Comment Organization**: Logical grouping by object type
- **Encoding**: EUC-KR with Korean descriptions

### Notes

- File uses Windows-style line endings (\r\n)
- Comments are in Polish and Korean (mixed encoding)
- Descriptions use EUC-KR encoding for international characters
- Integrated with map placement and interaction systems
- **No copyrighted game content** is reproduced or distributed

## Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game assets, sprite files, or proprietary artwork. All references to interactive objects are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

## Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any SPR/SPX files or game artwork
- Focuses on **technical organization and interaction systems**
- Uses **generic examples** of object structures
- Maintains **nominal fair use** for trademark references

## Extractor

An extractor is available in `src/references/extra_ini.rs` to parse this file format.

### How to Run

```bash
# Extract Extra.ini to JSON
cargo run -- extract -i "fixtures/Dispel/Extra.ini"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
