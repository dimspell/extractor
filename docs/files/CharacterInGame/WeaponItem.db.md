# WeaponItem.db Documentation

## File Format: Weapons & Armor Database

### Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, weapon data, or proprietary assets. All references to equipment systems are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Overview

Binary database file that defines all weapons, armor, and equipment with comprehensive statistics, requirements, and game properties for the game's character equipment and inventory systems.

### File Structure

**Location**: `CharacterInGame/weaponItem.db`
**Encoding**: Binary (Little-Endian)
**Text Encoding**: WINDOWS-1250
**Header**: 4-byte record count
**Record Size**: 284 bytes (71 × 4-byte fields)
**Total Records**: Variable (determined by header)

### Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of weapon/armor entries)

[Records: 284 bytes each]
- name: 30 bytes (WINDOWS-1250, null-padded)
- description: 202 bytes (WINDOWS-1250, null-padded)
- base_price: i16 (economic value)
- padding: i16 × 3
- health_points: i16 (HP modifier)
- mana_points: i16 (MP modifier)
- strength: i16 (STR bonus)
- agility: i16 (AGI bonus)
- wisdom: i16 (WIS/MAG bonus)
- constitution: i16 (CON bonus)
- to_dodge: i16 (dodge modifier)
- to_hit: i16 (hit chance bonus)
- attack: i16 (offensive power)
- defense: i16 (defensive power)
- magical_strength: i16 (magic power)
- durability: i16 (item durability)
- padding: i16 × 2
- req_strength: i16 (required STR)
- padding: i16
- req_agility: i16 (required AGI)
- padding: i16
- req_wisdom: i16 (required WIS)
- padding: i16 × 3
```

### Field Definitions

| Field            | Size | Type   | Description                             |
| ---------------- | ---- | ------ | --------------------------------------- |
| id               | N/A  | i32    | Record index (assigned during parsing)  |
| name             | 30   | string | Item name (WINDOWS-1250 encoded)        |
| description      | 202  | string | Item description (WINDOWS-1250 encoded) |
| base_price       | 2    | i16    | Economic value (shop price)             |
| health_points    | 2    | i16    | Health points bonus                     |
| mana_points      | 2    | i16    | Mana points bonus                       |
| strength         | 2    | i16    | Strength bonus                          |
| agility          | 2    | i16    | Agility bonus                           |
| wisdom           | 2    | i16    | Wisdom/Magic bonus                      |
| constitution     | 2    | i16    | Constitution bonus                      |
| to_dodge         | 2    | i16    | Dodge chance modifier                   |
| to_hit           | 2    | i16    | Hit chance bonus                        |
| attack           | 2    | i16    | Offensive power                         |
| defense          | 2    | i16    | Defensive power                         |
| magical_strength | 2    | i16    | Magical power bonus                     |
| durability       | 2    | i16    | Item durability/health                  |
| req_strength     | 2    | i16    | Required strength to equip              |
| req_agility      | 2    | i16    | Required agility to equip               |
| req_wisdom       | 2    | i16    | Required wisdom to equip                |

### Statistic System

**Character Bonuses:**

- **Health/Mana**: Direct stat improvements
- **Strength/Agility/Wisdom**: Attribute bonuses
- **Constitution**: Physical endurance
- **Dodge/Hit**: Combat effectiveness
- **Attack/Defense**: Combat power
- **Magical Strength**: Spell effectiveness

**Requirement System:**

- **req_strength**: Minimum strength to equip
- **req_agility**: Minimum agility to equip
- **req_wisdom**: Minimum wisdom to equip
- Prevents use of advanced equipment

### Data Structure

The codebase defines the weapon/armor structure as:

```rust
pub struct WeaponItem {
    id: i32,                    // Record index
    name: String,              // Item name (30 chars max)
    description: String,       // Item description (202 chars max)
    base_price: i16,           // Shop value in gold
    health_points: i16,       // HP modifier
    mana_points: i16,         // MP modifier
    strength: i16,            // Strength bonus
    agility: i16,             // Agility bonus
    wisdom: i16,              // Wisdom/Magic bonus
    constitution: i16,        // Constitution bonus
    to_dodge: i16,            // Dodge modifier
    to_hit: i16,              // Hit chance bonus
    attack: i16,             // Offensive power
    defense: i16,            // Defensive power
    magical_strength: i16,    // Magical power
    durability: i16,         // Item durability
    req_strength: i16,        // Required strength
    req_agility: i16,         // Required agility
    req_wisdom: i16,          // Required wisdom
}
```

### Binary Record Layout

```
Offset | Size | Field | Description
-------|------|-------|-------------
0      | 30   | name  | Null-padded WINDOWS-1250 string
30     | 202  | desc  | Null-padded WINDOWS-1250 string
232    | 2    | price | Economic value (i16)
234    | 6    | pad1  | Unused padding
240    | 2    | HP    | Health bonus (i16)
242    | 2    | MP    | Mana bonus (i16)
244    | 2    | STR   | Strength bonus (i16)
246    | 2    | AGI   | Agility bonus (i16)
248    | 2    | WIS   | Wisdom bonus (i16)
250    | 2    | CON   | Constitution bonus (i16)
252    | 2    | DOD   | Dodge modifier (i16)
254    | 2    | HIT   | Hit chance bonus (i16)
256    | 2    | ATK   | Attack power (i16)
258    | 2    | DEF   | Defense power (i16)
260    | 2    | MAG   | Magical power (i16)
262    | 2    | DUR   | Durability (i16)
264    | 4    | pad2  | Unused padding
268    | 2    | REQ_STR | Required strength (i16)
270    | 2    | pad3  | Unused padding
272    | 2    | REQ_AGI | Required agility (i16)
274    | 2    | pad4  | Unused padding
276    | 2    | REQ_WIS | Required wisdom (i16)
278    | 6    | pad5  | Unused padding
```

### Field Abbreviations

| Explanation                 | Polish  | English |
| --------------------------- | ------- | ------- |
| Health Points               | PZ      | HP      |
| Mana Points                 | PM      | MP      |
| Strength                    | SIŁ     | STR     |
| Agility                     | ZW      | AGI     |
| Constitution                | TF      | CON     |
| Wisdom/Magic                | MM      | WIS     |
| Dodge                       | UNK     | DOD     |
| Hit Rate                    | TRF     | HIT     |
| Attack power                | ATK     | ATK     |
| Defense                     | OBR     | DEF     |
| Magical power               | MAG/PGM | MAG     |
| Durability                  | WYT     | DUR     |
| Required stat for equipment | REQ     | REQ     |

### Usage in Game

1. **Equipment System**: Character gear and items
2. **Combat System**: Attack/defense calculations
3. **Inventory Management**: Item organization
4. **Shop System**: Economic value and trading
5. **Progression System**: Requirements for advanced gear

### File Characteristics

- **Record Size**: 284 bytes (fixed)
- **Header**: 4-byte record count
- **Encoding**: WINDOWS-1250 for all text
- **Structure**: Complex statistical system
- **Padding**: Multiple unused fields for alignment

### Technical Details

**Text Encoding:**

- WINDOWS-1250 for all text fields
- Null-terminated strings with padding
- Fixed field sizes (30 and 202 bytes)

**Binary Processing:**

- Little-endian byte order
- Fixed record size validation
- Statistical data parsing
- Requirement system handling

**Database Integration:**

- Processed by `WeaponItem` struct
- Stored with all statistical fields
- Linked to equipment and combat systems
- Complex binary format with padding

### Equipment System Analysis

**Stat Bonuses:**

- Direct attribute improvements
- Combat effectiveness modifiers
- Character customization
- Equipment specialization

**Requirement System:**

- Prevents early-game abuse
- Encourages character development
- Creates equipment progression
- Balances game difficulty

**Durability System:**

- Item wear and tear
- Repair mechanics
- Equipment longevity
- Economic considerations

### Legal Compliance

This documentation:

- Describes **file format specifications only**
- Does **not** distribute any weapon data or game content
- Focuses on **technical organization and equipment systems**
- Explains **statistical mechanics and combat properties**
- Maintains **nominal fair use** for trademark references

### Notes

- File uses binary format with WINDOWS-1250 text encoding
- Complex statistical system for equipment
- Fixed record size enables efficient parsing
- Integrated with combat and inventory systems
- **No copyrighted game content** is reproduced or distributed

### Comparison with Other Item Databases

**WeaponItem.db vs HealItem.db:**

- **WeaponItem.db**: Permanent equipment with stats
- **HealItem.db**: Consumable items with effects
- **WeaponItem.db**: Complex statistical system
- **HealItem.db**: Simple healing mechanics

**WeaponItem.db vs MiscItem.db:**

- **WeaponItem.db**: Combat equipment with requirements
- **MiscItem.db**: Utility items without stats
- **WeaponItem.db**: Character progression focus
- **MiscItem.db**: Economic/utility focus

### File Purpose Summary

WeaponItem.db serves as a comprehensive database for:

- Weapon and armor statistics
- Character equipment systems
- Combat effectiveness modifiers
- Equipment requirements and progression
- Shop and inventory management

The file provides a sophisticated system for managing all character equipment in the game, supporting both simple weapons and complex armor sets with extensive statistical properties and requirement systems.

## Extractor

An extractor is available in `src/references/weapons_db.rs` to parse this file format.

### How to Run

```bash
# Extract weaponItem.db to JSON
cargo run -- ref weapons "fixtures/Dispel/CharacterInGame/weaponItem.db"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
