# EventItem.db Documentation

## File Format: Quest Item Database

### Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, quest data, or proprietary assets. All references to item systems are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Overview

Binary database file that defines quest and event items with names and descriptions for the game's quest progression and event triggering system.

### File Structure

**Location**: `CharacterInGame/EventItem.db`
**Encoding**: Binary (Little-Endian)
**Text Encoding**: WINDOWS-1250 (Central European)
**Header**: 4-byte record count
**Record Size**: 240 bytes (60 × 4-byte fields)
**Total Records**: Variable (determined by header)

### Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of quest item entries)

[Records: 240 bytes each]
- name: 30 bytes (WINDOWS-1250, null-padded)
- description: 202 bytes (WINDOWS-1250, null-padded)
- padding: 8 bytes (unused)
```

### Field Definitions

| Field | Size | Type | Description |
|-------|------|------|-------------|
| id | N/A | i32 | Record index (assigned during parsing) |
| name | 30 | string | Quest item name (WINDOWS-1250 encoded) |
| description | 202 | string | Item description (WINDOWS-1250 encoded) |
| padding | 8 | bytes | Unused padding bytes |

### Data Structure

The codebase defines the quest item structure as:

```rust
pub struct EventItem {
    id: i32,                    // Record index (0, 1, 2...)
    name: String,              // Item name (30 chars max)
    description: String,       // Item description (202 chars max)
}
```

### Binary Record Layout

```
Offset | Size | Field | Description
-------|------|-------|-------------
0      | 30   | name  | Null-padded WINDOWS-1250 string
30     | 202  | desc  | Null-padded WINDOWS-1250 string
232    | 8    | pad   | Unused padding bytes
```

### Special Values

- **Null-padded strings**: Fixed-size fields with null termination
- **8-byte padding**: Unused space for alignment
- **Record count**: Determines number of entries
- **Fixed record size**: 240 bytes per entry

### Example Data Structure

Based on the binary format, entries follow this pattern:

```
Record 0:
- name: "Quest Item Name" (30 bytes)
- description: "Item description text" (202 bytes)
- padding: 8 null bytes

Record 1:
- name: "Another Item" (30 bytes)
- description: "Quest-related description" (202 bytes)
- padding: 8 null bytes
```

### Technical Details

**Text Encoding**:
- WINDOWS-1250 for Central European characters
- Null-terminated strings with padding
- Fixed field sizes (30 and 202 bytes)

**Binary Processing**:
- Little-endian byte order
- Fixed record size validation
- Null-padded string handling
- 8-byte alignment padding

**Database Integration**:
- Processed by `EventItem` struct in the codebase
- Stored with id, name, and description fields
- Linked to quest and event systems
- Used for quest progression tracking

### Usage in Game

1. **Quest System**: Defines items required for quest completion
2. **Event Triggers**: Items that trigger special events
3. **Inventory Management**: Quest items in player inventory
4. **Story Progression**: Key items for narrative advancement
5. **Unique Items**: Special objects with lore significance

### File Characteristics

- **Record Size**: 240 bytes (fixed)
- **Header**: 4-byte record count
- **Encoding**: WINDOWS-1250 for text fields
- **Structure**: Simple name/description pairs
- **Alignment**: 8-byte padding for alignment

### Technical Analysis

**Efficiency:**
- Fixed record size enables random access
- Simple structure with minimal overhead
- Padding ensures proper memory alignment

**Limitations:**
- Fixed-size text fields (30 + 202 = 232 chars)
- No statistical data (pure lore items)
- Basic structure focused on quest functionality

**Performance:**
- Fast parsing due to fixed record size
- Efficient database storage
- Simple binary format for quick loading

### Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any quest data or game content
- Focuses on **technical organization and binary structure**
- Explains **database systems and quest item mechanics**
- Maintains **nominal fair use** for trademark references

### Notes

- File uses binary format with WINDOWS-1250 text encoding
- Simple name/description structure for quest items
- Fixed record size enables efficient parsing
- Integrated with quest progression systems
- **No copyrighted game content** is reproduced or distributed

## Extractor

An extractor is available in `src/references/event_item_db.rs` to parse this file format.

### How to Run

```bash
# Extract EventItem.db to JSON
cargo run -- ref event-items "fixtures/Dispel/CharacterInGame/EventItem.db"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
