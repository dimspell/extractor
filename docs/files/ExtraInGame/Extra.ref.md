# ExtraInGame/Ext*.ref Documentation

> DISPEL® is a registered trademark. This project is not affiliated with,
> endorsed by, or sponsored by the trademark owner.

## File Information

### Overview

Binary files that define the placement and configuration of interactive objects (chests, doors, signs, altars, magic
items) on specific maps. Each map has its own `Ext*.ref` file containing all interactive elements for that map.

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
- map_object_id: u16 (map-local ID; exposed as `700 + map_object_id` in the tile/object system)
- unknown1: u8 (padding, always 0)
- extra_definition_id: u8 (links to Extra.ini)
- object_name: 32 bytes (WINDOWS-1250, null-padded)
- object_type: u8
- map_x: i32
- map_y: i32
- direction: u8 (sprite-facing/frame index)
- direction_padding: 3 bytes (padding, normally [205, 205, 205])
- interaction_state: i32 (mutable state: 0 before interaction, 1 after activation/opening)
- requires_key: i32 (enables key/requirement checks; not the open/closed state)
- required_item_id: u8 (lower key bound)
- required_item_type_id: u8
- requirement_range_1_padding: i16 (padding, normally 0)
- required_item_id2: u8 (upper key bound)
- required_item_type_id2: u8
- unknown5: i16 (padding, always 0)
- requirement_range_2_start: i32 (inclusive start of the second accepted key/item range; 9999 = unused)
- requirement_range_2_end: i32 (inclusive end of the second accepted key/item range)
- requirement_range_3_start: i32 (inclusive start of the third accepted key/item range; 9999 = unused)
- requirement_range_3_end: i32 (inclusive end of the third accepted key/item range)
- gold_amount: i32
- loot_item_id: u8
- loot_item_type_id: u8
- loot_item_padding: i16 (padding, normally 0)
- loot_item_count: i32
- additional_loot_1: i32 (second loot item identifier; 9999 = unused)
- additional_loot_1_count: i32 (quantity of the second loot item)
- additional_loot_2: i32 (third loot item identifier; 9999 = unused)
- additional_loot_2_count_and_config: 28 bytes (first i32 is the third loot quantity; the remaining 24 bytes are object-specific configuration)
- interaction_event_id: i32 (links to Event.ini)
- interaction_message_id: i32 (links to Message.scr)
- footprint_width: i32 (occupied map-cell width)
- footprint_height: i32 (occupied map-cell height)
- footprint_orientation: u8 (normal/reversed footprint traversal)
- interaction_range: u8 (maximum activation distance)
- interaction_range_padding: 2 bytes (padding, normally [205, 205])
- is_quest_element: i32 (requests a quest-state refresh after a successful requirement check)
- post_activation_tile_flag: i32 (enables a post-activation map-grid tile flag)
- post_activation_footprint_mode: i32 (selects the post-activation map-grid footprint update mode)
- preserve_final_sprite_frame: i32 (prevents the terminal sprite frame from being reset after interaction)
- alternate_render_mode: i32 (selects the alternative object renderer)
- activation_effect_id: u8 (index passed to the activation-effect dispatcher; observed values 0 and 10)
- activation_effect_reserved: u8 (reserved; preserve verbatim)
- activation_effect_padding: i16 (padding, normally 0)
- active_overlay_enabled: i32 (enables the active-object overlay render path)
- map_object_active: i32 (marks the object active in the map-object grid and update loop)

[Record 2]
... (same structure) ...
```

### Field Definitions

### Field semantics

- `map_object_id` is exposed to the tile/object system as `700 + map_object_id`.
- `interaction_state` is preserved across reloads; it selects the closed/open sprite sequence for chest-type objects.
- `requires_key` enables validation of the three inclusive requirement ranges. A range starting with `9999` is disabled.
- `gold_amount` awards gold, and three `(item, count)` loot pairs are processed: `loot_item`/`loot_item_count`, then the
  two
  `additional_loot_*` pairs.
- `footprint_width`, `footprint_height`, and `footprint_orientation` determine which map cells the object occupies.
  `interaction_range` limits activation distance.

The 28-byte `additional_loot_2_count_and_config` field contains mixed configuration: its first `i32` is the third loot
quantity, its second `i32`
selects the loot-delivery mode, and its final `i32` is a remaining-use counter. The intervening 16 bytes vary by object
type.

#### Core Identification

| Field               | Type              | Description                                                                    |
|---------------------|-------------------|--------------------------------------------------------------------------------|
| record_index        | i32               | Zero-based parser-derived record position; not stored in the file              |
| map_object_id       | u16               | Map-local object ID. The engine refers to the object as `700 + map_object_id`. |
| extra_definition_id | u8                | ID of the visual/behavior definition in `Extra.ini`.                           |
| object_name         | String (32 bytes) | Author-facing object label, WINDOWS-1250 encoded and null-padded.              |

#### Position and Orientation

| Field     | Type | Description                                                                  |
|-----------|------|------------------------------------------------------------------------------|
| map_x     | i32  | Horizontal tile-grid coordinate.                                             |
| map_y     | i32  | Vertical tile-grid coordinate.                                               |
| direction | u8   | Sprite-facing/frame index; chest rendering uses it with `interaction_state`. |

#### Object Classification

| Field                | Type | Description                                                               |
|----------------------|------|---------------------------------------------------------------------------|
| object_type          | u8   | Object category enum                                                      |
| activation_effect_id | u8   | Index passed to the activation-effect dispatcher (observed: 0, 10).       |
| interaction_range    | u8   | Maximum tile distance at which the engine allows activation.              |
| is_quest_element     | i32  | Quest-related interaction flag; set after a successful requirement check. |

#### Runtime and Rendering Controls

| Field                          | Type        | Description                                                           |
|--------------------------------|-------------|-----------------------------------------------------------------------|
| post_activation_tile_flag      | BooleanFlag | Enables the post-activation tile flag written to the map-object grid. |
| post_activation_footprint_mode | BooleanFlag | Selects the footprint update mode after activation.                   |
| preserve_final_sprite_frame    | i32         | Non-zero keeps the terminal sprite frame after interaction.           |
| alternate_render_mode          | BooleanFlag | Selects the alternate renderer.                                       |
| active_overlay_enabled         | BooleanFlag | Enables the active-object overlay render path.                        |
| map_object_active              | BooleanFlag | Makes the object participate in the map-object grid and update loop.  |
| activation_effect_reserved     | u8          | Reserved byte adjacent to the effect ID; preserve verbatim.           |

#### Container State

| Field             | Type | Description                                                  |
|-------------------|------|--------------------------------------------------------------|
| interaction_state | i32  | Current activation/open state (0 before use, 1 after use)    |
| requires_key      | i32  | Whether interaction validates the configured key/item ranges |

#### Key Requirements

| Field                  | Type | Description                    |
|------------------------|------|--------------------------------|
| required_item_id       | u8   | Lower bound key ID to interact |
| required_item_type_id  | u8   | Category of lower bound key    |
| required_item_id2      | u8   | Upper bound key ID to interact |
| required_item_type_id2 | u8   | Category of upper bound key    |

#### Contents

| Field                                 | Type       | Description                                                             |
|---------------------------------------|------------|-------------------------------------------------------------------------|
| gold_amount                           | i32        | Gold awarded by the normal loot path.                                   |
| loot_item                             | packed u16 | First static loot item (`low byte = item ID`, `high byte = item type`). |
| loot_item_count                       | i32        | Quantity of `loot_item`.                                                |
| additional_loot_1 / additional_loot_2 | i32        | Further loot item IDs; `9999` means unused.                             |

#### Event Triggers

| Field                  | Type | Description                                                              |
|------------------------|------|--------------------------------------------------------------------------|
| interaction_event_id   | i32  | `Event.ini` logic executed after interaction; zero means none.           |
| interaction_message_id | i32  | `Message.scr` entry used by message/sign-style objects; zero means none. |

### Enumerations

#### Object Type (object_type)

| Value | Name        | Description                 |
|-------|-------------|-----------------------------|
| 0     | Chest       | Treasure container          |
| 2     | Door        | Passage barrier             |
| 4     | Sign        | Text display object         |
| 5     | Altar       | Religious/ritual object     |
| 6     | Interactive | General interactive element |
| 7     | Magic       | Magical object              |

#### Activation Effect ID (activation_effect_id)

| Value | Name     | Description                                                    |
|-------|----------|----------------------------------------------------------------|
| 0     | None     | No activation effect dispatched.                               |
| 10    | Effect10 | Effect index 10; the exact effect asset is not yet identified. |

#### Item Type ID (required_item_type_id, loot_item_type_id)

| Value | Name   | Description                 |
|-------|--------|-----------------------------|
| 0     | Weapon | Weapon category             |
| 1     | Armor  | Armor category              |
| 2     | Heal   | Healing item category       |
| 3     | Misc   | Miscellaneous item category |
| 4     | Edit   | Edit item category          |
| 5     | Event  | Event item category         |
| 6     | Extra  | Extra item category         |

### Interactive Element Types

| Value | Description                       |
|-------|-----------------------------------|
| 0     | Pillars (e.g., Gods garden)       |
| 1     | Standard interactive              |
| 2     | Unknown variant                   |
| 3     | Special altars (e.g., Vera altar) |

### File Purpose

These files define interactive object placements with exact coordinates, requirements, contents, and behaviors. Used for
populating maps with chests, doors, signs, and other interactive elements. Each map in the game has a corresponding
`Ext*.ref` file.

### Cross-References

| Field                  | References                               |
|------------------------|------------------------------------------|
| extra_definition_id    | `Extra.ini` (visual/behavior definition) |
| interaction_event_id   | `Event.ini` (event logic)                |
| interaction_message_id | `Message.scr` (text display)             |
| required_item_type_id  | Item type enumeration                    |
| loot_item_type_id      | Item type enumeration                    |

### Technical Details

**Endianness**: Little-Endian throughout **Record Count**: Stored as first 4 bytes (i32)
**Record Size**: Fixed 184 bytes (46 × i32 equivalent)
**File Size**: 4 + (record_count × 184) bytes **Text Encoding**: WINDOWS-1250 for name fields **Padding**: Extensive
padding with known patterns (zeros, 205 bytes)

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

### Notes

- File uses little-endian byte order throughout
- Name fields are null-terminated WINDOWS-1250 strings
- Padding bytes are often filled with the byte value 205 or zero
- Sign objects use `interaction_message_id` to display text from `Message.scr`.
- Door objects may require specific keys defined by `required_item_*` fields
- The dual key system (`required_item_id`/`required_item_id2`) supports key ranges or multiple key types

## Extractor

An extractor is available in `src/references/extra_ref.rs` to parse this file format.

### How to Run

```bash
# Extract Extdun01.ref to JSON
cargo run -- extract -i "fixtures/Dispel/ExtraInGame/Extdun01.ref"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
