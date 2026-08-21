# DISPEL® Map File Format (`.map`)

All game maps — overworld (`map1.map`…), catacombs (`cat1.map`…, `catp.map`), and dungeons (`dun01.map`…, `final.map`) —
share this binary format. A `.map`
file contains everything needed to render a map: dimensions, embedded sprites (thrones, statues, decor), building
stacks, and three per-tile grids (events, ground tiles, occlusion).

> DISPEL® is a registered trademark. This project is not affiliated with,
> endorsed by, or sponsored by the trademark owner.

## Quick Facts

| Property      | Value                                         |
|---------------|-----------------------------------------------|
| Location      | `Map/*.map`                                   |
| Endianness    | Little-endian throughout                      |
| Grid unit     | Tile, stored row-major (`y` outer, `x` inner) |
| Source tile   | 32×32 px, RGB565 (2048 bytes)                 |
| Rendered tile | 62×32 px isometric diamond                    |
| Chunk         | 25×25 tiles (dimensions are stored in chunks) |

## File Layout

Blocks appear strictly in this order:

```
┌─────────────────────────────────────────────┐
│ Header                       12 bytes       │
│ First block                  variable       │  skipped by our parser
│ Second block                 variable       │  kept: referenced by Access-Ref
│ Sprite block                 variable       │  embedded sprite sequences
│ Sprite Info block            variable       │  where to place them
│ Tiled Objects block          variable       │  buildings (BTL tile stacks)
│ Event block                  width×height×4 │
│ Tile & Access block          width×height×4 │
│ Access-Ref block ("roof")    width×height×4 │
└─────────────────────────────────────────────┘
```

The last three blocks are dense grids of exactly `width_tiles × height_tiles`
entries each, so a reader can seek to them from the end of the file.

---

## Block Reference

### Header (12 bytes)

| Field              | Type | Meaning                       |
|--------------------|------|-------------------------------|
| `width_in_chunks`  | i32  | Map width in 25-tile chunks   |
| `height_in_chunks` | i32  | Map height in 25-tile chunks  |
| `border_count`     | i32  | Border chunk count (always 2) |

### First Block *(unused)*

```
count: i32
records: (count − 1) × 8 bytes   -- each record: value1: i32, value2: i32
```

Skipping `(count − 1) × 8` lands exactly on the next block's size field, verified against the cat1/cat3/dun01/map1/catp
fixtures. The pairs appear unused by our tools; they are kept here for format completeness only.

### Second Block *(u16 lookup table)*

```
size: i32
data: size × 2 bytes             -- table of u16 entries
```

Not dead data: the Access-Ref block indexes into this table, and each entry's low byte acts as a boolean flag consumed
by occlusion/access checks. All non-zero ids observed fall inside the table bounds.

### Sprite Block *(embedded sprites)*

```
sprite_count: i32
for each sprite:
    image_stamp: i32             -- 6 or 9
    metadata: 264 bytes
    sequence_info: variable      -- frames, timing (see src/sprite.rs)
    pixel_data: variable         -- length depends on image_stamp:
                                    6 → 1904 bytes, 9 → 2996 bytes
```

Any other `image_stamp` is a parse error. Sprites here are thrones, statues, decor — anything placed by pixel coordinate
rather than tile grid.

### Sprite Info Block *(placements)*

```
placement_count: i32
for each placement:
    sprite_id: i32               -- index into the Sprite Block
    unknown: 2 × i32
    bottom_right_x: i32          -- Y-sort anchor (see below)
    bottom_right_y: i32
    x: i32                       -- top-left placement, map-local pixels
    y: i32
    frame_skip: (frame_count − 1) × 6 × 4 bytes
```

`bottom_right_y` equals `y + sprite_height` and serves as the depth key for interlaced rendering.

### Tiled Objects Block *(buildings)*

```
bundle_count: i32
number1: i32                     -- always observed as 1
for each bundle:
    metadata: 264 bytes
    control: 4 × i32             -- expected pattern: 8, 0, 1, 0
    v1..v4: 4 × i32              -- unknown
    x: i32                       -- stack anchor, map-local pixels
    y: i32
    v7, v8: 2 × i32              -- unknown
    c1, c2, c3: 3 × i32          -- counts
    tile_ids: c3 × i16           -- BTL tile stack, top → bottom
    unknown_a: 84 bytes
    unknown_b: (c1 + c2 + c3) × 4 bytes
<end sentinel alignment>         -- see tiled_objects_block() in reader.rs
```

Each bundle is one building: its `tile_ids` stack is drawn downward from the anchor, one 62×32 diamond per entry.
Negative ids occur and are skipped when drawing.

### Event Block

One record per tile, row-major:

```
event_id: i16                    -- low 14 bits hold the id (ids < 70 are
                                   considered valid; resolvable through the
                                   Map.ini / AllMap.ini tables)
unknown: i16                     -- parameters/flags, semantics TBD
```

### Tile & Access Block

One packed u32 per tile, row-major (bit layout verified against shipped map data):

| Bits  | Meaning                                                      |
|-------|--------------------------------------------------------------|
| 0     | Collision flag (tile blocked)                                |
| 1–9   | Object slot id (0–511); non-zero marks an interactive object |
| 10–24 | GTL ground-tile index — bits 10–24 of the word               |
| 25–31 | Unused (always 0) |

The index points straight into the `.gtl` pixel data: the renderer blits from
`gtl_base + index × 2048` (one 32×32 RGB565 tile). Across observed maps the maximum index matches the `.gtl` tile
capacity minus one.

Bits 0–9 together form one access field that can be rewritten without touching the tile index. Maps only ever
use bit 0; bits 1–9 are written at runtime.

### Access-Ref Block *(a.k.a. "roof" block)*

One record per tile, row-major:

```
ref_id: i16                      -- bits 0–14: index into the Second Block table
flag: i16                        -- rarely non-zero (a handful of border tiles)
```

Despite the historical name, this grid drives **occlusion/access**, not visuals — roof pixels come from the Tiled
Objects block. Each referenced table entry's low byte is a boolean flag used by occlusion checks.

---

## Coordinate System

Tiles form an isometric diamond grid. Tile `(x, y)` renders at:

```
pixel_x = (x + y) × 32
pixel_y = (y − x) × 16 + (map_diagonal / 2) × 16
```

where `map_diagonal = tiled_width + tiled_height`. Moving `+1` in `x` steps up-left on screen (−32, −16); moving `+1` in
`y` steps down-right (+32, +16).

Sprite and building placements inside the file are **map-local**: subtract
`map_non_occluded_start_x/y` (computed in `src/map/model.rs`) to convert to world pixels.

## File Size Calculation

```
fixed_tail = width_tiles × height_tiles × 12   (Event 4 + Tile&Access 4 + Access-Ref 4)
total      = header + variable blocks + fixed_tail
```

## Related Files

| File                     | Role                                               |
|--------------------------|----------------------------------------------------|
| `*.gtl`                  | Ground tileset (raw 32×32 RGB565 tiles, no header) |
| `*.btl`                  | Building/roof tileset (same raw format)            |
| `AllMap.ini` / `Map.ini` | Map metadata, per-map monster/NPC/extra ref files  |
| `Mon*/Npc*/Ext*.ref`     | External entities placed on the map                |

## Implementation & Usage

- Parser: `read_map_data()` in `src/map/mod.rs`, block readers in `src/map/reader.rs`
- Renderer: `render_map()` in `src/map/render.rs`; GUI canvas in
  `dispel-gui/src/components/map_render/`

```bash
# Render a map to PNG
cargo run -- map render \
  --map "fixtures/Dispel/Map/cat1.map" \
  --btl "fixtures/Dispel/Map/cat1.btl" \
  --gtl "fixtures/Dispel/Map/cat1.gtl" \
  --output cat1.png

# Extract embedded sprites
cargo run -- map sprites "fixtures/Dispel/Map/cat1.map"

# Import map data to SQLite
cargo run -- map to-db --database "database.sqlite" --map "fixtures/Dispel/Map/cat1.map"
```
