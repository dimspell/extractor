# Extra.ini documentation

## File information

### Overview

The tool reads `Extra.ini` as a CSV file with comments. Each row defines an interactive object with a sprite filename, an activation-frame mode and a description.

### File structure

- **Location**: `Extra.ini`
- **Encoding**: EUC-KR (Korean character encoding)
- **Format**: CSV (Comma-Separated Values) with comments

### Format specification

```ini
; Comment line explaining field structure
id,sprite_filename,activation_sprite_frame_mode,description
0,null,0,null
1,object1.spr,0,Storage object
2,object2.spr,1,Interactive object
...
```

### Field definitions

| Field                        | Type   | Description                                                                                                   |
|------------------------------|--------|---------------------------------------------------------------------------------------------------------------|
| id                           | i32    | Unique interactive object identifier.                                                                         |
| sprite_filename              | string | SPR filename or "null".                                                                                       |
| activation_sprite_frame_mode | i32    | Selects the sprite frame used after activation for the object handlers that use this setting.                 |
| description                  | string | Editor-facing description or "null". This column is not loaded by the game into its runtime definition table. |

### Activation-frame mode

The placement records in `Ext*.ref` carry a copy of this value. For object handlers 5, 6 and 8, values greater than `1` select sprite frame `1`. Values `0` and `1` select frame `0`. Observed files contain `0`, `1` and `2`.

### Text-format details

- **"null"**: The tool reads this string when no sprite or description is present.
- **`;`**: Lines starting with this mark are comments.
- **Empty lines**: The parser ignores them.

### Example format

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

### Technical details

**Encoding**: EUC-KR (Extended Unix Code Korea)

- The encoding covers Korean characters in descriptions.
- The reader and writer must use this encoding.

**File processing:**

- The parser ignores comment lines that start with `;`.
- It skips empty lines.
- It splits rows on commas.
- It reads `null` as a missing field.

### Database integration

The `Extra` struct in `src/references/extra_ini.rs` parses the file. The tool stores every field in the database. Object placement files (`Ext*.ref`) reference these IDs.

### Object type system

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

1. The tool defines objects with an ID and a sprite filename.
2. It links object IDs to sprite files.
3. It selects an activated sprite frame for supported handlers.
4. Environmental puzzles can use these objects.
5. `Ext*.ref` placement files reference the IDs.

### Notes

- The file uses Windows-style line endings (`\r\n`).
- Descriptions use EUC-KR encoding for international characters.
- The tool links these definitions to map placement files.

## Extractor

The `Extra` struct in `src/references/extra_ini.rs` parses the file. It implements the `Extractor` trait.

### How to run

```bash
# Extract Extra.ini to JSON
cargo run -- extract -i "Extra.ini"

# Import to SQLite database
cargo run -- database import "path/to/Dispel/" "database.sqlite"
```
