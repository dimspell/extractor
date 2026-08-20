# ChData.db - Character Initial Attributes

## File Information

- **Location**: `CharacterInGame/ChData.db`
- **Format**: Binary (Little-Endian)
- **Record Size**: 84 bytes
- **Single-record file**: Contains one record with starting stats per class

## File Structure

### Name Section (30 bytes)

- `unused_name`: 30 bytes (WINDOWS-1250)

### Class Base Attributes (32 bytes)

16 × `i16` — Base attributes (STR, CON, WIS, AGI) for each of 4 classes:

| Offset | Class   | Fields (each i16)  |
| ------ | ------- | ------------------ |
| 30     | Warrior | STR, CON, WIS, AGI |
| 38     | Knight  | STR, CON, WIS, AGI |
| 46     | Archer  | STR, CON, WIS, AGI |
| 54     | Mage    | STR, CON, WIS, AGI |

### Reserved (2 bytes)

- `reserved_stat`: i16 — Separator between base stats and extra points (purpose unknown)

### Derived Stat Bonuses (20 bytes)

5 × `i32` — per-class derived-stat bonuses applied at character creation. The game reads only the low byte of each (values are small, e.g. 5). Each bonus is added to a derived combat stat:

| Offset | Class | Field | Derived stat | Base attr |
| ------ | ----- | ----- | ------------ | --------- |
| 64 | Warrior | warrior_offense_bonus | offense | STR |
| 68 | Knight | knight_defense_bonus | defense | AGI |
| 72 | Archer | archer_dodge_bonus | dodge_rate | CON |
| 76 | Archer | archer_hit_bonus | hit_rate | CON |
| 80 | Mage | mage_magic_power_bonus | magic_power | WIS |

> Points-per-level is hardcoded to 5 in the game and is NOT read from this file.

## Field Details

### unused_name

- 30-byte WINDOWS-1250 encoded string
- Starts with "Item" magic signature
- Otherwise unused by the game

### Class Attributes (warrior_strength through mage_agility)

- 16 × i16 signed values
- STR (Strength), CON (Constitution), WIS (Wisdom), AGI (Agility)
- One set per character class (Warrior, Knight, Archer, Mage)

### reserved_stat

- i16 between class attributes and extra points
- Value in the game appears to be ignored

### Derived Stat Bonuses (warrior_offense_bonus through mage_magic_power_bonus)

- 5 × i32 — per-class derived-stat bonuses. Each is added to a derived combat stat (offense, defense, dodge_rate, hit_rate, magic_power) for the corresponding class.

### mage_magic_power_bonus

- i32 — Mage class bonus added to magic_power (WIS-derived).

## Example Usage

### Extract and display character data:

```bash
cargo run -- extract -i "fixtures/Dispel/CharacterInGame/ChData.db"
```

### Format Structure

```
Bytes 0-29:   unused_name (30 bytes, WINDOWS-1250)
Bytes 30-31:  warrior_strength (i16)
Bytes 32-33:  warrior_constitution (i16)
Bytes 34-35:  warrior_wisdom (i16)
Bytes 36-37:  warrior_agility (i16)
Bytes 38-39:  knight_strength (i16)
Bytes 40-41:  knight_constitution (i16)
Bytes 42-43:  knight_wisdom (i16)
Bytes 44-45:  knight_agility (i16)
Bytes 46-47:  archer_strength (i16)
Bytes 48-49:  archer_constitution (i16)
Bytes 50-51:  archer_wisdom (i16)
Bytes 52-53:  archer_agility (i16)
Bytes 54-55:  mage_strength (i16)
Bytes 56-57:  mage_constitution (i16)
Bytes 58-59:  mage_wisdom (i16)
Bytes 60-61:  mage_agility (i16)
Bytes 62-63:  reserved_stat (i16)
Bytes 64-67:  warrior_offense_bonus (i32)
Bytes 68-71:  knight_defense_bonus (i32)
Bytes 72-75:  archer_dodge_bonus (i32)
Bytes 76-79:  archer_hit_bonus (i32)
Bytes 80-83:  mage_magic_power_bonus (i32)
```

## File Layout Visualization

```
+--------------------------------------+
| ChData.db File Structure (84 bytes)  |
+--------------------------------------+
| Bytes  0-29:  unused_name (string)   |
| Bytes 30-37:  Warrior STR/CON/WIS/AGI|
| Bytes 38-45:  Knight  STR/CON/WIS/AGI|
| Bytes 46-53:  Archer  STR/CON/WIS/AGI|
| Bytes 54-61:  Mage    STR/CON/WIS/AGI|
| Bytes 62-63:  reserved_stat          |
| Bytes 64-79:  Derived stat bonuses |
| Bytes 80-83:  Mage magic power bonus |
+--------------------------------------+
```

## Binary Structure Details

### Byte Offsets

- `0x00-0x1D`: unused_name (30 bytes)
- `0x1E-0x3D`: Class attributes (16 × i16)
- `0x3E-0x3F`: reserved_stat (i16)
- `0x40-0x4F`: Derived stat bonuses (4 × i32)
- `0x50-0x53`: Mage magic power bonus (i32)

### Data Types

- `unused_name`: [u8; 30] WINDOWS-1250
- Class attributes: [i16; 16]
- `reserved_stat`: i16
- Derived stat bonuses: [i32; 4]
- Mage magic power bonus: i32

### Endianness

- All numeric values: Little-Endian
- Standard x86 format

## Extractor

An extractor is available in `src/references/chdata_db.rs` to parse this file format.

### How to Run

```bash
# Extract ChData.db to JSON
cargo run -- extract -i "fixtures/Dispel/CharacterInGame/ChData.db"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
