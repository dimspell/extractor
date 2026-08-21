# Monster.ini

> DISPEL® is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
> owner.

## File Information

### Overview

Text file that defines monster visual appearances and animation sequences for the game's monster rendering system.

### File Structure

**Location**: `Monster.ini`
**Encoding**: WINDOWS-1250 (Central European character encoding)
**Format**: CSV (Comma-Separated Values) with comments

### Format Specification

```ini
; Comment line explaining field structure
id,name,sprite_filename,attack_seq,hit_seq,death_seq,walk_seq,cast_seq
0,null,null,0,0,0,0,0
1,MonsterName,monster.spr,101,102,103,104,105
2,MonsterName,monster.spr,106,107,108,109,110
...
```

### Field Definitions

| Field           | Type   | Description                             |
|-----------------|--------|-----------------------------------------|
| id              | i32    | Unique monster visual type identifier   |
| name            | string | Monster display name or "null"          |
| sprite_filename | string | SPR filename or "null" for no sprite    |
| attack_seq      | i32    | Animation sequence ID for attacking     |
| hit_seq         | i32    | Animation sequence ID for taking damage |
| death_seq       | i32    | Animation sequence ID for dying         |
| walk_seq        | i32    | Animation sequence ID for walking       |
| cast_seq        | i32    | Animation sequence ID for spellcasting  |

### Animation System

The file maps monster types to animation sequences that reference frame indices in SPR files:

**Animation Sequence IDs:**

- **0**: No animation (unused sequence)
- **1-N**: Frame sequence numbers in SPR file
- **-1**: Special/alternative animation sequence
- **Positive numbers**: Link to specific animation frames

### Monster Categories

The file organizes monsters into logical groups based on the comment header:

```ini
; (number, name, file, attack, hit, death, approach, magic)
```

**Example Monsters:**

- ID 0: Default/null entry (no monster)
- IDs 1+: Various monster types with sprites and animations

### Animation Sequence Details

**Combat Animations:**

- `attack_seq`: Attacking/offensive animations
- `hit_seq`: Damage/taking hit animations
- `death_seq`: Death/dying animations

**Movement Animations:**

- `walk_seq`: Walking/movement animations
- `cast_seq`: Spellcasting/magic animations

### Special Values

- **"null"**: Literal string indicating no name or sprite
- **0**: Unused animation sequence (no animation)
- **-1**: Special or alternative animation sequence
- **;**: Lines starting with semicolon are comments
- **Empty lines**: Ignored during processing

### Example Format

```ini
; Default entry (no monster)
0,null,null,0,0,0,0,0

; Basic monster template
1,MonsterType,monster.spr,201,202,203,204,205
2,MonsterType,monster.spr,206,207,208,209,210

; Monster with special casting animation
30,EliteMonster,elite.spr,251,252,253,254,-1
```

### Technical Details

**Encoding**: WINDOWS-1250 (Central European)

- Supports Polish characters in comments and names
- Requires proper encoding handling for reading/writing

**File Processing**:

- Comments (lines starting with ";") are ignored
- Empty lines are skipped
- CSV format with comma delimiter
- "null" literal used for missing name/sprite fields

**Database Integration**:

- Processed by `MonsterIni` struct in the codebase
- Stored in database with all field mappings
- Linked to monster placement files (mondun*.ref, monmap*.ref)
- Referenced by combat and rendering systems

### Animation Sequence System

The animation sequences link to specific frame ranges in SPR files:

```rust
pub struct MonsterIni {
    id: i32,                    // Monster ID
    name: Option<String>,       // Display name
    sprite_filename: Option<String>, // SPR file
    attack: i32,               // Attack animation sequence
    hit: i32,                   // Hit animation sequence
    death: i32,                // Death animation sequence
    walking: i32,              // Walking animation sequence
    casting_magic: i32,        // Casting animation sequence
}
```

### Usage in Game

1. **Monster Rendering**: Game loads visual definitions from Monster.ini
2. **Animation Mapping**: Links monster IDs to animation sequences
3. **Combat System**: Uses attack/hit/death animations during battles
4. **Movement System**: Applies walking animations during navigation
5. **Magic System**: Triggers casting animations for spellcasters

### Notes

- File uses Windows-style line endings (\r\n)
- Comments are in Polish (WINDOWS-1250 encoding)
- Animation sequences link to SPR file frame indices
- Integrated with monster placement and combat systems

## Extractor

An extractor is available in `src/references/monster_ini.rs` to parse this file format.

### How to Run

```bash
# Extract Monster.ini to JSON
cargo run -- extract -i "fixtures/Dispel/CharacterInGame/Monster.ini"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
