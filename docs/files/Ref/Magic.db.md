# Magic.db - Spell Database

## File Information
- **Location**: `MagicInGame/Magic.db` or `MulMagic.db`
- **Format**: Binary (Little-Endian)
- **Record Size**: 88 bytes (22 × u32)
- **No Header**: Record count derived from file size

## File Structure

### Record Structure (88 bytes)
- `enabled`: u32 - Spell availability (0=disabled, 1=enabled)
- `flag1`: u32 - Validation flag (always 1 for valid spells)
- `mana_cost`: u32 - Mana cost (999=unlimited/special)
- `success_rate`: u32 - Accuracy percentage (0-100%)
- `base_damage`: u32 - Primary effect value
- `reserved1`: u32 - Reserved field (always 0)
- `reserved2`: u32 - Reserved field (always 0)
- `flag2`: u32 - Unknown flag (0 or 1)
- `range`: u32 - Range/duration (999=unlimited)
- `reserved3`: u32 - Reserved field (always 0)
- `level_required`: u32 - Minimum level to learn/cast
- `constant1`: u32 - Constant value (always 1)
- `effect_value`: u32 - Secondary effect value
- `effect_type`: u32 - Effect type identifier
- `effect_modifier`: u32 - Effect modifier value
- `reserved4`: u32 - Reserved field (always 0)
- `magic_school`: u32 - School of magic (0-6)
- `flag3`: u32 - Unknown flag (0 or 1)
- `animation_id`: u32 - Visual effect identifier
- `visual_id`: u32 - Sound/visual reference
- `icon_id`: u32 - UI icon identifier
- `target_type`: u32 - Targeting mode (1-4)

## Field Details

### enabled
- `0`: Spell disabled/unavailable
- `1`: Spell enabled/available
- Controls spell accessibility

### mana_cost
- Mana points required to cast
- `999`: Special/unlimited mana
- Balances spell power vs. resource cost

### success_rate
- Accuracy percentage (0-100)
- Chance of successful casting
- Affects spell reliability

### base_damage
- Primary effect magnitude
- Damage amount for offensive spells
- Healing amount for restorative spells
- Effect strength for other spell types

### level_required
- Minimum character level
- Prerequisite for learning/casting
- Controls spell progression

### effect_type
- Determines spell behavior
- Links to spell effect system
- Defines what the spell actually does

### effect_value
- Secondary effect magnitude
- Additional effect parameters
- Modifies primary effect

### effect_modifier
- Effect adjustment value
- Fine-tunes spell behavior
- Context-dependent meaning

### magic_school

### target_type
- `1`: Single target
- `2`: Self (caster)
- `3`: Area of effect (AoE)
- `4`: Multi-target

### animation_id
- Visual effect identifier
- Links to animation system
- Determines spell visuals

### visual_id
- Sound and visual reference
- Audio-visual effects
- Particle systems

### icon_id
- UI icon identifier
- Spell menu representation
- Quick access icons

## File Purpose
Complete spell database defining all magical abilities with statistics, requirements, effects, and visual/audio assets. Used for:
- Combat magic system
- Character progression
- Spellcasting mechanics
- Magic-based gameplay
- Balancing and difficulty scaling



## Implementation
- **Rust Module**: `src/references/magic_db.rs`
- **Extractor**: `MagicSpell` struct implementing `Extractor` trait
- **Data Structure**: `MagicSpell` with comprehensive spell attributes
- **Database**: Saved to SQLite via `save_magic_spells` function

## Example Usage

### Extract and display spells:
```bash
cargo run -- extract -i "Dispel/MagicInGame/Magic.db"
```

### Import to database:
```bash
cargo run -- database import "Dispel/"
```

## Extractor

An extractor is available in `src/references/magic_db.rs` to parse this file format.

### How to Run

```bash
# Extract Magic.db to JSON
cargo run -- extract -i "fixtures/Dispel/Ref/Magic.db"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
