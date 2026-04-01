# Extra.ini Documentation

## File Format: Interactive Object Definitions

### Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game assets, sprite files, or proprietary artwork. All references to interactive objects are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Overview

Text file that defines interactive objects with visual assets, flags, and descriptions for the game's environmental interaction system.

### File Structure

**Location**: `Extra.ini`
**Encoding**: EUC-KR (Korean character encoding)
**Format**: CSV (Comma-Separated Values) with comments
**Total Entries**: 182 interactive object definitions

### Format Specification

```ini
; Comment line explaining field structure
id,sprite_filename,flag,description
0,null,0,null
1,object1.spr,0,Object description
2,object2.spr,1,Special object description
...
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| id | i32 | Unique interactive object identifier (0-181) |
| sprite_filename | string | SPR/SPX filename or "null" |
| flag | i32 | Object flag (0 = standard, 1 = special) |
| description | string | Object description or "null" |

### Flag System

**Standard Objects (flag = 0):**
- Regular interactive objects
- Basic containers and doors
- Standard environmental features

**Special Objects (flag = 1):**
- Quest-related objects
- Unique interaction points
- Special mechanics or behaviors

### Special Values

- **"null"**: Literal string indicating no sprite or description
- **flag = 0**: Standard interactive object
- **flag = 1**: Special/quest-related object
- **;**: Lines starting with semicolon are comments
- **Empty lines**: Ignored during processing

### Example Format

```ini
; Default entry
0,null,0,null

; Container template
1,container.spr,0,Storage object
2,container.spr,0,Loot container

; Special object template
10,special.spr,1,Quest-related object
11,unique.spr,1,Story-critical item
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
    unknown: i32,               // Object flag (0 or 1)
    description: Option<String>, // Object description
}
```

### Usage in Game

1. **Environment Interaction**: Defines objects players can interact with
2. **Visual Mapping**: Links object IDs to sprite files
3. **Quest Integration**: Special objects trigger quest progression
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

- **Entry Count**: 182 object definitions
- **ID Range**: 0-181 (ID 0 = default/null entry)
- **Flag Distribution**: Mix of standard (0) and special (1) objects
- **Comment Organization**: Logical grouping by object type
- **Encoding**: EUC-KR with Korean descriptions

### Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any SPR/SPX files or game artwork
- Focuses on **technical organization and interaction systems**
- Uses **generic examples** of object structures
- Maintains **nominal fair use** for trademark references

### Notes

- File uses Windows-style line endings (\r\n)
- Comments are in Polish and Korean (mixed encoding)
- Descriptions use EUC-KR encoding for international characters
- Integrated with map placement and interaction systems
- **No copyrighted game content** is reproduced or distributed