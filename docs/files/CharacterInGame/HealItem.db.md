# HealItem.db Documentation

## File Format: Consumable Healing Items Database

### Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, item data, or proprietary assets. All references to healing systems are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Overview

Binary database file that defines consumable healing items with restoration effects, status cures, and economic values for the game's inventory and combat systems.

### File Structure

**Location**: `CharacterInGame/HealItem.db`
**Encoding**: Binary (Little-Endian)
**Text Encodings**: Mixed (WINDOWS-1250 and EUC-KR)
**Header**: 4-byte record count
**Record Size**: 252 bytes (63 × 4-byte fields)
**Total Records**: Variable (determined by header)

### Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of healing item entries)

[Records: 252 bytes each]
- name: 30 bytes (WINDOWS-1250, null-padded)
- description: 202 bytes (EUC-KR, null-padded)
- base_price: i16 (economic value)
- padding: i16 × 3 (12 bytes unused)
- health_points: i16 (HP restore amount)
- mana_points: i16 (MP restore amount)
- restore_full_health: u8 (full HP restoration flag)
- restore_full_mana: u8 (full MP restoration flag)
- poison_heal: u8 (poison cure flag)
- petrif_heal: u8 (petrification cure flag)
- polimorph_heal: u8 (polymorph cure flag)
- padding: u8 + i16 (3 bytes unused)
```

### Field Definitions

| Field | Size | Type | Description |
|-------|------|------|-------------|
| id | N/A | i32 | Record index (assigned during parsing) |
| name | 30 | string | Item name (WINDOWS-1250 encoded) |
| description | 202 | string | Item description (EUC-KR encoded) |
| base_price | 2 | i16 | Economic value (0 = non-tradable) |
| padding1-3 | 2×3 | i16 | Unused padding fields |
| health_points | 2 | i16 | Health points restored (PZ) |
| mana_points | 2 | i16 | Mana points restored (PM) |
| restore_full_health | 1 | u8 | Full health restoration flag |
| restore_full_mana | 1 | u8 | Full mana restoration flag |
| poison_heal | 1 | u8 | Poison status cure flag |
| petrif_heal | 1 | u8 | Petrification status cure flag |
| polimorph_heal | 1 | u8 | Polymorph status cure flag |
| padding4 | 1 | u8 | Unused padding byte |
| padding5 | 2 | i16 | Unused padding field |

### Healing Flag System

**Flag Values (HealItemFlag enum):**
- **0**: None - No effect
- **1**: FullRestoration - Complete restoration/cure

**Flag Fields:**
- `restore_full_health`: Restores health to maximum
- `restore_full_mana`: Restores mana to maximum
- `poison_heal`: Cures poison status effect
- `petrif_heal`: Cures petrification status effect
- `polimorph_heal`: Cures polymorph status effect

### Data Structure

The codebase defines the healing item structure as:

```rust
pub struct HealItem {
    id: i32,                    // Record index
    name: String,              // Item name (30 chars max)
    description: String,       // Item description (202 chars max)
    base_price: i16,           // Economic value
    padding1: i16,            // Unused padding
    padding2: i16,            // Unused padding
    padding3: i16,            // Unused padding
    health_points: i16,        // HP restore amount
    mana_points: i16,         // MP restore amount
    restore_full_health: HealItemFlag, // Full HP restoration
    restore_full_mana: HealItemFlag,   // Full MP restoration
    poison_heal: HealItemFlag,        // Poison cure
    petrif_heal: HealItemFlag,        // Petrification cure
    polimorph_heal: HealItemFlag,     // Polymorph cure
    padding4: u8,             // Unused padding
    padding5: i16,            // Unused padding
}
```

### Binary Record Layout

```
Offset | Size | Field | Description
-------|------|-------|-------------
0      | 30   | name  | Null-padded WINDOWS-1250 string
30     | 202  | desc  | Null-padded EUC-KR string
232    | 2    | price | Economic value (i16)
234    | 2    | pad1  | Unused padding (i16)
236    | 2    | pad2  | Unused padding (i16)
238    | 2    | pad3  | Unused padding (i16)
240    | 2    | PZ    | Health restore amount (i16)
242    | 2    | PM    | Mana restore amount (i16)
244    | 1    | full_hp | Full HP restoration flag (u8)
245    | 1    | full_mp | Full MP restoration flag (u8)
246    | 1    | poison | Poison cure flag (u8)
247    | 1    | petrif | Petrification cure flag (u8)
248    | 1    | poly   | Polymorph cure flag (u8)
249    | 1    | pad4  | Unused padding (u8)
250    | 2    | pad5  | Unused padding (i16)
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
- `restore_full_health = 1`: Complete HP recovery
- `restore_full_mana = 1`: Complete MP recovery
- Often high-value quest items

**Status Cure Items:**
- `poison_heal = 1`: Cures poison effects
- `petrif_heal = 1`: Cures petrification
- `polimorph_heal = 1`: Cures polymorph
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
- **Padding**: Multiple unused fields for alignment

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

### Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any item data or game content
- Focuses on **technical organization and healing systems**
- Explains **consumable item mechanics and effects**
- Maintains **nominal fair use** for trademark references

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
