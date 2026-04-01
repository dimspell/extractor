# cat1.map - Dispel Game Map File Format

## File Information
- **Location**: `Map/*.map` files
- **Format**: Binary (Little-Endian)
- **Coordinate System**: Isometric with 25×25 tile chunks
- **Tile Size**: 32×32 pixels

## File Structure

### Header (8 bytes)
- `width_in_chunks`: i32 - Map width in 25-tile chunks
- `height_in_chunks`: i32 - Map height in 25-tile chunks

### Blocks (in order)

#### First Block (variable size)
- `multiplier`: i32
- `size`: i32
- `data`: multiplier × size × 4 bytes (unknown purpose, skipped)

#### Second Block (variable size)
- `size`: i32
- `data`: size × 2 bytes (unknown purpose, skipped)

#### Sprite Block
- `sprite_count`: i32
- For each sprite:
  - `image_stamp`: i32
  - `metadata`: 264 bytes
  - `sequence_info`: variable
  - `pixel_data`: variable

#### Sprite Info Block
- `placement_count`: i32
- For each placement:
  - `sprite_id`: i32
  - `position_data`: variable
  - `frame_count`: i32

#### Tiled Objects Block
- `bundle_count`: i32
- For each bundle:
  - `metadata`: 264 bytes
  - `coordinates`: (x,y) i32 each
  - `tile_stack_ids`: variable
  - `building_definition`: variable

#### Event Block (near end of file)
- For each tile (width × height):
  - `event_id`: i16
  - `unknown`: i16

#### Tile & Access Block
- For each tile (width × height):
  - `gtl_tile_id`: i32
  - `collision_flag`: i32

#### Roof Tile Block (optional)
- For each tile (width × height):
  - `btl_tile_id`: i16
  - `flags`: i16

## Coordinate System
- **Chunk-based**: 1 chunk = 25×25 tiles
- **Isometric coordinates**: (x,y) tile positions
- **Tile size**: 32×32 pixels
- **Offsets**:
  - `TILE_HORIZONTAL_OFFSET_HALF` = 32
  - `TILE_HEIGHT_HALF` = 16
  - `TILE_WIDTH_HALF` = 16

## File Size Calculation
```
Total size = header + blocks + (width × height × (2+4+2)) + optional roof data
```

## Related Files
- `*.gtl` - Ground tileset files
- `*.btl` - Building/roof tileset files
- `AllMap.ini` - Map metadata and associations

## Map Files
- **Main maps**: `map1.map`, `map2.map`, `map3.map`
- **Catacombs**: `cat1.map`, `cat2.map`, `cat3.map`, `catp.map`
- **Dungeons**: `dun01.map` through `dun25.map`, `final.map`

## Implementation
- **Rust Module**: `src/map/mod.rs`
- **Parser**: `read_map_data` function
- **Renderer**: `render_map` function
- **Database**: `import_map_to_database` function

## Example Usage

### Render a map to PNG:
```bash
cargo run -- map render \
  --map "fixtures/Dispel/Map/cat1.map" \
  --btl "fixtures/Dispel/Map/cat1.btl" \
  --gtl "fixtures/Dispel/Map/cat1.gtl" \
  --output cat1.png
```

### Extract sprites from a map:
```bash
cargo run -- map sprites "fixtures/Dispel/Map/cat1.map"
```

### Import to database:
```bash
cargo run -- map import "fixtures/Dispel/Map/cat1.map"
```

## Coordinate Conversion
The `convert_map_coords_to_image_coords` function handles the isometric coordinate system conversion for rendering.

## Sprite Handling
Sprites are stored as sequences with metadata including:
- Frame information
- Animation timing
- Pixel data
- Placement coordinates

## Event System
Each tile can have an associated event ID that triggers in-game events when the player interacts with that location.
