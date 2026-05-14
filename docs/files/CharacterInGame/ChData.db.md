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

### Extra Points (20 bytes)

- 4 × `i32`: Warrior/Knight/Archer/Mage extra attribute points at character creation
- 1 × `i32`: Extra points per level-up

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

### Extra Points (warrior_extra_points through mage_extra_points)

- 4 × i32 — Unused in the game
- Would grant additional attribute points per class during character creation

### extra_points_per_level

- i32 — Unused in the game
- Would grant additional points on each level-up

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
Bytes 64-67:  warrior_extra_points (i32)
Bytes 68-71:  knight_extra_points (i32)
Bytes 72-75:  archer_extra_points (i32)
Bytes 76-79:  mage_extra_points (i32)
Bytes 80-83:  extra_points_per_level (i32)
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
| Bytes 64-79:  Extra points per class |
| Bytes 80-83:  Per-level points       |
+--------------------------------------+
```

## Binary Structure Details

### Byte Offsets

- `0x00-0x1D`: unused_name (30 bytes)
- `0x1E-0x3D`: Class attributes (16 × i16)
- `0x3E-0x3F`: reserved_stat (i16)
- `0x40-0x4F`: Extra points per class (4 × i32)
- `0x50-0x53`: Extra points per level (i32)

### Data Types

- `unused_name`: [u8; 30] WINDOWS-1250
- Class attributes: [i16; 16]
- `reserved_stat`: i16
- Extra points: [i32; 4]
- Per-level: i32

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
