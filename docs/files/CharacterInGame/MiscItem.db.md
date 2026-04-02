# MiscItem.db Documentation

## File Format: Generic Items Database

### Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, item data, or proprietary assets. All references to item systems are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Overview

Binary database file that defines generic miscellaneous items with names, descriptions, and economic values for the game's crafting, inventory, and utility systems.

### File Structure

**Location**: `CharacterInGame/MiscItem.db`
**Encoding**: Binary (Little-Endian)
**Text Encodings**: Mixed (WINDOWS-1250 and EUC-KR)
**Header**: 4-byte record count
**Record Size**: 256 bytes (64 × 4-byte fields)
**Total Records**: Variable (determined by header)

### Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of miscellaneous item entries)

[Records: 256 bytes each]
- name: 30 bytes (WINDOWS-1250, null-padded)
- description: 202 bytes (EUC-KR, null-padded)
- base_price: i32 (economic value)
- padding: 20 bytes (unused)
```

### Field Definitions

| Field | Size | Type | Description |
|-------|------|------|-------------|
| id | N/A | i32 | Record index (assigned during parsing) |
| name | 30 | string | Item name (WINDOWS-1250 encoded) |
| description | 202 | string | Item description (EUC-KR encoded) |
| base_price | 4 | i32 | Economic value (0 = non-tradable, -1 = quest item) |
| padding | 20 | bytes | Unused padding field |

### Data Structure

The codebase defines the generic item structure as:

```rust
pub struct MiscItem {
    id: i32,                    // Record index
    name: String,              // Item name (30 chars max)
    description: String,       // Item description (202 chars max)
    base_price: i32,           // Economic value
}
```

### Binary Record Layout

```
Offset | Size | Field | Description
-------|------|-------|-------------
0      | 30   | name  | Null-padded WINDOWS-1250 string
30     | 202  | desc  | Null-padded EUC-KR string
232    | 4    | price | Economic value (i32)
236    | 20   | pad   | Unused padding bytes
```

### Special Values

- **base_price = 0**: Non-tradable items
- **base_price = -1**: Quest-related items
- **Positive base_price**: Tradable items with economic value
- **Null-padded strings**: Fixed-size fields with null termination
- **20-byte padding**: Unused space for alignment



### Usage in Game

1. **Inventory System**: Generic items in player inventory
2. **Crafting System**: Materials for item creation
3. **Economic System**: Tradable items with value
4. **Quest System**: Special quest-related objects
5. **Utility Functions**: Tools and functional items

### File Characteristics

- **Record Size**: 256 bytes (fixed)
- **Header**: 4-byte record count
- **Encoding**: Mixed text encodings
- **Structure**: Simple name/description/price system
- **Padding**: 20-byte alignment padding

### Technical Details

**Text Encoding:**
- WINDOWS-1250 for item names
- EUC-KR for item descriptions
- Null-terminated strings with padding
- Fixed field sizes (30 and 202 bytes)

**Binary Processing:**
- Little-endian byte order
- Fixed record size validation
- Mixed text encoding handling
- 32-bit economic values

**Database Integration:**
- Processed by `MiscItem` struct
- Stored with name, description, and price
- Linked to inventory and crafting systems
- Simple binary format for quick loading

### Economic System

**Price Categories:**
- **base_price = 0**: Non-tradable items
- **base_price = -1**: Quest items (special)
- **base_price > 0**: Tradable items with value
- **Higher values**: More valuable items

**Item Value Analysis:**
- Currency items typically have standard values
- Crafting materials vary by rarity
- Quest items are non-tradable (base_price = -1)
- Utility items have functional value

### Technical Analysis

**Efficiency:**
- Fixed record size enables random access
- Simple structure with minimal overhead
- Large padding for future expansion
- Fast parsing and loading

**Limitations:**
- Fixed-size text fields (30 + 202 = 232 chars)
- No statistical data (pure utility items)
- Basic structure focused on generic items
- Large padding reduces space efficiency

**Performance:**
- Fast parsing due to fixed record size
- Efficient database storage
- Simple binary format for quick loading
- Minimal processing requirements

### Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any item data or game content
- Focuses on **technical organization and item systems**
- Explains **generic item mechanics and structure**
- Maintains **nominal fair use** for trademark references

### Notes

- File uses binary format with mixed text encodings
- Simple structure for generic utility items
- Fixed record size enables efficient parsing
- Integrated with inventory and crafting systems
- **No copyrighted game content** is reproduced or distributed

### Comparison with Other Item Databases

**MiscItem.db vs HealItem.db:**
- **MiscItem.db**: Generic utility items
- **HealItem.db**: Specialized healing consumables
- **MiscItem.db**: No healing effects
- **HealItem.db**: Complex healing mechanics

**MiscItem.db vs EditItem.db:**
- **MiscItem.db**: Basic utility items
- **EditItem.db**: Modifiable equipment
- **MiscItem.db**: No statistical modifiers
- **EditItem.db**: Complex stat systems

**MiscItem.db vs EventItem.db:**
- **MiscItem.db**: Functional utility items
- **EventItem.db**: Quest/lore items only
- **MiscItem.db**: Economic value system
- **EventItem.db**: Story progression focus

### File Purpose Summary

MiscItem.db serves as a comprehensive database for:
- Currency and economic items
- Crafting materials and components
- Utility tools and equipment
- Quest-related special items
- Miscellaneous game objects

The file provides a simple but flexible system for managing all types of generic items in the game, supporting both functional utility items and narrative quest objects.

## Extractor

An extractor is available in `src/references/misc_item_db.rs` to parse this file format.

### How to Run

```bash
# Extract MiscItem.db to JSON
cargo run -- ref misc-item "fixtures/Dispel/CharacterInGame/MiscItem.db"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```