# Map Module Specification

> DISPEL® is a registered trademark. This project is not affiliated with,
> endorsed by, or sponsored by the trademark owner.

---

## Overview

The `src/map/` module handles parsing, rendering, and database operations for the Dispel game's `.MAP`, `.GTL`, and
`.BTL` binary file formats. It provides:

- **Binary parsing** of `.MAP` files (geometry, sprites, tiles, events, collisions)
- **Tileset extraction** from `.GTL` (ground) and `.BTL` (building) files
- **Isometric rendering** to PNG images
- **Database import/export** via SQLite
- **Sprite loading** from `.SPR` files for entity rendering

### Module Files

| File               | Purpose                                                            |
|--------------------|--------------------------------------------------------------------|
| `mod.rs`           | Public API, top-level `.MAP` parser, CLI commands, DB import       |
| `types.rs`         | Coordinate types, constants, data structs                          |
| `model.rs`         | Map geometry computation from chunk dimensions                     |
| `reader.rs`        | Binary block readers for each `.MAP` section                       |
| `writer.rs`        | `.MAP` binary writer (round-trip serialization)                    |
| `tmx.rs`           | TMX (Tiled) map export                                             |
| `render.rs`        | Isometric rendering pipeline (ground → objects → roofs)            |
| `tileset.rs`       | `.GTL`/`.BTL` tileset extraction and atlas generation              |
| `database.rs`      | Render from SQLite + external entity (NPC/monster/extra) rendering |
| `sprite_loader.rs` | `.SPR` file frame loading and sprite plotting                      |

---

## File Formats

### `.MAP` — Map File

Binary file containing the complete map definition: geometry, embedded sprites, tiled objects, event triggers, ground
tiles, collision data, and optional roof tiles.

#### Structure

```
+-----------------------------------------------+
| HEADER (12 bytes)                             |
|   chunk_width:   i32  ← number of 25×25 chunks on X |
|   chunk_height:  i32  ← number of 25×25 chunks on Y |
|   border_count:  i32  ← always 2              |
+-----------------------------------------------+
| OBJECT-REF RECORDS (skipped by the parser)    |
|   count: i32                                  |
|   records: (count-1) × {value1, value2} i32   |
|     value2 = linear tile index into the three |
|     end grids (y × stride + x)                |
+-----------------------------------------------+
| OVERLAY-ID TABLE (skipped by the parser)      |
|   size: i32                                   |
|   entries: size × u16                         |
|     {low byte: transparency mode,             |
|      high byte: draw-enable}                  |
|     indexed by the Access-Ref grid's refs     |
+-----------------------------------------------+
| SPRITE BLOCK                                  |
|   sprite_count: i32                           |
|   For each sprite:                            |
|     image_stamp:    i32     ← 6 or 9          |
|     metadata:       264 bytes                 |
|     sequence_info:  variable                  |
|     pixel_data:     1904 or 2996 bytes        |
+-----------------------------------------------+
| SPRITE INFO BLOCK                             |
|   placement_count: i32                        |
|   For each placement:                         |
|     sprite_id:        i32                     |
|     frame-0 record (24 bytes):                |
|       bbox_left, bbox_top,                    |
|       bbox_right, bbox_bottom  i32 × 4        |
|       ← bounding box in map-local pixels;     |
|         right-left = frame width,             |
|         bottom-top = frame height;            |
|         bottom is the Y-sort key              |
|     anchor_x, anchor_y         i32 × 2        |
|       ← duplicate left/top in shipped maps    |
|     remaining frames: (frame_count-1) × 24 B  |
+-----------------------------------------------+
| TILED OBJECTS BLOCK (buildings)               |
|   bundles_count:      i32                     |
|   sub_record_count:   i32                     |
|   For each bundle:                            |
|     metadata:            264 bytes            |
|     control_0..3:        i32 × 4  ← (8,0,1,0) |
|     param_0..3:          i32 × 4  ← unmapped  |
|     anchor_x, anchor_y:  i32     ← map px     |
|     param_4..5:          i32 × 2  ← unmapped  |
|     extra_count_a/b:     i32 × 2              |
|     tile_stack_len:      i32                  |
|     tile_ids:            i16 × tile_stack_len |
|       ← BTL tiles stacked top → bottom        |
|     trailing: 84 bytes + (counts sum) × 4     |
|   Sentinel alignment (20-byte scan for byte 1)|
+-----------------------------------------------+
| EVENT GRID (end of file)                      |
|   For each tile: packed u32                   |
|     bits 0-13  event id (ids < 70 valid)      |
|     bit  22    tile marked / entity occupies  |
|     remainder  unmapped                       |
+-----------------------------------------------+
| TILE & ACCESS GRID                            |
|   For each tile: packed u32                   |
|     bit 0      collision                      |
|     bits 1-9   object slot id                 |
|     bits 10-24 GTL ground-tile index          |
+-----------------------------------------------+
| ACCESS-REF GRID ("roof")                      |
|   For each tile: packed u32                   |
|     bits 0-14  overlay ref → overlay table    |
|     bits 15-29 shadow level (0-199)           |
|     bits 30-31 light-source flags             |
+-----------------------------------------------+
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

This accounts for 3 blocks of `width × height` records: events (4 bytes/tile), tiles (4 bytes/tile), and roof tiles (4
bytes/tile).

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
- **Transparency**: RGB (0,0,0) = transparent

**RGB565 → RGB888 conversion:**

```
red   = bits 11–15 of pixel → scale 0-31 to 0-255
green = bits 5–10 of pixel  → scale 0-63 to 0-255
blue  = bits 0–4 of pixel   → scale 0-31 to 0-255
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

| Function                                  | Input               | Output                           | Description                       |
|-------------------------------------------|---------------------|----------------------------------|-----------------------------------|
| `read_map_data(reader)`                   | `BufReader<File>`   | `MapData`                        | Parse complete `.MAP` file        |
| `read_map_model(reader)`                  | `BufReader<File>`   | `MapModel`                       | Parse header + compute geometry   |
| `tileset::extract(path)`                  | `&Path` (.gtl/.btl) | `Vec<Tile>`                      | Extract all tiles from tileset    |
| `sprite_loader::load_sprite_frames(path)` | `&Path` (.spr)      | `Option<Vec<LoadedSpriteFrame>>` | Load first frame of each sequence |

### Rendering

| Function                                                                             | Description                         |
|--------------------------------------------------------------------------------------|-------------------------------------|
| `extract(map, btl, gtl, output, save_sprites)`                                       | Render `.MAP` + tilesets to PNG     |
| `extract_sprites(map, output_dir)`                                                   | Extract embedded sprites to PNGs    |

### Database

| Function                                        | Description                        |
|-------------------------------------------------|------------------------------------|
| `import_to_database(db_path, map_path)`         | Parse `.MAP` and save to SQLite    |
| `save_to_db(conn, map_id, data)`                | Write `MapData` to database tables |
| `save_map_tiles(params)`                        | Save tile/collision/event records  |
| `save_map_objects(conn, map_id, tiled_infos)`   | Save building object records       |
| `save_map_sprites(conn, map_id, sprite_blocks)` | Save sprite placement records      |
| `save_map_metadata(conn, map_id, model)`        | Save map dimension metadata        |

### Utilities

| Function                                             | Description                                     |
|------------------------------------------------------|-------------------------------------------------|
| `convert_map_coords_to_image_coords(x, y, diagonal)` | Convert tile coords to pixel coords             |
| `plot_entity_sprite(dest, sprite, x, y, flip)`       | Plot sprite frame with optional horizontal flip |

---

## Rendering Pipeline

### Pass Order (from `render.rs`)

```
1. plot_base()     — Ground tiles (GTL) with event/collision coloring
2. plot_objects()  — Sprites + tiled objects, sorted by ground_y for proper depth
3. plot_roofs()    — Roof/building tiles (BTL)
```

### Depth Sorting

Sprites and tiled objects are sorted by `ground_y` before rendering:

- **Sprites**: `ground_y = sprite_y + frame_height`
- **Tiled objects**: `ground_y = tile_y + (stack_height × 32)`

This ensures proper isometric depth ordering (painter's algorithm).

---

## Database Schema

### Tables Created by Map Import

| Table          | Columns                                                                                                                                         | Description                               |
|----------------|-------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------|
| `map_metadata` | `map_id`, `tiled_width`, `tiled_height`, `pixel_width`, `pixel_height`, `non_occluded_x`, `non_occluded_y`, `occluded_width`, `occluded_height` | Map dimensions and offsets                |
| `map_tiles`    | `map_id`, `x`, `y`, `gtl_tile_id`, `btl_tile_id`, `collision`, `event_id`                                                                       | Per-tile ground/roof/collision/event data |
| `map_objects`  | `map_id`, `object_index`, `x`, `y`, `btl_tile_id`, `stack_order`                                                                                | Building tile stacks                      |
| `map_sprites`  | `map_id`, `sprite_index`, `x`, `y`, `sprite_id`                                                                                                 | Embedded sprite placements                |

### Tables Used by External Entity Rendering

| Table          | Source                      | Description                             |
|----------------|-----------------------------|-----------------------------------------|
| `map_inis`     | `references/map_ini.rs`     | Map config with ref filenames           |
| `maps`         | `database.rs`               | Map file → map_id mapping               |
| `monster_refs` | `references/monster_ref.rs` | Monster placements                      |
| `monster_inis` | `references/monster_ini.rs` | Monster visual config (sprite filename) |
| `npc_refs`     | `references/npc_ref.rs`     | NPC placements                          |
| `npc_inis`     | `references/npc_ini.rs`     | NPC visual config (sprite filename)     |
| `extra_refs`   | `references/extra_ref.rs`   | Extra object placements                 |
| `extras`       | `references/extra_ini.rs`   | Extra object config (sprite filename)   |

---

## JSON Schema for Map Data Export

The following JSON schema describes the structure for exporting `.MAP` file data:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DispelMapData",
  "description": "Complete parsed representation of a Dispel game .MAP file",
  "type": "object",
  "required": [
    "metadata",
    "gtl_tiles",
    "btl_tiles",
    "collisions",
    "events",
    "tiled_objects",
    "sprites"
  ],
  "properties": {
    "metadata": {
      "type": "object",
      "required": [
        "chunk_width",
        "chunk_height",
        "tiled_width",
        "tiled_height"
      ],
      "properties": {
        "chunk_width": {
          "type": "integer",
          "description": "Number of 25-tile chunks on X axis"
        },
        "chunk_height": {
          "type": "integer",
          "description": "Number of 25-tile chunks on Y axis"
        },
        "tiled_width": {
          "type": "integer",
          "description": "Total tile count on X axis (chunk_width * 25 - 1)"
        },
        "tiled_height": {
          "type": "integer",
          "description": "Total tile count on Y axis (chunk_height * 25 - 1)"
        },
        "map_width_in_pixels": {
          "type": "integer",
          "description": "Full rendered image width in pixels"
        },
        "map_height_in_pixels": {
          "type": "integer",
          "description": "Full rendered image height in pixels"
        },
        "non_occluded_start_x": {
          "type": "integer",
          "description": "Visible viewport X offset"
        },
        "non_occluded_start_y": {
          "type": "integer",
          "description": "Visible viewport Y offset"
        },
        "occluded_width": {
          "type": "integer",
          "description": "Cropped image width"
        },
        "occluded_height": {
          "type": "integer",
          "description": "Cropped image height"
        }
      }
    },
    "gtl_tiles": {
      "type": "array",
      "description": "Ground tile assignments per coordinate",
      "items": {
        "type": "object",
        "required": [
          "x",
          "y",
          "tile_id"
        ],
        "properties": {
          "x": {
            "type": "integer",
            "minimum": 0
          },
          "y": {
            "type": "integer",
            "minimum": 0
          },
          "tile_id": {
            "type": "integer",
            "description": "Index into the .GTL tileset"
          }
        }
      }
    },
    "btl_tiles": {
      "type": "array",
      "description": "Roof/building tile assignments per coordinate",
      "items": {
        "type": "object",
        "required": [
          "x",
          "y",
          "tile_id"
        ],
        "properties": {
          "x": {
            "type": "integer",
            "minimum": 0
          },
          "y": {
            "type": "integer",
            "minimum": 0
          },
          "tile_id": {
            "type": "integer",
            "description": "Index into the .BTL tileset"
          }
        }
      }
    },
    "collisions": {
      "type": "array",
      "description": "Collision flags per coordinate",
      "items": {
        "type": "object",
        "required": [
          "x",
          "y",
          "blocked"
        ],
        "properties": {
          "x": {
            "type": "integer",
            "minimum": 0
          },
          "y": {
            "type": "integer",
            "minimum": 0
          },
          "blocked": {
            "type": "boolean",
            "description": "Whether this tile blocks movement"
          }
        }
      }
    },
    "events": {
      "type": "array",
      "description": "Event triggers per coordinate",
      "items": {
        "type": "object",
        "required": [
          "x",
          "y",
          "event_id"
        ],
        "properties": {
          "x": {
            "type": "integer",
            "minimum": 0
          },
          "y": {
            "type": "integer",
            "minimum": 0
          },
          "event_id": {
            "type": "integer",
            "description": "Event trigger ID (0 = no event)"
          }
        }
      }
    },
    "tiled_objects": {
      "type": "array",
      "description": "Buildings and objects made of stacked BTL tiles",
      "items": {
        "type": "object",
        "required": [
          "index",
          "x",
          "y",
          "tile_ids"
        ],
        "properties": {
          "index": {
            "type": "integer",
            "description": "Object index (0-based)"
          },
          "x": {
            "type": "integer",
            "description": "Tile X coordinate"
          },
          "y": {
            "type": "integer",
            "description": "Tile Y coordinate"
          },
          "tile_ids": {
            "type": "array",
            "items": {
              "type": "integer"
            },
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
        "required": [
          "index",
          "sprite_id",
          "x",
          "y"
        ],
        "properties": {
          "index": {
            "type": "integer",
            "description": "Placement index (0-based)"
          },
          "sprite_id": {
            "type": "integer",
            "description": "Index into internal_sprites array"
          },
          "x": {
            "type": "integer",
            "description": "Pixel X position on map"
          },
          "y": {
            "type": "integer",
            "description": "Pixel Y position on map"
          }
        }
      }
    },
    "internal_sprites": {
      "type": "array",
      "description": "Embedded sprite sequence definitions",
      "items": {
        "type": "object",
        "required": [
          "index",
          "image_stamp",
          "frame_count",
          "frames"
        ],
        "properties": {
          "index": {
            "type": "integer",
            "description": "Sprite index (0-based)"
          },
          "image_stamp": {
            "type": "integer",
            "enum": [
              6,
              9
            ],
            "description": "Data layout variant"
          },
          "frame_count": {
            "type": "integer",
            "description": "Number of animation frames"
          },
          "frames": {
            "type": "array",
            "items": {
              "type": "object",
              "required": [
                "width",
                "height",
                "origin_x",
                "origin_y"
              ],
              "properties": {
                "width": {
                  "type": "integer",
                  "minimum": 0
                },
                "height": {
                  "type": "integer",
                  "minimum": 0
                },
                "origin_x": {
                  "type": "integer",
                  "description": "Anchor offset X"
                },
                "origin_y": {
                  "type": "integer",
                  "description": "Anchor offset Y"
                }
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
  "required": [
    "tile_count",
    "tile_width",
    "tile_height",
    "color_format",
    "tiles"
  ],
  "properties": {
    "tile_count": {
      "type": "integer",
      "description": "Total number of tiles"
    },
    "tile_width": {
      "type": "integer",
      "const": 32,
      "description": "Tile pixel width"
    },
    "tile_height": {
      "type": "integer",
      "const": 32,
      "description": "Tile pixel height"
    },
    "rendered_width": {
      "type": "integer",
      "const": 62,
      "description": "Isometric diamond width"
    },
    "rendered_height": {
      "type": "integer",
      "const": 32,
      "description": "Isometric diamond height"
    },
    "color_format": {
      "type": "string",
      "const": "RGB565",
      "description": "Source color encoding"
    },
    "file_type": {
      "type": "string",
      "enum": [
        "gtl",
        "btl"
      ],
      "description": "Ground or building tile layer"
    },
    "tiles": {
      "type": "array",
      "items": {
        "type": "object",
        "required": [
          "index",
          "pixels"
        ],
        "properties": {
          "index": {
            "type": "integer",
            "description": "Tile index (0-based, used as tile_id in map)"
          },
          "pixels": {
            "type": "array",
            "items": {
              "type": "object",
              "required": [
                "x",
                "y",
                "r",
                "g",
                "b"
              ],
              "properties": {
                "x": {
                  "type": "integer",
                  "minimum": 0,
                  "maximum": 31
                },
                "y": {
                  "type": "integer",
                  "minimum": 0,
                  "maximum": 31
                },
                "r": {
                  "type": "integer",
                  "minimum": 0,
                  "maximum": 255
                },
                "g": {
                  "type": "integer",
                  "minimum": 0,
                  "maximum": 255
                },
                "b": {
                  "type": "integer",
                  "minimum": 0,
                  "maximum": 255
                }
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

| Offset       | Size  | Field                    | Notes                                                  |
|--------------|-------|--------------------------|--------------------------------------------------------|
| 0            | 4     | `chunk_width`            | i32 LE                                                 |
| 4            | 4     | `chunk_height`           | i32 LE                                                 |
| 8            | 4     | `border_count`           | i32 LE (always 2)                                      |
| 12           | 4     | `object_ref_count`       | i32 LE                                                 |
| 16           | var   | `object_ref_records`     | Skip `(count − 1) × 8` bytes; each record's `value2` is a linear tile index into the end grids |
| var          | 4     | `overlay_table_size`     | i32 LE                                                 |
| var          | var   | `overlay_table`          | `size × 2` bytes; u16 entries `{transparency, draw_enable}` indexed by Access-Ref ids |
| var          | 4     | `sprite_count`           | i32 LE                                                 |
| var          | var   | `sprite entries`         | Each: stamp(4) + meta(264) + sequence_info + padding   |
| var          | 4     | `sprite_placement_count` | i32 LE                                                 |
| var          | var   | `sprite placements`      | Each: sprite_id(4) + frame-0 bbox {left,top,right,bottom} + dup anchor {x,y}, then `(frame_count − 1) × 24` bytes |
| var          | 4     | `tiled_object_count`     | i32 LE                                                 |
| var          | 4     | `tiled_object_subrecords`| i32 LE                                                 |
| var          | var   | `tiled objects`          | Each: 264 + control(4×i32) + params(6×i32) + anchor(x,y) + counts(3×i32) + tile_stack(c3×i16) + 84 + (c1+c2+c3)×4 |
| EOF-(w×h×12) | w×h×4 | `event grid`             | Each packed u32: low 14 bits = event id, high half = flags (bit 22 = tile marked) |
| EOF-(w×h×8)  | w×h×4 | `tile & access`          | Each packed u32: bit 0 collision, bits 1–9 object slot, bits 10–24 GTL tile index |
| EOF-(w×h×4)  | w×h×4 | `access-ref ("roof")`    | Each packed u32: bits 0–14 overlay id → overlay_table, bits 15–29 shadow level 0–199, bits 30–31 light flags |

Where `w = tiled_map_width`, `h = tiled_map_height`.

---

## External Entity Rendering Colors

When `game_path` is not provided or sprite files are missing, entities are rendered as colored diamond markers:

| Entity Type | Color (RGBA)                     | Sprite Directory |
|-------------|----------------------------------|------------------|
| Monsters    | `rgba(255, 60, 60, 255)` — red   | `MonsterInGame/` |
| NPCs        | `rgba(60, 255, 60, 255)` — green | `NpcInGame/`     |
| Extras      | `rgba(80, 120, 255, 255)` — blue | `ExtraInGame/`   |

---

## Notes

1. **Two unknown blocks** at the start of `.MAP` files are skipped — their purpose is not yet understood
2. **Image stamps** (6 or 9) determine the padding size after sprite sequence data (1904 or 2996 bytes respectively)
3. **Tiled object sentinel** — the end of the tiled objects block is detected by scanning 20 bytes backwards for the
   byte value 1
4. **Roof block is optional** — only parsed if remaining file size is sufficient
5. **Collision flag** is extracted from bit 0 of the packed tile value; tile index is bits 10–24
6. **All integers are little-endian**
7. **Sprite sequences** in the map file are self-contained — pixel data is embedded directly, not referenced externally
