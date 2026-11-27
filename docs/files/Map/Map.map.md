# DISPEL® Map File Format (`.map`)

The tool reads overworld, catacomb and dungeon maps with one binary format. Sample names include `map1.map`, `cat1.map`, `catp.map`, `dun01.map` and `final.map`. A `.map` file holds dimensions, embedded sprites, building stacks and three per-tile grids for events, ground tiles and occlusion.

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
records: (count − 1) × 8 bytes   -- each record: word0: i32, word1: i32
```

The tool skips `(count − 1) × 8` bytes and lands on the next block's size field. The cat1, cat3, dun01, map1 and catp samples contain 99,104 records. In those records `word0 == 0` and `word1 == 1`. The supplied data shows no use for these fields.

### Second Block *(u16 lookup table)*

```
size: i32
data: size × 2 bytes             -- table of u16 entries
```

Not dead data: the Access-Ref block indexes into this table. Each u16 entry is
`{low byte: transparency mode (0 = opaque copy, 1 = skip transparent pixels),
high byte: draw-enable flag}`. All non-zero ids observed fall inside the table
bounds.

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

Any other `image_stamp` is a parse error. These sprites use pixel coordinates rather than tile grid coordinates.

### Sprite Info Block *(placements)*

```
placement_count: i32
for each placement:
    sprite_id: i32               -- index into the Sprite Block
    bbox_left: i32               -- frame bounding box in map-local pixels
    bbox_top: i32
    bbox_right: i32              -- == left + frame width
    bottom_right_y: i32          -- == top + frame height (Y-sort key)
    x: i32                       -- duplicates bbox_left
    y: i32                       -- duplicates bbox_top
    frame_skip: (frame_count − 1) × 6 × 4 bytes
```

The seven i32 values after `sprite_id` hold frame 0 of a per-frame record. Each record has 24 bytes: a `{left, top, right, bottom}` box and a duplicated `{x, y}` anchor. The format stores these values in separate per-frame arrays. In the cat1 and map1 samples, `right − left` and `bottom − top` equal the frame pixel `width × height`. Placements sort by `bottom` on the Y axis.

### Tiled Objects Block *(buildings)*: fully typed

This block is a three-level tree: bundle, record, item and entry. The structure below follows the on-disk data.

```
bundle_count: i32
for each bundle:
    record_count: i32
    for each record:
        field_04: i32                       -- observed as 0 in shipped maps
        body: 260 bytes                     -- binary metadata, not text
        item_count: i32
        for each item:
            type_flag: i32                  -- 1 ⇒ entries carry extra bytes
            entry_count: i32
            field_14: i32                   -- always 0 in all shipped maps
            for each entry:
                bound_x, bound_y,           -- bounding box, map pixels:
                bound_right, bound_bottom     right = x + 64 and
                                              bottom = y + grid_height*32,
                                              exact across all shipped entries
                anchor_x, anchor_y          -- stack anchor, map-local pixels
                draw_x, draw_y              -- position terms used during
                                            -- drawing relative to the camera
                grid_width                  -- constant 1 in all shipped maps
                grid_height                 -- equals ids.len()
                stored_cell_count           -- equals grid_width*grid_height
                                            -- in all shipped maps
                ids: stored_cell_count × u16    -- BTL tile stack of this entry
                if type_flag == 1:
                    extra_payload: stored_cell_count bytes
    n = first_record.first_item.entries[0].grid_height
    level_flags: n × (flag_a: i32, flag_b: i32)  -- two binary 0/1 flags per
                                    -- stack level; their meaning is unknown
```

There is no end-of-block sentinel. The three end grids follow the last bundle's level flags immediately.

The 33 sample maps contain 43,554 entries. Those entries use this layout:
- `anchor_x` and `anchor_y` give the stack anchor `(x, y)` in map-local pixels. The entry's `ids` u16 array is its BTL tile stack. The renderer-facing `TiledObjectInfo` comes from the bundle's first entry (first record, first item, first entry).
- The 43,554 entries follow the bounding-box relations above.
- An entry's `grid_height` sizes the per-bundle flag arrays that follow.

The sample maps contain one record per bundle, eight items per record and at most one entry per item. `type_flag` and `field_14` are zero in those files.

The tool draws each bundle's first-entry tile stack downward from the anchor. It draws one 62×32 diamond per id. Negative ids occur. The tool skips them when it draws.

### Event Block

One record per tile, row-major:

```
event_id: i16                    -- low half of a packed u32; low 14 bits hold
                                   the id (ids < 70 are considered valid;
                                   resolvable through Map.ini / AllMap.ini)
flags: i16                       -- high half: parameter/flag bits
```

In the packed u32, bit 22 sets `EventBlock::is_tile_marked`.

Two documented checks use this exact event-word predicate:

```text
(word & 0x0FC00000) == 0x00400000
```

- `EventBlock::is_under_roof_marker` identifies a player tile that hides the
  BTL pass.
- `EventBlock::is_no_spell_cast_zone` identifies a visible tile that prevents
  a player from preparing a spell.

The predicate is the same, but the systems use different tile scopes. The
remaining high bits have no documented meaning.

### Tile & Access Block

One packed u32 per tile, row-major:

| Bits  | Meaning                                                      |
|-------|--------------------------------------------------------------|
| 0     | Collision flag (tile blocked)                                |
| 1–9   | Object slot id (0–511); non-zero marks an interactive object |
| 10–24 | GTL ground-tile index — bits 10–24 of the word               |
| 25–31 | Unused (always 0) |

The index points straight into the `.gtl` pixel data: the renderer blits from
`gtl_base + index × 2048` (one 32×32 RGB565 tile). Across observed maps the maximum index matches the `.gtl` tile
capacity minus one.

Bits 0 to 9 form one access field. The tool can rewrite that field without changing the tile index. The sample maps use bit 0 only. No sample value sets bits 1 to 9.

### Access-Ref Block *(a.k.a. "roof" block)*

One record per tile, row-major:

```
ref_id: i16                      -- bits 0–14: index into the Second Block table
shadow_and_flags: i16            -- bits 15–29 of the u32: shadow level 0–199;
                                   bits 30–31: light-source flags
```

This grid packs **two layers** in one u32:

1. BTL overlay ref (bits 0 to 14): when the referenced Second Block entry has its high byte set, the renderer blits BTL pixels from
   `btl_base + ref_id × 2048` (entry *low* byte selects transparent vs opaque
   blit). So the Second Block is `{transparency, draw_enable}` per overlay id.
2. Shadow and fog level (bits 15 to 29): the value spans 0 to 199. The documented rule raises the value for light sources. The highest value wins.

On a dark map, a BTL overlay must have a non-zero reference and at least one
bit in positions 15–31. The extractor exposes this condition as
`btl_overlay_draws_in_dark_map`. It is not a guarantee that the overlay draws:
the referenced Second Block entry must also have its draw-enable byte set.
Light maps use a different roof pass and do not apply this extra condition.

Bit 15 overlaps the signed-i16 view of `ref_id`. Readers must mask the low 15 bits. A negative i16 value does not mean the reference is absent.

---

## Coordinate System

Tiles form an isometric diamond grid. Tile `(x, y)` renders at:

```
pixel_x = (x + y) × 32
pixel_y = (y − x) × 16 + (map_diagonal / 2) × 16
```

Here `map_diagonal = tiled_width + tiled_height`. A step of +1 in `x` moves the tile by (−32, −16) on screen. A step of +1 in `y` moves it by (+32, +16).

Sprite and building placements in the file use map-local coordinates. Subtract `map_non_occluded_start_x/y`, computed in `src/map/model.rs`, to convert them to world pixels.

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
- Renderer: `render_map()` in `src/map/render.rs`. The GUI canvas is in
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
