# Monster.db Documentation

## File Format: Monster Statistics Database

### Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, monster data, or proprietary assets. All references to creature systems are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Overview

Binary database file that defines all monster types with complete combat statistics, behavior patterns, and reward systems for the game's monster spawning and combat AI.

### File Structure

**Location**: `MonsterInGame/Monster.db`
**Encoding**: Binary (Little-Endian)
**Text Encoding**: EUC-KR
**Record Size**: 160 bytes (40 × 4-byte fields)
**Header**: None (count derived from file size)
**Total Records**: Variable (file_size / 160)

### Binary Format

```
[No Header - count from file size]

[Records: 160 bytes each]
- name: 24 bytes (EUC-KR, null-padded)
- health_points_max: i32
- health_points_min: i32
- mana_points_max: i32
- mana_points_min: i32
- walk_speed: i32
- to_hit_max: i32
- to_hit_min: i32
- to_dodge_max: i32
- to_dodge_min: i32
- offense_max: i32
- offense_min: i32
- defense_max: i32
- defense_min: i32
- magic_attack_max: i32
- magic_attack_min: i32
- is_undead: i32 (PropertyFlag)
- has_blood: i32 (PropertyFlag)
- ai_type: i32 (MonsterAiType)
- exp_gain_max: i32
- exp_gain_min: i32
- gold_drop_max: i32
- gold_drop_min: i32
- detection_sight_size: i32
- distance_range_size: i32
- known_spell_slot1: i32
- known_spell_slot2: i32
- known_spell_slot3: i32
- is_oversize: i32
- magic_level: i32
- special_attack: i32
- special_attack_chance: i32
- special_attack_duration: i32
- boldness: i32
- attack_speed: i32
```

### Field Definitions

| Field | Size | Type | Description |
|-------|------|------|-------------|
| id | N/A | i32 | Record index (assigned during parsing) |
| name | 24 | string | Monster name (EUC-KR encoded) |
| health_points_max | 4 | i32 | Maximum HP ceiling |
| health_points_min | 4 | i32 | Minimum HP floor |
| mana_points_max | 4 | i32 | Maximum MP limit |
| mana_points_min | 4 | i32 | Minimum MP limit |
| walk_speed | 4 | i32 | Baseline tiles moved per tick |
| to_hit_max | 4 | i32 | Upper bound of accuracy |
| to_hit_min | 4 | i32 | Lower bound of accuracy |
| to_dodge_max | 4 | i32 | Upper bound for evasion rate |
| to_dodge_min | 4 | i32 | Lower bound for evasion rate |
| offense_max | 4 | i32 | Maximum physical damage |
| offense_min | 4 | i32 | Minimum physical damage |
| defense_max | 4 | i32 | Upper bound armor class |
| defense_min | 4 | i32 | Lower bound armor class |
| magic_attack_max | 4 | i32 | Maximum magical intensity |
| magic_attack_min | 4 | i32 | Minimum magical intensity |
| is_undead | 4 | i32 | Undead flag (PropertyFlag) |
| has_blood | 4 | i32 | Blood flag (PropertyFlag) |
| ai_type | 4 | i32 | Combat behavior (MonsterAiType) |
| exp_gain_max | 4 | i32 | High roll for experience points |
| exp_gain_min | 4 | i32 | Low roll for experience points |
| gold_drop_max | 4 | i32 | Maximum gold drop |
| gold_drop_min | 4 | i32 | Minimum gold drop |
| detection_sight_size | 4 | i32 | Aggro radius in tiles |
| distance_range_size | 4 | i32 | Maximum engage distance |
| known_spell_slot1 | 4 | i32 | Primary magic spell index |
| known_spell_slot2 | 4 | i32 | Secondary magic spell index |
| known_spell_slot3 | 4 | i32 | Tertiary magic spell index |
| is_oversize | 4 | i32 | Oversize collision flag |
| magic_level | 4 | i32 | Spellcasting potency |
| special_attack | 4 | i32 | Unique monster skill ID |
| special_attack_chance | 4 | i32 | Percentage likelihood |
| special_attack_duration | 4 | i32 | Effect duration in ticks |
| boldness | 4 | i32 | Retreat threshold metric |
| attack_speed | 4 | i32 | Delay ticks between attacks |

### Enumeration Types

**PropertyFlag (is_undead, has_blood):**
- **0**: Absent - Property is false/absent
- **1**: Present - Property is true/present

**MonsterAiType (ai_type):**
- **0**: Passive - No AI, doesn't attack
- **1**: Aggressive - Attacks on sight
- **2**: Defensive - Attacks when provoked
- **3**: Ranged - Uses ranged attacks
- **4**: Boss - Special boss behavior
- **5**: Special - Unique AI patterns
- **6**: Custom - Scripted AI behavior

### Data Structure

The codebase defines the monster structure as:

```rust
pub struct Monster {
    id: i32,                    // Record index
    name: String,              // Monster name (24 chars max)
    health_points_max: i32,   // Maximum HP
    health_points_min: i32,   // Minimum HP
    mana_points_max: i32,     // Maximum MP
    mana_points_min: i32,     // Minimum MP
    walk_speed: i32,          // Movement speed
    to_hit_max: i32,           // Max accuracy
    to_hit_min: i32,           // Min accuracy
    to_dodge_max: i32,        // Max evasion (usually 10)
    to_dodge_min: i32,        // Min evasion (usually 10)
    offense_max: i32,         // Max physical damage
    offense_min: i32,         // Min physical damage
    defense_max: i32,         // Max armor class
    defense_min: i32,         // Min armor class
    magic_attack_max: i32,   // Max magic damage
    magic_attack_min: i32,   // Min magic damage
    is_undead: PropertyFlag,   // Undead status
    has_blood: PropertyFlag,   // Blood presence
    ai_type: MonsterAiType,    // AI behavior type
    exp_gain_max: i32,        // Max EXP reward
    exp_gain_min: i32,        // Min EXP reward
    gold_drop_max: i32,       // Max gold reward
    gold_drop_min: i32,       // Min gold reward
    detection_sight_size: i32, // Aggro radius
    distance_range_size: i32,  // Attack range
    known_spell_slot1: i32,   // Spell 1 ID
    known_spell_slot2: i32,   // Spell 2 ID
    known_spell_slot3: i32,   // Spell 3 ID
    is_oversize: i32,         // Large monster flag
    magic_level: i32,         // Magic potency (usually 1)
    special_attack: i32,      // Special attack ID
    special_attack_chance: i32, // Special attack probability
    special_attack_duration: i32, // Special attack duration
    boldness: i32,            // Courage/retreat threshold (usually 10)
    attack_speed: i32,       // Attack cooldown
}
```

### Binary Record Layout

```
Offset | Size | Field | Description
-------|------|-------|-------------
0      | 24   | name  | Null-padded EUC-KR string
24     | 4    | hp_max | Maximum HP (i32)
28     | 4    | hp_min | Minimum HP (i32)
32     | 4    | mp_max | Maximum MP (i32)
36     | 4    | mp_min | Minimum MP (i32)
40     | 4    | speed | Walk speed (i32)
44     | 4    | hit_max | Max accuracy (i32)
48     | 4    | hit_min | Min accuracy (i32)
52     | 4    | dodge_max | Max evasion (i32)
56     | 4    | dodge_min | Min evasion (i32)
60     | 4    | off_max | Max physical damage (i32)
64     | 4    | off_min | Min physical damage (i32)
68     | 4    | def_max | Max armor class (i32)
72     | 4    | def_min | Min armor class (i32)
76     | 4    | mag_max | Max magic damage (i32)
80     | 4    | mag_min | Min magic damage (i32)
84     | 4    | undead | Undead flag (i32)
88     | 4    | blood | Blood flag (i32)
92     | 4    | ai_type | AI behavior (i32)
96     | 4    | exp_max | Max EXP reward (i32)
100    | 4    | exp_min | Min EXP reward (i32)
104    | 4    | gold_max | Max gold reward (i32)
108    | 4    | gold_min | Min gold reward (i32)
112    | 4    | sight | Detection range (i32)
116    | 4    | range | Attack range (i32)
120    | 4    | spell1 | Spell slot 1 (i32)
124    | 4    | spell2 | Spell slot 2 (i32)
128    | 4    | spell3 | Spell slot 3 (i32)
132    | 4    | oversize | Oversize flag (i32)
136    | 4    | mag_lvl | Magic level (i32)
140    | 4    | spec_atk | Special attack ID (i32)
144    | 4    | spec_chance | Special attack chance (i32)
148    | 4    | spec_dur | Special attack duration (i32)
152    | 4    | bold | Boldness/courage (i32)
156    | 4    | atk_spd | Attack speed (i32)
```

### Special Values

- **to_dodge_max/min**: Usually both = 10
- **magic_level**: Usually = 1
- **boldness**: Usually = 10
- **is_undead = 1**: Holy weakness, no bleed
- **has_blood = 0**: No blood effects (golems)
- **is_oversize = 1**: Large monsters (dragons, etc.)
- **ai_type = 1**: Melee fighters (goblins, chickens)
- **ai_type = 2**: Archers
- **ai_type = 3**: Casters (worms, zombies)
- **ai_type = 5**: Passive (deer, dogs)

### Usage in Game

1. **Combat System**: Monster stats for battles
2. **AI System**: Behavior patterns and tactics
3. **Reward System**: EXP and gold drops
4. **Spawning System**: Monster placement and variety
5. **Progression System**: Balanced difficulty curve

### File Characteristics

- **Record Size**: 160 bytes (fixed)
- **No Header**: Record count derived from file size
- **Encoding**: EUC-KR for monster names
- **Structure**: Comprehensive statistical system
- **Complexity**: 40 fields per monster

### Technical Details

**Text Encoding:**
- EUC-KR for monster names
- Null-terminated strings with padding
- Fixed field size (24 bytes)

**Binary Processing:**
- Little-endian byte order
- No header validation
- Fixed record size parsing
- Enum conversion for flags

**Database Integration:**
- Processed by `Monster` struct
- Uses type-safe enums for flags
- Stored with all statistical fields
- Linked to combat and spawning systems

### Combat System Analysis

**Vital Statistics:**
- HP/MP ranges for variability
- Walk speed affects movement
- Accuracy and evasion mechanics

**Damage Systems:**
- Physical offense/defense
- Magical attack ranges
- Special attack mechanics

**Reward Systems:**
- EXP and gold drop ranges
- Balanced risk/reward
- Progression scaling

**AI Systems:**
- Behavior type patterns
- Detection and range mechanics
- Special abilities

### Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any monster data or game content
- Focuses on **technical organization and combat systems**
- Explains **statistical mechanics and AI behaviors**
- Maintains **nominal fair use** for trademark references

### Notes

- File uses binary format with EUC-KR text encoding
- No header - record count calculated from file size
- Complex statistical system for balanced combat
- Integrated with spawning and AI systems
- **No copyrighted game content** is reproduced or distributed

### Comparison with Other Databases

**Monster.db vs Mondun/Monmap.ref:**
- **Monster.db**: Monster statistics and AI
- **Mondun/Monmap.ref**: Monster placements
- **Monster.db**: Combat properties
- **Mondun/Monmap.ref**: Spatial positioning

**Monster.db vs Npc.ini:**
- **Monster.db**: Combat monsters
- **Npc.ini**: Non-combat NPCs
- **Monster.db**: Aggressive AI
- **Npc.ini**: Passive interactions

### File Purpose Summary

Monster.db serves as a comprehensive database for:
- Monster combat statistics
- AI behavior patterns
- Reward systems (EXP/gold)
- Special abilities and spells
- Balanced difficulty progression

The file provides a sophisticated system for managing all monster types in the game, supporting diverse combat encounters with varied statistics, behaviors, and reward structures.

## Extractor

An extractor is available in `src/references/monster_db.rs` to parse this file format.

### How to Run

```bash
# Extract Monster.db to JSON
cargo run -- extract -i "fixtures/Dispel/MonsterInGame/Monster.db"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```