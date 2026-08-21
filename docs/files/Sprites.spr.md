# *.spr — Sprite File Format

> **DISPEL®** is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
> owner.

## File Information

- **Location**: `CharacterInGame/*.spr`, `NpcInGame/*.spr`, `MonsterInGame/*.spr`, `ExtraInGame/*.spr`
- **Format**: Binary (Little-Endian)
- **Color Format**: RGB565 (2 bytes per pixel)
- **File Extension**: `.spr`

---

## File Layout

```
Offset      Size          Description
----------  ------------  -------------------------------------------
0           268 bytes     Unknown file header (skip to offset 268)
268         variable      Sequence 1
...         variable      Sequence 2
...         variable      Sequence N
```

The file ends with the last byte of the last sequence's pixel data. There is no footer or sequence count in the header.

---

## Sequence Detection

Sequences are not stored at fixed offsets. They must be found by scanning the file for a valid sequence header pattern.
The algorithm reads 15 consecutive i32 values (60 bytes) and checks two valid patterns:

**Pattern A (no leading stamp):**

| Field                 | Constraint     | Meaning                       |
|-----------------------|----------------|-------------------------------|
| `ints[0]`             | `== 0`         | No stamp                      |
| `ints[1]`             | `> 0 && < 255` | Frame count (1–254 frames)    |
| `ints[2]`             | `== 0`         | Padding                       |
| `ints[11]`            | `> 0`          | First frame width             |
| `ints[12]`            | `> 0`          | First frame height            |
| `ints[11] * ints[12]` | `== ints[13]`  | Width × height == pixel count |

**Pattern B (with leading stamp 8, 0):**

| Field                 | Constraint     | Meaning                       |
|-----------------------|----------------|-------------------------------|
| `ints[0]`             | `== 8`         | Stamp byte                    |
| `ints[1]`             | `== 0`         | Stamp padding                 |
| `ints[2]`             | `> 0 && < 255` | Frame count (1–254 frames)    |
| `ints[3]`             | `== 0`         | Padding                       |
| `ints[12]`            | `> 0`          | First frame width             |
| `ints[13]`            | `> 0`          | First frame height            |
| `ints[12] * ints[13]` | `== ints[14]`  | Width × height == pixel count |

If neither pattern matches, the scanner advances by 4 bytes (one i32) and retries.

---

## Sequence Header

Once a valid pattern is found, the sequence header is read:

| Offset (relative) | Size  | Description                                                 |
|-------------------|-------|-------------------------------------------------------------|
| `0`               | `i32` | Stamp: `8` (Pattern B) or `0` (Pattern A)                   |
| `4`               | `i32` | If stamp was `8`: always `0`. If stamp was `0`: frame count |
| `8`               | `i32` | Frame count (Pattern A only)                                |
| `12`              | `i32` | Always `0` (padding)                                        |

After the header, frame metadata begins immediately.

---

## Frame Metadata (per frame)

Each frame in the sequence has a metadata block:

| Offset | Size  | Description                                         |
|--------|-------|-----------------------------------------------------|
| `0`    | `24`  | Unknown data (6 × i32, skipped during parsing)      |
| `24`   | `i32` | `origin_x` — X offset from anchor point             |
| `28`   | `i32` | `origin_y` — Y offset from anchor point             |
| `32`   | `i32` | `width` — frame width in pixels                     |
| `36`   | `i32` | `height` — frame height in pixels                   |
| `40`   | `u32` | `pixel_count` — number of RGB565 pixels (NOT bytes) |

The pixel data size in bytes is: `pixel_count * 2`.

After the metadata block, pixel data follows immediately.

---

## Frame Pixel Data

Each pixel is stored as RGB565 (16-bit, Little-Endian):

| Bits  | Channel | Range         |
|-------|---------|---------------|
| 15–11 | Red     | 5 bits (0–31) |
| 10–5  | Green   | 6 bits (0–63) |
| 4–0   | Blue    | 5 bits (0–31) |

**Conversion to 8-bit RGB:**

```
R = (red_value   << 3)   // 5→8 bits, range 0–248
G = (green_value << 2)   // 6→8 bits, range 0–252
B = (blue_value  << 3)   // 5→8 bits, range 0–248
```

Pixel value `0` (all channels zero) is transparent.

Pixels are stored in row-major order (left-to-right, top-to-bottom).

---

## Bounding Rectangle Calculation

Frames within a sequence may have different sizes and origins. To render them into a common canvas, a bounding rectangle
is computed:

```
max_left  = max(origin_x)
max_right = max(width - origin_x)
max_up    = max(origin_y)
max_down  = max(height - origin_y)

canvas_width  = max_left + max_right   (or single frame width)
canvas_height = max_up + max_down      (or single frame height)
```

Each frame is placed at offset:

```
x = max_left - origin_x
y = max_up - origin_y
```

---

## Reading a File (Step-by-Step)

1. Open the file, seek to offset 268.
2. Call `seek_next_sequence()` to find the next valid sequence header. If it returns `false`, you've reached the end of
   the file.
3. Call `get_sequence_info()` to parse the header and all frame metadata. This advances the reader past all pixel data
   to the end of the sequence.
4. To render frames, seek back to `sequence_start_position` before reading pixel data.
5. Seek to `sequence_end_position` to continue scanning for the next sequence.
6. Repeat steps 2–5 until `seek_next_sequence()` returns `false`.

---

## File Purpose

Stores character sprites, animations, and visual effects for:

- NPCs and monsters
- Party members
- Special effects
- Interactive objects

---

## Sprite Types

- Character Sprites
- Monster Sprites
- NPC Sprites
- Objects

---

## Origin Points

Each frame has an origin point (`origin_x`, `origin_y`) that defines:

- The reference point for positioning
- Rotation center
- Collision detection point

---

## Related Files

- `Npc.ini` — NPC definitions referencing sprites
- `Monster.ini` — Monster definitions referencing sprites
- `Extra.ini` — Object definitions referencing sprites
- `*.map` files — Map files containing embedded sprites

---

## Extractor

The file is parsed by the `src/sprite.rs` module.

### CLI Commands

```bash
# Extract individual frames as separate PNG files
cargo run -- sprite "path/to/file.spr"

# Extract animation sequences as horizontal strip PNGs
cargo run -- sprite "path/to/file.spr" --mode animation

# Output sprite metadata as JSON (no rendering)
cargo run -- sprite "path/to/file.spr" --info
```

### Library API

```rust
use dispel_core::sprite::{
    get_sprite_info,         // Get full metadata as JSON-serializable struct
    get_sprite_metadata,     // Get frame counts per sequence
    get_sequence_pngs_by_index, // Render a specific sequence to PNG buffers
};

// Get all sequences and frame counts
let frame_counts = get_sprite_metadata(Path::new("file.spr"))?;

// Get full metadata
let info = get_sprite_info(Path::new("file.spr"))?;

// Render sequence 0 to PNG buffers
let pngs = get_sequence_pngs_by_index(Path::new("file.spr"), 0)?;
// => Vec<Vec<u8>> — one PNG buffer per frame
```

### Animation Sequences

Sprites can contain multiple animation sequences:

- Walking cycles
- Attack animations
- Special ability effects
- Idle animations
- Death sequences
