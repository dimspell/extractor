# Npc.ini Documentation

> **DISPEL®** is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
> owner.

## File Information

### Overview

Text file that defines visual appearances and descriptions for NPC (Non-Player Character) types in the game.

### File Structure

**Location**: `Npc.ini`
**Encoding**: EUC-KR (Korean character encoding)
**Format**: CSV (Comma-Separated Values) with comments

### Format Specification

```ini
; Comment line
id,sprite_filename,description
1,guard.spr,City Guard
2,merchant.spr,Shopkeeper
...
```

### Field Definitions

| Field           | Type   | Description                                      |
|-----------------|--------|--------------------------------------------------|
| id              | i32    | Unique NPC visual type identifier                |
| sprite_filename | string | SPR filename or "null" for no sprite             |
| description     | string | NPC role/appearance description (EUC-KR encoded) |

### Special Values

- `null`: Literal string indicating no sprite filename
- `;`: Lines starting with semicolon are comments
- Empty lines are ignored

### Example Entries

```ini
; Party members
1,Party1.spr,Party Member 1
2,Party2.spr,Party Member 2

; Guards
9,guard1.spr,City Guard
10,guard2.spr,Town Guard

; Kings
16,King1.spr,Royal Guard
17,King2.spr,Royal Guard
```

### Technical Details

**Encoding**: EUC-KR (Extended Unix Code Korea)

- Supports Korean characters used in descriptions
- Requires proper encoding handling for reading/writing

**File Processing**:

- Comments (lines starting with ";") are ignored
- Empty lines are skipped
- CSV format with comma delimiter
- "null" literal used for missing sprite filenames

### Usage

1. Game loads NPC definitions from Npc.ini
2. Links NPC visual types to behavior scripts
3. Renders NPCs using specified sprite files
4. Displays descriptions in appropriate contexts
5. Manages NPC interactions based on type

### Notes

- Some entries have "null" sprite filenames when no visual is needed
- Descriptions are in Korean (EUC-KR encoding)
- File uses Windows-style line endings (`\r\n`)
- IDs may have gaps in the sequence

## Extractor

The file is parsed by the `NpcIni` struct in `src/references/npc_ini.rs`, which implements the `Extractor` trait.

### How to Run

```bash
# Extract Npc.ini to JSON
cargo run -- extract -i "Npc.ini"

# Import to SQLite database
cargo run -- database import "path/to/Dispel/" "database.sqlite"
```
