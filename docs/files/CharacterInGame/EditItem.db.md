# EditItem.db Documentation

## File Format: Modifiable Item Database

### Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, item data, or proprietary assets. All references to item systems are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Overview

Binary database file that defines modifiable items with upgradeable statistics and special effects for the game's item enhancement and crafting system.

### File Structure

**Location**: `CharacterInGame/EditItem.db`
**Encoding**: Binary (Little-Endian)
**Text Fields**: Mixed encodings (WINDOWS-1250)
**Header**: 4-byte record count
**Record Size**: 268 bytes (67 × 4-byte fields)
**Total Records**: Variable (determined by header)

### Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of item entries)

[Records: 268 bytes each]
- name: 30 bytes (WINDOWS-1250, null-padded)
- description: 202 bytes (WINDOWS-1250, null-padded)
- base_price: i16 (economic value)
- padding: 6 bytes
- health_points: i16 (Health Points)
- mana_points: i16 (Mana Points)
- strength: i16 (Strength)
- agility: i16 (Agility)
- wisdom: i16 (Wisdom)
- constitution: i16 (Constitution)
- to_dodge: i16 (Dodge modifier)
- to_hit: i16 (Hit Chance)
- offense: i16 (Attack power)
- defense: i16 (Defense power)
- magical_power: i16 (magical power bonus)
- item_destroying_power: i16 (durability impact)
- padding: u8
- modifies_item: u8 (modification flag)
- additional_effect: i16 (special effect type)
```

### Field Definitions

| Field | Size | Type | Description |
|-------|------|------|-------------|
| name | 30 | string | Item name (WINDOWS-1250 encoded) |
| description | 202 | string | Item description (WINDOWS-1250 encoded) |
| base_price | 2 | i16 | Economic value (0 = non-tradable) |
| padding | 6 | bytes | Unused padding |
| health_points | 2 | i16 | Health bonus |
| mana_points | 2 | i16 | Mana bonus |
| strength | 2 | i16 | Strength bonus |
| agility | 2 | i16 | Agility bonus |
| wisdom | 2 | i16 | Wisdom bonus |
| constitution | 2 | i16 | Constitution bonus |
| to_dodge | 2 | i16 | Dodge modifier |
| to_hit | 2 | i16 | Hit chance |
| offense | 2 | i16 | Attack power |
| defense | 2 | i16 | Defense power |
| magical_power | 2 | i16 | Magical power bonus |
| item_destroying_power | 2 | i16 | Durability/destruction impact |
| padding | 1 | u8 | Unused padding |
| modifies_item | 1 | u8 | Modification capability flag |
| additional_effect | 2 | i16 | Special effect type |

### Statistic Fields

**Field Descriptions:**
- **Health Points**: Character vitality/HP bonus
- **Mana Points**: Magic energy/MP bonus
- **Strength**: Physical power and melee damage
- **Agility**: Speed and evasion capabilities
- **Wisdom**: Magic effectiveness and resistance
- **Constitution**: Physical endurance and defense
- **Dodge**: Chance to avoid attacks
- **Hit Chance**: Accuracy and attack success rate
- **Attack**: Offensive power and damage output
- **Defense**: Protective capabilities and damage reduction

### Modification System

**Modification Types (modifies_item field):**
- **0**: DoesNotModify - Item cannot modify other items
- **1**: CanModify - Item can modify/enhance other items

**Additional Effects (additional_effect field):**
- **0**: None - No special effect
- **1**: Fire - Fire damage/elemental effect
- **2**: ManaDrain - Mana draining effect

### Data Structure

The codebase defines the item structure as:

```rust
pub struct EditItem {
    index: i32,                    // Record index
    name: String,                 // Item name (30 chars max)
    description: String,          // Item description (202 chars max)
    base_price: i16,              // Economic value
    health_points: i16,           // Health bonus
    mana_points: i16,            // Mana bonus
    strength: i16,               // Strength bonus
    agility: i16,                // Agility bonus
    wisdom: i16,                 // Wisdom bonus
    constitution: i16,           // Constitution bonus
    to_dodge: i16,               // Dodge modifier
    to_hit: i16,                 // Hit chance bonus
    offense: i16,                // Attack power
    defense: i16,                // Defense power
    magical_power: i16,          // Magical power
    item_destroying_power: i16,   // Durability impact
    padding3: u8,                // Unused padding
    modifies_item: EditItemModification, // Modification capability
    additional_effect: EditItemEffect,   // Special effect type
}
```

### Special Values

- **base_price = 0**: Non-tradable items
- **item_destroying_power**: Higher values = more durability impact
- **modifies_item = 0**: Cannot modify other items
- **modifies_item = 1**: Can enhance/modify other items
- **additional_effect = 0**: No special effect
- **additional_effect = 1**: Fire effect
- **additional_effect = 2**: Mana drain effect

### Binary Record Layout

```
Offset | Size | Field | Description
-------|------|-------|-------------
0      | 30   | name | Null-padded WINDOWS-1250 string
30     | 202  | desc | Null-padded WINDOWS-1250 string
232    | 2    | price| Economic value (i16)
234    | 6    | pad1 | Padding bytes
240    | 2    | HP   | Health points (i16)
242    | 2    | MP   | Mana points (i16)
244    | 2    | STR  | Strength (i16)
246    | 2    | AGI  | Agility (i16)
248    | 2    | WIS  | Wisdom (i16)
250    | 2    | CON  | Constitution (i16)
252    | 2    | DOD  | Dodge modifier (i16)
254    | 2    | HIT  | Hit chance (i16)
256    | 2    | ATK  | Attack power (i16)
258    | 2    | DEF  | Defense power (i16)
260    | 2    | MAG  | Magical power bonus (i16)
262    | 2    | DUR  | Item destroying power (i16)
264    | 1    | pad2 | Padding byte
265    | 1    | mod  | Modification flag (u8)
266    | 2    | eff  | Additional effect (i16)
```

### Usage in Game

1. **Item Enhancement**: Used for upgrading equipment
2. **Crafting System**: Provides modification capabilities
3. **Economic System**: Base price determines trade value
4. **Combat System**: Stats affect character performance
5. **Special Effects**: Additional effects provide unique abilities

### File Characteristics

- **Record Size**: 268 bytes (fixed)
- **Header**: 4-byte record count
- **Encoding**: Mixed (WINDOWS-1250 for text)
- **Structure**: Strict binary format
- **Alignment**: 4-byte aligned fields

### Technical Details

**Text Encoding**:
- WINDOWS-1250 for Polish characters
- Null-terminated strings with padding
- Fixed field sizes (30 and 202 bytes)

**Binary Processing**:
- Little-endian byte order
- Fixed record size validation
- Null-padded string handling
- Enum conversion for flags

**Database Integration**:
- Processed by `EditItem` struct
- Uses type-safe enums for flags
- Stored with all statistical fields
- Linked to inventory and crafting systems

### Notes

- File uses binary format with mixed text encodings
- Fixed record size enables efficient random access
- Integrated with crafting and enhancement systems
- **No copyrighted game content** is reproduced or distributed

## Extractor

An extractor is available in `src/references/edit_item_db.rs` to parse this file format.

### How to Run

```bash
# Extract EditItem.db to JSON
cargo run -- ref edit-items "fixtures/Dispel/CharacterInGame/EditItem.db"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
