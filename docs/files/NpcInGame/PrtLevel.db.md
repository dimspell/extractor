# PrtLevel.db - Character Progression Database

## File Information
- **Location**: `NpcInGame/PrtLevel.db`
- **Format**: Binary (Little-Endian)
- **Record Size**: 36 bytes per level
- **Total Size**: 5760 bytes (8 NPCs × 20 levels × 36 bytes)
- **No Header**: Direct data structure

## File Structure

### Overall Structure
- 8 NPC slots (party size limit)
- 20 levels per NPC (levels 1-20)
- 36 bytes per level record
- Total: 8 × 20 × 36 = 5760 bytes

### Level Record Structure (36 bytes)
- `sentinel`: u32 (4 bytes) - Record marker (typically 0)
- `strength`: u32 (4 bytes) - Physical damage output
- `constitution`: u32 (4 bytes) - Health point scaling
- `wisdom`: u32 (4 bytes) - Mana point scaling
- `health_points`: u16 (2 bytes) - Base health points
- `mana_points`: u16 (2 bytes) - Base mana points
- `agility`: u32 (4 bytes) - Evasion and speed
- `attack`: u32 (4 bytes) - Combat accuracy
- `mana_recharge`: u32 (4 bytes) - Mana regeneration rate
- `defense`: u16 (2 bytes) - Damage resistance
- `padding`: u16 (2 bytes) - Null byte padding

## Stat Growth Patterns

### Strength
- Physical damage output
- Affects melee attack power
- Scales with weapon damage

### Constitution
- Health point scaling
- Determines maximum HP
- Affects survivability

### Wisdom
- Mana point scaling
- Determines maximum MP
- Affects spellcasting capacity

### Agility
- Evasion and speed
- Affects dodge chance
- Influences attack speed

### Attack
- Combat accuracy
- Affects hit chance
- Influences critical hit rate

### Defense
- Damage resistance
- Reduces incoming damage
- Affects armor effectiveness

### Mana Recharge
- Mana regeneration rate
- Affects MP recovery speed
- Influences sustained spellcasting

## Level Ranges
- **Levels 1-20**: Standard progression path
- Each level adds fixed stat increases
- Growth curves vary by character class
- Linear progression within each NPC slot

## Special Values
- `sentinel = 0`: Standard record marker
- Fixed 20 levels per NPC
- 8 NPC slots (party size limit)
- 5760-byte total file size
- Null byte padding at end of each record

## File Purpose
Defines character progression statistics for levels 1-20, used for:
- Level-up calculations
- Stat growth and development
- Character balancing
- Game difficulty scaling
- Party composition planning

## Implementation
- **Rust Module**: `src/references/party_level_db.rs`
- **Extractor**: `PartyLevelNpc` struct implementing `Extractor` trait
- **Data Structures**:
  - `PartyLevelNpc` - NPC progression data
  - `PartyLevelRecord` - Individual level statistics
- **Database**: Saved to SQLite via `save_party_levels` function

## Example Usage

### Extract and display progression data:
```bash
cargo run -- ref party-level "fixtures/Dispel/NpcInGame/PrtLevel.db"
```

### Import to database:
```bash
cargo run -- database import "fixtures/Dispel"
```

## Progression System
- **Linear Growth**: Stats increase predictably with each level
- **Class Specialization**: Each NPC slot has unique stat priorities
- **Balanced Design**: Different classes excel in different areas
- **Level Cap**: Maximum level 20 for all characters

## Related Files
- `PrtIni.db` - Party initialization data
- `Npc.ini` - NPC definitions
- Character sprite files (`.spr`)
