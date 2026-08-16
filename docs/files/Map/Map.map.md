# cat1.map - Dispel Game Map File Format

## File Information
- **Location**: `Map/*.map` files
- **Format**: Binary (Little-Endian)
- **Coordinate System**: Isometric
- **Tile Size**: 62×32 pixels (diamond)

## File Structure

### Header (12 bytes)
- `width_in_chunks`: i32 - Map width in 25-tile chunks
- `height_in_chunks`: i32 - Map height in 25-tile chunks
- `border_count`: i32 - Border chunk count (always 2)

### Blocks (in order)

#### First Block (variable size)
- `count`: i32 - Record count
- `data`: (count − 1) × 8 bytes - `count − 1` records, each 2 × i32: `value1`, `value2`

Skipping `(count-1)*8` lands exactly on the second block's size field.

**Decoded layout (from `FUN_00423a99` in Dispel.psudo.c):** the game reads `count − 1`
records of 8 bytes each into a 20-byte record array (map object offset `+0x14c`):

| offset | value | source |
|---|---|---|
| `+0x00` | `index = i + 1` | computed (linked-list-style index) |
| `+0x04` | `0` | computed |
| `+0x08` | `value1` | read from file (4 bytes) |
| `+0x0c` | `0` | computed |
| `+0x10` | `value2` | read from file (4 bytes) |

Record 0 is pre-initialized to `{0, 0, 0, 0, 1}` by constructor `FUN_0042244f`.

**Empirical observation (all 32 fixtures):** every record is `(0, 1)` — constant,
zero per-map information.

**No consumers found:** only ctor/dtor pairs (`FUN_0042244f/7a`,
`FUN_004224a2/ac`) allocate/initialize/free the array; no gameplay logic reads it.
Vestigial linked-list/link-table structure that is never dereferenced for map data.
Skipping is byte-exact correct and loses nothing meaningful.

#### Second Block (variable size)
- `size`: i32
- `data`: size × 2 bytes - byte pairs, read by the game into map object offset `+0x1ac`

The game also derives an identity index array `[0, 1, …, size − 1]` (map object
offset `+0x154`, allocated `size × 4 + 4` bytes) from `size` alone — it is not read
from the file.

**Empirical observation (all 32 fixtures):** bytes are only ever `{0, 1}` (starts
`00 01 01 01 …`) — constant, zero per-map information. No gameplay consumer found;
same vestigial status as the first block. Skipped.

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

## Extractor

An extractor is available in `src/map/mod.rs` to parse this file format.

### How to Run

```bash
# Extract map sprites
cargo run -- map sprites "fixtures/Dispel/Map/cat1.map"

# Render map to PNG
cargo run -- map render \
  --map "fixtures/Dispel/Map/cat1.map" \
  --btl "fixtures/Dispel/Map/cat1.btl" \
  --gtl "fixtures/Dispel/Map/cat1.gtl" \
  --output cat1.png

# Import map to database
cargo run -- map to-db \
  --database "database.sqlite" \
  --map "fixtures/Dispel/Map/cat1.map"
```
