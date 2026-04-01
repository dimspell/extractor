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
- `item_id`: i32 - Encoded item/object identifier

## Field Definitions

### map_id
- Target map identifier
- References map files (e.g., `map1`, `cat1`, `dun01`)
- Links to `AllMap.ini` entries

### x_coord, y_coord
- Tile coordinates in isometric system
- Reference specific positions on the map
- Used for precise object placement

### item_id
- Encoded 32-bit integer combining multiple fields
- Structure: `[item_id, group_id, 0, 0]`
- Combines specific object ID and group/category ID

## Item ID Encoding

### Format
- 32-bit integer: `0x0000IIGG`
- `II`: Item ID (specific object)
- `GG`: Group ID (object category)

### Examples
- `1001` = Group 1, Item 1 (e.g., basic chest)
- `2005` = Group 20, Item 5 (e.g., special quest object)
- `3010` = Group 30, Item 10 (e.g., elite container)

### Common Groups
- 1-10: Basic containers and chests
- 11-20: Quest-related objects
- 21-30: Environmental decorations
- 31-40: Interactive mechanisms
- 41-50: Special/hidden objects

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
- **Data Structure**: `DrawItem` with map coordinates and item ID
- **Database**: Saved to SQLite via `save_draw_items` function

## Example Usage

### Extract and display object placements:
```bash
cargo run -- ref draw-item "Dispel/Ref/DrawItem.ref"
```

### Format Example
```
; Map 1 object placements
(1,5,10,1001)    ; Basic chest at position 5,10
(1,6,11,1002)    ; Basic chest at position 6,11
(1,10,15,2005)   ; Quest object at position 10,15
(2,3,8,3010)     ; Elite container at position 3,8
```

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
