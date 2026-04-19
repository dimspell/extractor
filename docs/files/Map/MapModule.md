# Map Module Specification

> This document references the mark solely for identification and compatibility purposes. No affiliation, endorsement, or sponsorship is implied.

---

## Overview

The `src/map/` module handles parsing, rendering, and database operations for the Dispel game's `.MAP`, `.GTL`, and `.BTL` binary file formats. It provides:

- **Binary parsing** of `.MAP` files (geometry, sprites, tiles, events, collisions)
- **Tileset extraction** from `.GTL` (ground) and `.BTL` (building) files
- **Isometric rendering** to PNG images
- **Database import/export** via SQLite
- **Sprite loading** from `.SPR` files for entity rendering

### Module Files

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 546 | Public API, top-level `.MAP` parser, CLI commands, DB import |
| `types.rs` | 39 | Coordinate types, constants, data structs |
| `model.rs` | 72 | Map geometry computation from chunk dimensions |
| `reader.rs` | 246 | Binary block readers for each `.MAP` section |
| `render.rs` | 460 | Isometric rendering pipeline (ground → objects → roofs) |
| `tileset.rs` | 418 | `.GTL`/`.BTL` tileset extraction and atlas generation |
| `database.rs` | 512 | Render from SQLite + external entity (NPC/monster/extra) rendering |
| `sprite_loader.rs` | 137 | `.SPR` file frame loading and sprite plotting |

---

## File Formats

### `.MAP` — Map File

Binary file containing the complete map definition: geometry, embedded sprites, tiled objects, event triggers, ground tiles, collision data, and optional roof tiles.

#### Structure

```
+-----------------------------------+
| HEADER (8 bytes)                  |
|   tiled_map_chunk_width:  i32     |  ← number of 25×25 chunks on X axis
|   tiled_map_chunk_height: i32     |  ← number of 25×25 chunks on Y axis
+-----------------------------------+
| FIRST BLOCK (unknown, skipped)    |
|   multiplier: i32                 |
|   size:       i32                 |
|   data:       multiplier*size*4   |  ← skip this many bytes
+-----------------------------------+
| SECOND BLOCK (unknown, skipped)   |
|   size: i32                       |
|   data: size*2                    |  ← skip this many bytes
+-----------------------------------+
| SPRITE BLOCK                      |
|   sprite_count: i32               |
|   For each sprite:                |
|     image_stamp:    i32           |  ← 6 or 9 (determines data layout)
|     metadata:       264 bytes     |  ← unknown purpose
|     sequence_info:  variable      |  ← frame count, positions, pixel data
|     padding:        1904 or 2996  |  ← depends on image_stamp
+-----------------------------------+
| SPRITE INFO BLOCK                 |
|   placement_count: i32            |
|   For each placement:             |
|     sprite_id:          i32       |  ← index into sprite block
|     unknown:            i32 × 2   |
|     bottom_right_x:     i32       |
|     bottom_right_y:     i32       |
|     sprite_x:           i32       |  ← pixel X position on map
|     sprite_y:           i32       |  ← pixel Y position on map
|     frame_skip_data:    variable  |  ← (frame_count-1) × 6 × 4 bytes
+-----------------------------------+
| TILED OBJECTS BLOCK               |
|   bundles_count: i32              |
|   number1:       i32              |
|   For each bundle:                |
|     metadata:  264 bytes          |
|     s8, s0_1, s1, s0_2: i32 × 4  |
|     v1..v4:       i32 × 4         |
|     x:            i32             |  ← tile X coordinate
|     y:            i32             |  ← tile Y coordinate
|     v7, v8:       i32 × 2         |
|     c1, c2, c3:   i32 × 3         |  ← counts for various data sections
|     tile_ids:     i16 × c3        |  ← BTL tile IDs stacked vertically
|     padding:      84 bytes        |
|     extra_data:   (c1+c2+c3) × 4  |
|   Sentinel alignment (20 bytes scan for 0x01 marker)
+-----------------------------------+
| EVENT BLOCK (read from end)       |
|   For each tile (width × height): |
|     event_id:      i16            |  ← event trigger ID
|     unknown_value: i16            |
+-----------------------------------+
| TILE & ACCESS BLOCK               |
|   For each tile (width × height): |
|     packed_value: i32             |  ← bits 10..31 = GTL tile ID
|                                   |  ← bit 0 = collision flag
+-----------------------------------+
| ROOF TILE BLOCK (optional)        |
|   For each tile (width × height): |
|     btl_tile_id: i16              |  ← BTL tile ID (0 = none)
|     some_flag:   i16              |  ← unknown flag
+-----------------------------------+
```

#### Computed Dimensions

From the chunk header (`chunk_width`, `chunk_height`):

```
MAP_CHUNK_SIZE = 25
tiled_map_width  = chunk_width  × 25 - 1
tiled_map_height = chunk_height × 25 - 1
diagonal         = chunk_width + chunk_height

map_width_in_pixels  = diagonal × 25 × 32
map_height_in_pixels = diagonal × 25 × 16

map_non_occluded_start_x = round(0.3 × map_width_in_pixels  - 32)
map_non_occluded_start_y = round(0.2 × map_height_in_pixels -  0)

occluded_map_in_pixels_width  = map_width_in_pixels  - 2 × map_non_occluded_start_x
occluded_map_in_pixels_height = map_height_in_pixels - 2 × map_non_occluded_start_y
```

#### Coordinate System

- **Chunk-based**: 1 chunk = 25×25 tiles
- **Tile size**: 32×32 pixels
- **Isometric projection**: diamond-shaped tiles
- **Constants**:
  - `TILE_HORIZONTAL_OFFSET_HALF = 32`
  - `TILE_HEIGHT_HALF = 16`
  - `TILE_WIDTH_HALF = 31`
  - `TILE_PIXEL_NUMBER = 1024`

**Tile → Pixel conversion:**
```
start_x = (x + y) × 32
start_y = (-x + y) × 16 + (diagonal / 2 × 16)
```

#### Event Block Location

The event block and tile blocks are located at the **end of the file**. The parser seeks backwards from EOF:
```
seek_offset = -(tiled_map_width × tiled_map_height × 4 × 3)
```
This accounts for 3 blocks of `width × height` records: events (4 bytes/tile), tiles (4 bytes/tile), and roof tiles (4 bytes/tile).

---

### `.GTL` / `.BTL` — Tileset Files

Simple binary format with **no header**. Direct sequence of 32×32 pixel tiles in RGB565.

#### Structure

```
+---------------------------+
| TILE #0                   |
|   pixels: u16 × 1024      |  ← RGB565, little-endian
+---------------------------+
| TILE #1                   |
|   pixels: u16 × 1024      |
+---------------------------+
| ...                       |
+---------------------------+
| TILE #N                   |
|   pixels: u16 × 1024      |
+---------------------------+
```

#### Properties

- **No header or metadata**
- **Tile size**: 32×32 pixels = 1024 pixels × 2 bytes = 2048 bytes per tile
- **Tile count**: `file_size / 2048`
- **Color format**: RGB565 (5-bit red, 6-bit green, 5-bit blue)
- **Rendered size**: 62×32 pixels (isometric diamond)
- **Transparency**: RGB(0,0,0) = transparent

**RGB565 → RGB888 conversion:**
```
red   = (pixel >> 11) & 0x1F  → scale 0-31 to 0-255
green = (pixel >>  5) & 0x3F  → scale 0-63 to 0-255
blue  =  pixel        & 0x1F  → scale 0-31 to 0-255
```

**File types:**
- `.GTL` — Ground Tile Layer (terrain, paths, natural features)
- `.BTL` — Building Tile Layer (structures, roofs, man-made objects)

---

### `.SPR` — Sprite Files (referenced, not owned by map module)

Sprite files are parsed by the parent `sprite` module but loaded by `sprite_loader.rs` for entity rendering.

#### Structure (per sequence)

```
+-----------------------------------+
| image_stamp:  i32                 |  ← 6 or 9
| metadata:     264 bytes           |
| frame_count:  i32                 |
| For each frame:                   |
|   width:              i32         |
|   height:             i32         |
|   origin_x:           i32         |  ← anchor offset X
|   origin_y:           i32         |  ← anchor offset Y
|   image_start_pos:    i64         |  ← absolute file position
|   pixels:             u16 × (w×h) |  ← RGB565
| sequence_end_position: i64        |  ← absolute file position (next sequence)
+-----------------------------------+
```

---

## Data Types

### Core Types (`types.rs`)

```rust
type Coords = (i32, i32);  // isometric (x, y) tile coordinate

struct EventBlock {
    x: i32,             // tile X
    y: i32,             // tile Y
    event_id: i16,      // event trigger ID
    _unknown_value: i16,
}

struct SpriteInfoBlock {
    sprite_id: usize,   // index into internal_sprites
    sprite_x: i32,      // pixel X position
    sprite_y: i32,      // pixel Y position
}

struct TiledObjectInfo {
    ids: Vec<i16>,      // stacked BTL tile IDs (bottom to top)
    x: i32,             // tile X coordinate
    y: i32,             // tile Y coordinate
}
```

### MapModel (`model.rs`)

```rust
struct MapModel {
    tiled_map_width: i32,              // tiles on X axis
    tiled_map_height: i32,             // tiles on Y axis
    map_width_in_pixels: i32,          // full image width
    map_height_in_pixels: i32,         // full image height
    map_non_occluded_start_x: i32,     // visible viewport X offset
    map_non_occluded_start_y: i32,     // visible viewport Y offset
    occluded_map_in_pixels_width: i32, // cropped image width
    occluded_map_in_pixels_height: i32,// cropped image height
}
```

### MapData (`mod.rs`)

```rust
struct MapData {
    model: MapModel,                                    // computed geometry
    gtl_tiles: HashMap<Coords, i32>,                    // ground tile ID per coordinate
    btl_tiles: HashMap<Coords, i32>,                    // roof tile ID per coordinate
    collisions: HashMap<Coords, bool>,                  // collision flag per coordinate
    events: HashMap<Coords, EventBlock>,                // event trigger per coordinate
    tiled_infos: Vec<TiledObjectInfo>,                  // building/object definitions
    internal_sprites: Vec<SequenceInfo>,                // embedded sprite sequences
    sprite_blocks: Vec<SpriteInfoBlock>,                // sprite placements
}
```

### Tile (`tileset.rs`)

```rust
struct Tile {
    colors: [Color; 1024],  // 32×32 pixel color data
}

struct Color {
    r: u8,
    g: u8,
    b: u8,
}
```

### LoadedSpriteFrame (`sprite_loader.rs`)

```rust
struct LoadedSpriteFrame {
    image: RgbaImage,   // decoded frame pixels
    origin_x: i32,      // anchor offset X
    origin_y: i32,      // anchor offset Y
}
```

---

## Public API

### Parsing

| Function | Input | Output | Description |
|----------|-------|--------|-------------|
| `read_map_data(reader)` | `BufReader<File>` | `MapData` | Parse complete `.MAP` file |
| `read_map_model(reader)` | `BufReader<File>` | `MapModel` | Parse header + compute geometry |
| `tileset::extract(path)` | `&Path` (.gtl/.btl) | `Vec<Tile>` | Extract all tiles from tileset |
| `sprite_loader::load_sprite_frames(path)` | `&Path` (.spr) | `Option<Vec<LoadedSpriteFrame>>` | Load first frame of each sequence |

### Rendering

| Function | Description |
|----------|-------------|
| `extract(map, btl, gtl, output, save_sprites)` | Render `.MAP` + tilesets to PNG |
| `extract_sprites(map, output_dir)` | Extract embedded sprites to PNGs |
| `render_from_database(db, map_id, gtl_atlas, btl_atlas, columns, output, game_path)` | Render map from SQLite + atlas PNGs |

### Database

| Function | Description |
|----------|-------------|
| `import_to_database(db_path, map_path)` | Parse `.MAP` and save to SQLite |
| `save_to_db(conn, map_id, data)` | Write `MapData` to database tables |
| `save_map_tiles(params)` | Save tile/collision/event records |
| `save_map_objects(conn, map_id, tiled_infos)` | Save building object records |
| `save_map_sprites(conn, map_id, sprite_blocks)` | Save sprite placement records |
| `save_map_metadata(conn, map_id, model)` | Save map dimension metadata |

### Utilities

| Function | Description |
|----------|-------------|
| `convert_map_coords_to_image_coords(x, y, diagonal)` | Convert tile coords to pixel coords |
| `plot_atlas_tile(params)` | Blit tile from atlas with alpha blending |
| `plot_entity_sprite(dest, sprite, x, y, flip)` | Plot sprite frame with optional horizontal flip |

---

## Rendering Pipeline

### Pass Order (from `render.rs`)

```
1. plot_base()     — Ground tiles (GTL) with event/collision coloring
2. plot_objects()  — Sprites + tiled objects, sorted by ground_y for proper depth
3. plot_roofs()    — Roof/building tiles (BTL)
```

### Pass Order (from `database.rs` — render_from_database)

```
1. Ground tiles    — GTL tiles from atlas PNG
2. Objects         — Stacked BTL tiles from atlas, sorted by ground_y
3. Roofs           — BTL roof tiles from atlas
4. External entities — NPCs (green), monsters (red), extras (blue)
                     — Real sprites if game_path provided, else colored markers
```

### Depth Sorting

Sprites and tiled objects are sorted by `ground_y` before rendering:
- **Sprites**: `ground_y = sprite_y + frame_height`
- **Tiled objects**: `ground_y = tile_y + (stack_height × 32)`

This ensures proper isometric depth ordering (painter's algorithm).

---

## Database Schema

### Tables Created by Map Import

| Table | Columns | Description |
|-------|---------|-------------|
| `map_metadata` | `map_id`, `tiled_width`, `tiled_height`, `pixel_width`, `pixel_height`, `non_occluded_x`, `non_occluded_y`, `occluded_width`, `occluded_height` | Map dimensions and offsets |
| `map_tiles` | `map_id`, `x`, `y`, `gtl_tile_id`, `btl_tile_id`, `collision`, `event_id` | Per-tile ground/roof/collision/event data |
| `map_objects` | `map_id`, `object_index`, `x`, `y`, `btl_tile_id`, `stack_order` | Building tile stacks |
| `map_sprites` | `map_id`, `sprite_index`, `x`, `y`, `sprite_id` | Embedded sprite placements |

### Tables Used by External Entity Rendering

| Table | Source | Description |
|-------|--------|-------------|
| `map_inis` | `references/map_ini.rs` | Map config with ref filenames |
| `maps` | `database.rs` | Map file → map_id mapping |
| `monster_refs` | `references/monster_ref.rs` | Monster placements |
| `monster_inis` | `references/monster_ini.rs` | Monster visual config (sprite filename) |
| `npc_refs` | `references/npc_ref.rs` | NPC placements |
| `npc_inis` | `references/npc_ini.rs` | NPC visual config (sprite filename) |
| `extra_refs` | `references/extra_ref.rs` | Extra object placements |
| `extras` | `references/extra_ini.rs` | Extra object config (sprite filename) |

---

## JSON Schema for Map Data Export

The following JSON schema describes the structure for exporting `.MAP` file data:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DispelMapData",
  "description": "Complete parsed representation of a Dispel game .MAP file",
  "type": "object",
  "required": ["metadata", "gtl_tiles", "btl_tiles", "collisions", "events", "tiled_objects", "sprites"],
  "properties": {
    "metadata": {
      "type": "object",
      "required": ["chunk_width", "chunk_height", "tiled_width", "tiled_height"],
      "properties": {
        "chunk_width": { "type": "integer", "description": "Number of 25-tile chunks on X axis" },
        "chunk_height": { "type": "integer", "description": "Number of 25-tile chunks on Y axis" },
        "tiled_width": { "type": "integer", "description": "Total tile count on X axis (chunk_width * 25 - 1)" },
        "tiled_height": { "type": "integer", "description": "Total tile count on Y axis (chunk_height * 25 - 1)" },
        "map_width_in_pixels": { "type": "integer", "description": "Full rendered image width in pixels" },
        "map_height_in_pixels": { "type": "integer", "description": "Full rendered image height in pixels" },
        "non_occluded_start_x": { "type": "integer", "description": "Visible viewport X offset" },
        "non_occluded_start_y": { "type": "integer", "description": "Visible viewport Y offset" },
        "occluded_width": { "type": "integer", "description": "Cropped image width" },
        "occluded_height": { "type": "integer", "description": "Cropped image height" }
      }
    },
    "gtl_tiles": {
      "type": "array",
      "description": "Ground tile assignments per coordinate",
      "items": {
        "type": "object",
        "required": ["x", "y", "tile_id"],
        "properties": {
          "x": { "type": "integer", "minimum": 0 },
          "y": { "type": "integer", "minimum": 0 },
          "tile_id": { "type": "integer", "description": "Index into the .GTL tileset" }
        }
      }
    },
    "btl_tiles": {
      "type": "array",
      "description": "Roof/building tile assignments per coordinate",
      "items": {
        "type": "object",
        "required": ["x", "y", "tile_id"],
        "properties": {
          "x": { "type": "integer", "minimum": 0 },
          "y": { "type": "integer", "minimum": 0 },
          "tile_id": { "type": "integer", "description": "Index into the .BTL tileset" }
        }
      }
    },
    "collisions": {
      "type": "array",
      "description": "Collision flags per coordinate",
      "items": {
        "type": "object",
        "required": ["x", "y", "blocked"],
        "properties": {
          "x": { "type": "integer", "minimum": 0 },
          "y": { "type": "integer", "minimum": 0 },
          "blocked": { "type": "boolean", "description": "Whether this tile blocks movement" }
        }
      }
    },
    "events": {
      "type": "array",
      "description": "Event triggers per coordinate",
      "items": {
        "type": "object",
        "required": ["x", "y", "event_id"],
        "properties": {
          "x": { "type": "integer", "minimum": 0 },
          "y": { "type": "integer", "minimum": 0 },
          "event_id": { "type": "integer", "description": "Event trigger ID (0 = no event)" }
        }
      }
    },
    "tiled_objects": {
      "type": "array",
      "description": "Buildings and objects made of stacked BTL tiles",
      "items": {
        "type": "object",
        "required": ["index", "x", "y", "tile_ids"],
        "properties": {
          "index": { "type": "integer", "description": "Object index (0-based)" },
          "x": { "type": "integer", "description": "Tile X coordinate" },
          "y": { "type": "integer", "description": "Tile Y coordinate" },
          "tile_ids": {
            "type": "array",
            "items": { "type": "integer" },
            "description": "BTL tile IDs stacked bottom-to-top"
          }
        }
      }
    },
    "sprites": {
      "type": "array",
      "description": "Embedded sprite placements",
      "items": {
        "type": "object",
        "required": ["index", "sprite_id", "x", "y"],
        "properties": {
          "index": { "type": "integer", "description": "Placement index (0-based)" },
          "sprite_id": { "type": "integer", "description": "Index into internal_sprites array" },
          "x": { "type": "integer", "description": "Pixel X position on map" },
          "y": { "type": "integer", "description": "Pixel Y position on map" }
        }
      }
    },
    "internal_sprites": {
      "type": "array",
      "description": "Embedded sprite sequence definitions",
      "items": {
        "type": "object",
        "required": ["index", "image_stamp", "frame_count", "frames"],
        "properties": {
          "index": { "type": "integer", "description": "Sprite index (0-based)" },
          "image_stamp": { "type": "integer", "enum": [6, 9], "description": "Data layout variant" },
          "frame_count": { "type": "integer", "description": "Number of animation frames" },
          "frames": {
            "type": "array",
            "items": {
              "type": "object",
              "required": ["width", "height", "origin_x", "origin_y"],
              "properties": {
                "width": { "type": "integer", "minimum": 0 },
                "height": { "type": "integer", "minimum": 0 },
                "origin_x": { "type": "integer", "description": "Anchor offset X" },
                "origin_y": { "type": "integer", "description": "Anchor offset Y" }
              }
            }
          }
        }
      }
    }
  }
}
```

---

## Tileset JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DispelTileset",
  "description": "Extracted tileset from .GTL or .BTL file",
  "type": "object",
  "required": ["tile_count", "tile_width", "tile_height", "color_format", "tiles"],
  "properties": {
    "tile_count": { "type": "integer", "description": "Total number of tiles" },
    "tile_width": { "type": "integer", "const": 32, "description": "Tile pixel width" },
    "tile_height": { "type": "integer", "const": 32, "description": "Tile pixel height" },
    "rendered_width": { "type": "integer", "const": 62, "description": "Isometric diamond width" },
    "rendered_height": { "type": "integer", "const": 32, "description": "Isometric diamond height" },
    "color_format": { "type": "string", "const": "RGB565", "description": "Source color encoding" },
    "file_type": { "type": "string", "enum": ["gtl", "btl"], "description": "Ground or building tile layer" },
    "tiles": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["index", "pixels"],
        "properties": {
          "index": { "type": "integer", "description": "Tile index (0-based, used as tile_id in map)" },
          "pixels": {
            "type": "array",
            "items": {
              "type": "object",
              "required": ["x", "y", "r", "g", "b"],
              "properties": {
                "x": { "type": "integer", "minimum": 0, "maximum": 31 },
                "y": { "type": "integer", "minimum": 0, "maximum": 31 },
                "r": { "type": "integer", "minimum": 0, "maximum": 255 },
                "g": { "type": "integer", "minimum": 0, "maximum": 255 },
                "b": { "type": "integer", "minimum": 0, "maximum": 255 }
              }
            },
            "minItems": 1024,
            "maxItems": 1024
          }
        }
      }
    }
  }
}
```

---

## Quick Reference: Byte Offsets in `.MAP` File

| Offset | Size | Field | Notes |
|--------|------|-------|-------|
| 0x00 | 4 | `chunk_width` | i32 LE |
| 0x04 | 4 | `chunk_height` | i32 LE |
| 0x08 | 4 | `first_block_multiplier` | i32 LE |
| 0x0C | 4 | `first_block_size` | i32 LE |
| 0x10 | var | `first_block_data` | Skip `multiplier * size * 4` bytes |
| var | 4 | `second_block_size` | i32 LE |
| var | var | `second_block_data` | Skip `size * 2` bytes |
| var | 4 | `sprite_count` | i32 LE |
| var | var | `sprite entries` | Each: stamp(4) + meta(264) + sequence_info + padding |
| var | 4 | `sprite_placement_count` | i32 LE |
| var | var | `sprite placements` | Each: 7×i32 + variable frame skip |
| var | 4 | `tiled_object_count` | i32 LE |
| var | 4 | `tiled_object_number1` | i32 LE |
| var | var | `tiled objects` | Each: 264 + 8×i32 + 3×i32 + c3×i16 + 84 + (c1+c2+c3)×4 |
| EOF-(w×h×12) | w×h×4 | `event blocks` | Each: event_id(i16) + unknown(i16) |
| EOF-(w×h×8) | w×h×4 | `tile & access` | Each: packed_value(i32) |
| EOF-(w×h×4) | w×h×4 | `roof tiles` | Each: btl_id(i16) + flag(i16) |

Where `w = tiled_map_width`, `h = tiled_map_height`.

---

## External Entity Rendering Colors

When `game_path` is not provided or sprite files are missing, entities are rendered as colored diamond markers:

| Entity Type | Color (RGBA) | Sprite Directory |
|-------------|-------------|------------------|
| Monsters | `rgba(255, 60, 60, 255)` — red | `MonsterInGame/` |
| NPCs | `rgba(60, 255, 60, 255)` — green | `NpcInGame/` |
| Extras | `rgba(80, 120, 255, 255)` — blue | `ExtraInGame/` |

---

## Notes

1. **Two unknown blocks** at the start of `.MAP` files are skipped — their purpose is not yet understood
2. **Image stamps** (6 or 9) determine the padding size after sprite sequence data (1904 or 2996 bytes respectively)
3. **Tiled object sentinel** — the end of the tiled objects block is detected by scanning 20 bytes backwards for a `0x01` marker
4. **Roof block is optional** — only parsed if remaining file size is sufficient
5. **Collision flag** is extracted from bit 0 of the packed tile value; tile ID is bits 10–31
6. **All integers are little-endian**
7. **Sprite sequences** in the map file are self-contained — pixel data is embedded directly, not referenced externally
