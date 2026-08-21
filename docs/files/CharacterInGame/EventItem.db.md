# EventItem.db

> DISPEL® is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
> owner.

## File Information

### Overview

Binary database file that defines quest and event items with names and descriptions for the game's quest progression and
event triggering system.

### File Structure

**Location**: `CharacterInGame/EventItem.db`
**Encoding**: Binary (Little-Endian)
**Text Encoding**: WINDOWS-1250 (Central European)
**Header**: 4-byte record count **Record Size**: 240 bytes **Total Records**: Variable (determined by header)

### Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of quest item entries)

[Records: 240 bytes each]
- name: 30 bytes (WINDOWS-1250, null-padded)
- description: 202 bytes (WINDOWS-1250, null-padded)
- base_price: i32
- padding: 4 bytes (unused)
```

### Field Definitions

| Field       | Size | Type   | Description                             |
|-------------|------|--------|-----------------------------------------|
| id          | N/A  | i32    | Record index (assigned during parsing)  |
| name        | 30   | string | Quest item name (WINDOWS-1250 encoded)  |
| description | 202  | string | Item description (WINDOWS-1250 encoded) |
| base_price  | 4    | i32    | Item price                              |
| padding     | 4    | bytes  | Unused padding bytes                    |

### Data Structure

The codebase defines the quest item structure as:

```rust
pub struct EventItem {
    id: i32,                    // Record index (0, 1, 2...)
    name: String,              // Item name (30 chars max)
    description: String,       // Item description (202 chars max)
    base_price: i32,           // Item price
    padding: i32,              // Unused padding
}
```

### Binary Record Layout

```
Offset | Size | Field | Description
-------|------|-------|-------------
0      | 30   | name  | Null-padded WINDOWS-1250 string
30     | 202  | desc  | Null-padded WINDOWS-1250 string
232    | 4    | price | Item price (i32)
236    | 4    | pad   | Unused padding bytes
```

### Special Values

- **Null-padded strings**: Fixed-size fields with null termination
- **4-byte padding**: Unused space for alignment
- **Record count**: Determines number of entries
- **Fixed record size**: 240 bytes per entry

### Technical Details

**Text Encoding**:

- WINDOWS-1250 for Central European characters
- Null-terminated strings with padding
- Fixed field sizes (30 and 202 bytes)

**Binary Processing**:

- Little-endian byte order
- Fixed record size validation
- Null-padded string handling

### Notes

- File uses binary format with WINDOWS-1250 text encoding
- Simple name/description/price structure for quest items
- Fixed record size enables efficient parsing
- Integrated with quest progression systems

## Extractor

An extractor is available in `src/references/event_item_db.rs` to parse this file format.

### How to Run

```bash
# Extract EventItem.db to JSON
cargo run -- extract -i "fixtures/Dispel/CharacterInGame/EventItem.db"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
