# HealItem.db Documentation

## File Information

### Overview

Binary database file that defines consumable healing items with restoration effects, status cures, and economic values for the game's inventory and combat systems.

### File Structure

**Location**: `CharacterInGame/HealItem.db`
**Encoding**: Binary (Little-Endian)
**Text Encodings**: Mixed (WINDOWS-1250 and EUC-KR)
**Header**: 4-byte record count
**Record Size**: 252 bytes
**Total Records**: Variable (determined by header)

### Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of healing item entries)

[Records: 252 bytes each]
- name: 30 bytes (WINDOWS-1250, null-padded)
- description: 202 bytes (EUC-KR, null-padded)
- base_price: i32 (economic value)
- runtime_item_index_slot: i32 (overwritten with the record index at load time)
- health_points: i16 (HP restore amount)
- mana_points: i16 (MP restore amount)
- restores_full_health: u8 (full HP restoration flag)
- restores_full_mana: u8 (full MP restoration flag)
- cures_poison: u8 (poison cure flag)
- cures_petrification: u8 (petrification cure flag)
- cures_polymorph: u8 (polymorph cure flag)
- reserved_trailer: 3 bytes (preserved, no use found)
```

### Field Definitions

| Field | Size | Type | Description |
|-------|------|------|-------------|
| id | N/A | i32 | Record index (assigned during parsing) |
| name | 30 | string | Item name (WINDOWS-1250 encoded) |
| description | 202 | string | Item description (EUC-KR encoded) |
| base_price | 4 | i32 | Economic value (0 = non-tradable) |
| runtime_item_index_slot | 4 | i32 | Replaced by the sequential record index while loading |
| health_points | 2 | i16 | Health points restored (PZ) |
| mana_points | 2 | i16 | Mana points restored (PM) |
| restores_full_health | 1 | u8 | Full health restoration flag |
| restores_full_mana | 1 | u8 | Full mana restoration flag |
| cures_poison | 1 | u8 | Poison status cure flag |
| cures_petrification | 1 | u8 | Petrification status cure flag |
| cures_polymorph | 1 | u8 | Polymorph status cure flag |
| reserved_trailer | 3 | bytes | Reserved data, zero in the bundled fixture |

### Healing Flag System

**Flag Values (HealItemFlag enum):**
- **0**: None - No effect
- **1**: Active - The associated restoration or cure is active

**Flag Fields:**
- `restores_full_health`: Restores health to maximum
- `restores_full_mana`: Restores mana to maximum
- `cures_poison`: Cures poison status effect
- `cures_petrification`: Cures petrification status effect
- `cures_polymorph`: Cures polymorph status effect

### Data Structure

The codebase defines the healing item structure as:

```rust
pub struct HealItem {
    id: i32,                    // Record index
    name: String,              // Item name (30 chars max)
    description: String,       // Item description (202 chars max)
    base_price: i32,           // Economic value
    runtime_item_index_slot: i32, // Overwritten with record index at load time
    health_points: i16,        // HP restore amount
    mana_points: i16,         // MP restore amount
    restores_full_health: HealItemFlag, // Full HP restoration
    restores_full_mana: HealItemFlag,   // Full MP restoration
    cures_poison: HealItemFlag,          // Poison cure
    cures_petrification: HealItemFlag,   // Petrification cure
    cures_polymorph: HealItemFlag,       // Polymorph cure
    reserved_trailer: Vec<u8>,            // Three preserved bytes
}
```

### Binary Record Layout

```
Offset | Size | Field | Description
-------|------|-------|-------------
0      | 30   | name  | Null-padded WINDOWS-1250 string
30     | 202  | desc  | Null-padded EUC-KR string
232    | 4    | price | Economic value (i32)
236    | 4    | runtime_item_index_slot | Replaced with record index at load time
240    | 2    | PZ    | Health restore amount (i16)
242    | 2    | PM    | Mana restore amount (i16)
244    | 1    | full_hp | Full HP restoration flag (u8)
245    | 1    | full_mp | Full MP restoration flag (u8)
246    | 1    | poison | Poison cure flag (u8)
247    | 1    | petrification | Petrification cure flag (u8)
248    | 1    | polymorph | Polymorph cure flag (u8)
249    | 3    | reserved_trailer | Reserved bytes; no direct use found
```

### Special Values

- **base_price = 0**: Non-tradable items
- **health_points/mana_points**: Positive = restore, Negative = damage
- **Flags = 0**: No effect
- **Flags = 1**: Full restoration/cure active
- **Null-padded strings**: Fixed-size fields with null termination

### Item Types

Based on the healing effects, items can be categorized:

**Basic Healing Items:**
- Restore health points (positive health_points)
- Restore mana points (positive mana_points)
- No special status cures

**Full Restoration Items:**
- `restores_full_health = 1`: Complete HP recovery
- `restores_full_mana = 1`: Complete MP recovery
- Often high-value quest items

**Status Cure Items:**
- `cures_poison = 1`: Cures poison effects
- `cures_petrification = 1`: Cures petrification
- `cures_polymorph = 1`: Cures polymorph
- Specialized healing items


### Usage in Game

1. **Inventory System**: Healing items in player inventory
2. **Combat Healing**: Restore health/mana during battles
3. **Status Recovery**: Cure negative status effects
4. **Economic System**: Items with trade value
5. **Quest Rewards**: Special healing items as rewards

### File Characteristics

- **Record Size**: 252 bytes (fixed)
- **Header**: 4-byte record count
- **Encoding**: Mixed text encodings
- **Structure**: Complex healing effect system
- **Runtime data**: The loader replaces `runtime_item_index_slot` with the record index.
- **Reserved data**: The final three bytes are preserved verbatim.

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
- Flag enum conversion

**Database Integration:**
- Processed by `HealItem` struct
- Uses type-safe enum for flags
- Stored with all healing properties
- Linked to inventory and combat systems

### Healing Effect System

The game supports sophisticated healing mechanics:

**Partial Restoration:**
- Fixed HP/MP amounts (health_points/mana_points)
- Stackable healing effects
- Consumable item usage

**Full Restoration:**
- Complete HP/MP recovery
- Quest-critical items
- Rare high-value consumables

**Status Cures:**
- Poison antidotes
- Petrification remedies
- Polymorph reversals
- Specialized healing

### Notes

- File uses binary format with mixed text encodings
- Complex healing system with multiple effect types
- Fixed record size enables efficient parsing
- Integrated with inventory and combat systems
- **No copyrighted game content** is reproduced or distributed

### Comparison with Other Item Databases

**HealItem.db vs EditItem.db:**
- **HealItem.db**: Consumable healing items with effects
- **EditItem.db**: Modifiable equipment with statistics
- **HealItem.db**: Single-use consumables
- **EditItem.db**: Permanent equipment upgrades

**HealItem.db vs EventItem.db:**
- **HealItem.db**: Functional healing items
- **EventItem.db**: Quest/lore items only
- **HealItem.db**: Gameplay mechanics
- **EventItem.db**: Story progression

## Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, item data, or proprietary assets. All references to healing systems are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

## Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any item data or game content
- Focuses on **technical organization and healing systems**
- Explains **consumable item mechanics and effects**
- Maintains **nominal fair use** for trademark references

## Extractor

An extractor is available in `src/references/heal_item_db.rs` to parse this file format.

### How to Run

```bash
# Extract HealItem.db to JSON
cargo run -- extract -i "fixtures/Dispel/CharacterInGame/HealItem.db"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
