# Extra.ini Documentation

> **DISPEL®** is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
> owner.

## File Information

### Overview

Text file that defines interactive-object definitions with visual assets, activation-frame settings, and descriptions.

### File Structure

**Location**: `Extra.ini`
**Encoding**: EUC-KR (Korean character encoding)
**Format**: CSV (Comma-Separated Values) with comments

### Format Specification

```ini
; Comment line explaining field structure
id,sprite_filename,activation_sprite_frame_mode,description
0,null,0,null
1,object1.spr,0,Storage object
2,object2.spr,1,Interactive object
...
```

### Field Definitions

| Field                        | Type   | Description                                                                                                   |
|------------------------------|--------|---------------------------------------------------------------------------------------------------------------|
| id                           | i32    | Unique interactive object identifier.                                                                         |
| sprite_filename              | string | SPR filename or "null".                                                                                       |
| activation_sprite_frame_mode | i32    | Selects the sprite frame used after activation for the object handlers that use this setting.                 |
| description                  | string | Editor-facing description or "null". This column is not loaded by the game into its runtime definition table. |

### Activation-Frame Mode
Every placed instance using the definition carries a copy of this field. For
object handlers 5, 6, and 8, values greater than `1` choose activated sprite
frame `1`; values `0` and `1` choose frame `0`. Observed files contain `0`,
`1`, and `2`, so this field is not a boolean or a quest flag.

### Text-Format Details

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

### Database Integration

- Processed by the `Extra` struct in `src/references/extra_ini.rs`
- Stored in database with all field mappings
- Linked to object placement files (Ext*.ref)
- Referenced by interaction and puzzle systems

### Object Type System

The codebase defines the structure for interactive objects:

```rust
pub struct Extra {
    pub id: i32,                           // Object ID
    pub sprite_filename: Option<String>,    // Visual asset
    pub activation_sprite_frame_mode: i32,  // Activation-frame selector
    pub description: Option<String>,        // Object description
}
```

### Usage

1. **Environment Interaction**: Defines objects players can interact with
2. **Visual Mapping**: Links object IDs to sprite files
3. **Activation visuals**: Selects an activated sprite frame for supported object handlers
4. **Puzzle Systems**: Objects used in environmental puzzles
5. **Map Placement**: Referenced by Ext*.ref placement files

### Notes

- File uses Windows-style line endings (`\r\n`)
- Descriptions use EUC-KR encoding for international characters
- Integrated with map placement and interaction systems

## Extractor

The file is parsed by the `Extra` struct in `src/references/extra_ini.rs`, which implements the `Extractor` trait.

### How to Run

```bash
# Extract Extra.ini to JSON
cargo run -- extract -i "Extra.ini"

# Import to SQLite database
cargo run -- database import "path/to/Dispel/" "database.sqlite"
```
