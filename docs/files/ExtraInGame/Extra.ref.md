# ExtraInGame/Ext*.ref Documentation

## File Format: Interactive Object Placements

### Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, assets, or proprietary data. All references are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

### Overview

Binary files that define the placement and configuration of interactive objects (chests, doors, signs, altars, magic items) on specific maps. Each map has its own `Ext*.ref` file containing all interactive elements for that map.

### File Structure

**Location**: `ExtraInGame/Ext*.ref` (e.g., `Extmap01.ref`, `Extdun01.ref`)
**Encoding**: Binary (Little-Endian)
**Text Encoding**: WINDOWS-1250 (for name fields)
**Header**: 4-byte record count (i32)
**Record Size**: 184 bytes per record

### Format Specification

```
[Header]
- record_count: i32 (number of records)

[Record 1]
- number_in_file: u8
- unknown1: u8 (padding, always 0)
- ext_id: u8 (links to Extra.ini)
- name: 32 bytes (WINDOWS-1250, null-padded)
- object_type: u8
- x_pos: i32
- y_pos: i32
- rotation: u8
- unknown2: 3 bytes (padding, always [205, 205, 205])
- unknown3: i32 (padding, always 0)
- closed: i32 (0=open, 1=closed)
- required_item_id: u8 (lower key bound)
- required_item_type_id: u8
- unknown4: i16 (padding, always 0)
- required_item_id2: u8 (upper key bound)
- required_item_type_id2: u8
- unknown5: i16 (padding, always 0)
- unknown6: i32 (0 or 9999)
- unknown7: i32 (0 or 9999)
- unknown8: i32 (0 or 9999)
- unknown9: i32 (0 or 9999)
- gold_amount: i32
- item_id: u8
- item_type_id: u8
- unknown10: i16 (padding, always 0)
- item_count: i32
- unknown11: i32 (0, 28, 84, 258, 9999)
- unknown12: i32 (0 or 1)
- unknown13: i32 (0 or 9999)
- unknown14: 28 bytes (padding, always zeros)
- event_id: i32 (links to Event.ini)
- message_id: i32 (links to Message.scr)
- unknown15: i32 (0, 1, 2, 3)
- unknown16: i32 (0, 1, 2, 3)
- unknown17: u8 (always 0)
- interactive_element_type: u8 (0, 1, 2, 3)
- unknown18: 2 bytes (padding, always [205, 205])
- is_quest_element: i32 (0 or 1)
- unknown20: i32 (0 or 1)
- unknown21: i32 (0 or 1)
- unknown22: i32 (always 0)
- unknown23: i32 (0 or 1)
- visibility: u8
- unknown24: u8 (0 or 1)
- unknown25: i16 (always 0)
- unknown26: i32 (0 or 1)
- unknown27: i32 (0 or 1)

[Record 2]
... (same structure) ...
```

### Field Definitions

#### Core Identification

| Field | Type | Description |
|-------|------|-------------|
| id | i32 | Record index (0-based, derived from position) |
| number_in_file | u8 | Sequential index within the file |
| ext_id | u8 | Links to Extra.ini entry for object definition |
| name | String (32 bytes) | Object label, WINDOWS-1250 encoded, null-padded |

#### Position and Orientation

| Field | Type | Description |
|-------|------|-------------|
| x_pos | i32 | Horizontal tile coordinate on the map |
| y_pos | i32 | Vertical tile coordinate on the map |
| rotation | u8 | Object facing direction (0-7, 8 directions) |

#### Object Classification

| Field | Type | Description |
|-------|------|-------------|
| object_type | u8 | Object category enum |
| visibility | u8 | Rendering transparency/visibility type |
| interactive_element_type | u8 | Interaction behavior modifier |
| is_quest_element | i32 | Whether object is part of quest logic |

#### Container State

| Field | Type | Description |
|-------|------|-------------|
| closed | i32 | Container state (0=open, 1=closed) |

#### Key Requirements

| Field | Type | Description |
|-------|------|-------------|
| required_item_id | u8 | Lower bound key ID to interact |
| required_item_type_id | u8 | Category of lower bound key |
| required_item_id2 | u8 | Upper bound key ID to interact |
| required_item_type_id2 | u8 | Category of upper bound key |

#### Contents

| Field | Type | Description |
|-------|------|-------------|
| gold_amount | i32 | Amount of gold contained in object |
| item_id | u8 | Static loot item identifier |
| item_type_id | u8 | Category of loot item |
| item_count | i32 | Stack quantity of loot |

#### Event Triggers

| Field | Type | Description |
|-------|------|-------------|
| event_id | i32 | Event.ini entry triggered on interaction |
| message_id | i32 | Message.scr entry for sign text display |

### Enumerations

#### Object Type (object_type)

| Value | Name | Description |
|-------|------|-------------|
| 0 | Chest | Treasure container |
| 2 | Door | Passage barrier |
| 4 | Sign | Text display object |
| 5 | Altar | Religious/ritual object |
| 6 | Interactive | General interactive element |
| 7 | Magic | Magical object |

#### Visibility Type (visibility)

| Value | Name | Description |
|-------|------|-------------|
| 0 | Visible0 | Standard visibility |
| 10 | Visible10 | Alternative visibility state |

#### Item Type ID (required_item_type_id, item_type_id)

| Value | Name | Description |
|-------|------|-------------|
| 0 | Weapon | Weapon category |
| 1 | Armor | Armor category |
| 2 | Heal | Healing item category |
| 3 | Misc | Miscellaneous item category |
| 4 | Edit | Edit item category |
| 5 | Event | Event item category |
| 6 | Extra | Extra item category |

### Interactive Element Types

| Value | Description |
|-------|-------------|
| 0 | Pillars (e.g., Gods garden) |
| 1 | Standard interactive |
| 2 | Unknown variant |
| 3 | Special altars (e.g., Vera altar) |

### File Purpose

These files define interactive object placements with exact coordinates, requirements, contents, and behaviors. Used for populating maps with chests, doors, signs, and other interactive elements. Each map in the game has a corresponding `Ext*.ref` file.

### Usage in Game

1. **Map Loading**: When a map loads, the game reads the corresponding `Ext*.ref` file
2. **Object Placement**: Objects are placed at specified x/y coordinates
3. **Interaction Handling**: Game checks requirements (keys, items) before allowing interaction
4. **Event Triggering**: Interaction triggers events from `Event.ini`
5. **Content Distribution**: Containers provide gold and items to players
6. **Quest Integration**: Quest elements are tracked separately via `is_quest_element` flag

### Cross-References

| Field | References |
|-------|------------|
| ext_id | `Extra.ini` (object definitions) |
| event_id | `Event.ini` (event logic) |
| message_id | `Message.scr` (text display) |
| required_item_type_id | Item type enumeration |
| item_type_id | Item type enumeration |

### Technical Details

**Endianness**: Little-Endian throughout
**Record Count**: Stored as first 4 bytes (i32)
**Record Size**: Fixed 184 bytes (46 × i32 equivalent)
**File Size**: 4 + (record_count × 184) bytes
**Text Encoding**: WINDOWS-1250 for name fields
**Padding**: Extensive padding with known patterns (zeros, 205/0xCD bytes)

### File Processing

1. Read 4-byte record count header
2. Calculate expected file size: 4 + (count × 184)
3. Parse each 184-byte record sequentially
4. Decode name fields using WINDOWS-1250 encoding
5. Apply enum conversions for typed fields

### Characteristics

- **Fixed Record Size**: All records are exactly 184 bytes
- **Binary Format**: Not human-readable without parsing
- **Map-Specific**: Each map has its own file
- **Extensive Padding**: Many unknown fields with consistent patterns
- **Cross-Referenced**: Links to multiple other game data files
- **Quest-Aware**: Special flag for quest-related objects
- **Container Support**: Built-in gold and item storage
- **Key System**: Dual key requirement system (lower/upper bounds)

### Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any game content or assets
- Uses **generic descriptions** of objects and behaviors
- Focuses on **technical organization**, not creative content
- Maintains **nominal fair use** for trademark references

### Notes

- File uses little-endian byte order throughout
- Name fields are null-terminated WINDOWS-1250 strings
- Padding bytes often contain 0xCD (205) or 0x00 patterns
- The `closed` field primarily applies to chest-type objects
- Sign objects use `message_id` to display text from `Message.scr`
- Door objects may require specific keys defined by `required_item_*` fields
- Magic objects (type 7) likely represent spell-related interactables
- The dual key system (`required_item_id`/`required_item_id2`) may support key ranges or multiple key types

## Extractor

An extractor is available in `src/references/extra_ref.rs` to parse this file format.

### How to Run

```bash
# Extract Extdun01.ref to JSON
cargo run -- ref extra-ref "fixtures/Dispel/ExtraInGame/Extdun01.ref"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
