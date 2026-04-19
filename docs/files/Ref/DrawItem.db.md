# DrawItem.db - Map Object Placements

## File Information
- **Location**: `Ref/DrawItem.db`
- **Format**: Parenthesized CSV
- **Encoding**: EUC-KR
- **Record Size**: Variable (text)

## File Structure

### File Format
- Lines starting with `;` are comments
- Parenthesized CSV format: `(field1,field2,field3,field4)`
- Empty lines are ignored

### Record Structure
- `map_id`: i32 - Target map identifier
- `x_coord`: i32 - Tile X coordinate (isometric)
- `y_coord`: i32 - Tile Y coordinate (isometric)
- `item_id`: i32 - Encoded item/object identifier (format: `[item_id, item_type, 0, 0]`)

### Item ID Encoding
The `item_id` field uses a 32-bit encoded format:
- **Structure**: `[item_id, item_type, 0, 0]` (little-endian)
- **item_id**: u8 (byte 0) - Specific object ID (0-255)
- **item_type**: u8 (byte 1) - Object type/category ID
- **Bytes 2-3**: Always 0

### Item Types
| Value | Type | Description |
|-------|------|-------------|
| 1 | Weapon | Weapon objects |
| 2 | Healing | Healing/health items |
| 3 | Edit | Editable/interactive objects |
| 4 | Event | Event-triggering objects |
| 5 | Misc | Miscellaneous objects |
| 255 | Other | Unknown/other types |

## Field Definitions

### map_id
- Target map identifier
- References map files (e.g., `map1`, `cat1`, `dun01`)
- Links to `AllMap.ini` entries

### x_coord, y_coord
- Tile coordinates in isometric system
- Reference specific positions on the map
- Used for precise object placement


## Special Values
- Lines starting with `;` are comments
- Parenthesized CSV format required
- Coordinates use isometric tile system
- Empty or malformed lines are skipped

## File Purpose
Defines placement of interactive and decorative objects on specific maps with exact coordinates. Used for populating game worlds with:
- Environmental elements
- Quest objects
- Interactive items
- Decorative features
- Hidden secrets

## Related Files
- `AllMap.ini` - Map metadata and associations
- `Extra.ini` - Object definitions
- `*.map` files - Map geometry and tiles

## Object Types

### Containers
- Chests, barrels, crates
- Loot containers
- Storage objects

### Quest Objects
- Special items for quests
- Interactive mechanisms
- Puzzle elements

### Environmental
- Decorative objects
- Scenery elements
- Atmospheric features

### Interactive
- Doors and gates
- Levers and switches
- Traps and hazards

## Implementation
- **Rust Module**: `src/references/draw_item.rs`
- **Extractor**: `DrawItem` struct implementing `Extractor` trait
- **Data Structure**: `DrawItem` with separate `item_type` and `item_id` fields
- **Database**: Stores decoded form with separate `item_id` and `item_type` columns
- **File I/O**: Maintains encoded i32 format for game compatibility

## Example Usage

### Extract and display object placements:
```bash
cargo run -- extract -i "Dispel/Ref/DrawItem.ref"
```

### Format Example
```
; Map 1 object placements
(1,5,10,1001)    ; Basic chest: item_id=222, item_type=4 (Event)
(1,6,11,1002)    ; Basic chest: item_id=223, item_type=4 (Event)  
(1,10,15,2005)   ; Quest object: item_id=245, item_type=8 (Misc)
(2,3,8,3010)     ; Elite container: item_id=246, item_type=12 (Other)
```

### Decoding Example
The value `1001` decodes as:
- Hex: `0x000004DE`
- Bytes: `[0xDE, 0x04, 0x00, 0x00]`
- item_id: `0xDE` = 222
- item_type: `0x04` = Event

## Coordinate System
- Isometric tile-based coordinates
- Each tile is 32×32 pixels
- Origin typically at top-left of map
- Y-axis increases downward

## Game Mechanics
- Objects are placed at specific map locations
- Item IDs link to object definitions
- Coordinates determine interaction positions
- Used for world building and quest design

## Data Analysis
The file enables analysis of:
- Object distribution across maps
- Quest item placement patterns
- Hidden object locations
- World design balance
- Object density and variety

## Extractor

An extractor is available in `src/references/draw_item.rs` to parse this file format.

### How to Run

```bash
# Extract DrawItem.ref to JSON
cargo run -- extract -i "fixtures/Dispel/Ref/DrawItem.ref"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
