# Npc.ini Documentation

## File Format: NPC Visual References

### Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game assets, sprite files, or proprietary content. All references to NPC types are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Overview

Text file that defines visual appearances and descriptions for NPC (Non-Player Character) types in the game.

### File Structure

**Location**: `Npc.ini`
**Encoding**: EUC-KR (Korean character encoding)
**Format**: CSV (Comma-Separated Values) with comments

### Format Specification

```
; Comment line
id,sprite_filename,description
1,guard1.spr,City Guard
2,merchant.spr,Shopkeeper
...
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| id | i32 | Unique NPC visual type identifier (0-99+) |
| sprite_filename | string | SPR filename or "null" for no sprite |
| description | string | NPC role/appearance description (EUC-KR encoded) |

### Special Values

- `null`: Literal string indicating no sprite filename
- `;`: Lines starting with semicolon are comments
- Empty lines are ignored

### Example Entries

```ini
; Party members
1,Party1.spr,NPC description
2,Party2.spr,NPC description
3,Party3.spr,NPC description

; Guards
9,guard1.spr,NPC description
10,guard2.spr,NPC description
11,guard3.spr,NPC description

; Kings
16,King1.spr,NPC description
17,King2.spr,NPC description
18,King3.spr,NPC description
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

### Usage in Game

1. Game loads NPC definitions from Npc.ini
2. Links NPC visual types to behavior scripts
3. Renders NPCs using specified sprite files
4. Displays descriptions in appropriate contexts
5. Manages NPC interactions based on type

### Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any SPR sprite files or game assets
- Uses **generic placeholders** for descriptions
- Focuses on **technical organization**, not creative content
- Maintains **nominal fair use** for trademark references

### Notes

- Some entries have "null" sprite filenames (IDs 0, 12, 29, etc.)
- Descriptions are in Korean (EUC-KR encoding)
- File uses Windows-style line endings (\r\n)
- IDs appear to be sequential but with some gaps (45-50, 64-66, etc.)
- **No copyrighted game content** is reproduced or distributed
