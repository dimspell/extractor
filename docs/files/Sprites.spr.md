# *.spr — Sprite File Format

> **DISPEL®** is a registered trademark. This project is not affiliated with,
> endorsed by, or sponsored by the trademark owner.

## Scope

This document describes the `.spr` layout that this project reads and writes.
All integer values use little-endian byte order.

## File Summary

- **Extension:** `.spr`
- **Data:** counted sequence records
- **Pixel data:** one or two bytes per pixel
- **Two-byte formats:** RGB555 or RGB565

## File Layout

| Offset | Size | Description |
|---|---:|---|
| `0` | 4 bytes | Sequence-record count (`i32`) |
| `4` | Variable | Sequence records |

The file has no footer.

## Sequence Record

Each sequence record starts with a 268-byte header.

| Offset | Size | Description |
|---|---:|---|
| `0` | 4 bytes | Sequence identifier (`i32`) |
| `4` | 260 bytes | Name or reserved data |
| `264` | 4 bytes | Group count (`i32`) |

The group data follows this header.

## Group Record

Each group starts with a 12-byte header.

| Offset | Size | Description |
|---|---:|---|
| `0` | 4 bytes | Variant (`i32`) |
| `4` | 4 bytes | Frame count (`i32`) |
| `8` | 4 bytes | Reserved data (`i32`) |

The frames for this group follow the group header.

The parser uses `variant == 1` for one-byte pixels. Other variants use
two-byte pixels. The parser does not render one-byte pixels because it has no
palette for them.

## Frame Record

Each frame has a 44-byte header followed by its pixel data.

| Offset | Size | Description |
|---|---:|---|
| `0` | 24 bytes | Reserved data |
| `24` | 4 bytes | Origin X (`i32`) |
| `28` | 4 bytes | Origin Y (`i32`) |
| `32` | 4 bytes | Width in pixels (`i32`) |
| `36` | 4 bytes | Height in pixels (`i32`) |
| `40` | 4 bytes | Pixel count (`u32`) |

The pixel data begins immediately after the frame header.

Calculate its size as follows:

```text
pixel_data_size = pixel_count × bytes_per_pixel
```

## Two-Byte Pixel Data

Two-byte pixels use RGB555 or RGB565 packing.

| Format | Red | Green | Blue |
|---|---|---|---|
| RGB565 | bits 15–11 | bits 10–5 | bits 4–0 |
| RGB555 | bits 14–10 | bits 9–5 | bits 4–0 |

Pixels use row-major order. The first pixel is at the top-left corner.

The default renderer treats `0x0000` as transparent. Use `--preserve-black`
to export this value as opaque `#000000`.

### Format Selection

The layout has no field that selects RGB555 or RGB565.

The parser uses this rule for an archive with two-byte pixels:

1. If a non-zero pixel has bit 15 set, use RGB565.
2. Otherwise, use RGB555.
3. If all pixels are zero, use RGB565.

This rule cannot identify an RGB565 archive that never sets bit 15. Add an
explicit format option if such a file needs different handling.

## Frame Position

`origin_x` and `origin_y` define the reference point in a frame. Use these
values to place frames in a common image area.

```text
max_left  = max(origin_x)
max_right = max(width - origin_x)
max_up    = max(origin_y)
max_down  = max(height - origin_y)

canvas_width  = max_left + max_right
canvas_height = max_up + max_down

x = max_left - origin_x
y = max_up - origin_y
```

For a single frame, use its own width and height.

## Read a File

1. Read the sequence-record count.
2. Read each sequence header.
3. Read each group header.
4. Read the frame headers and pixel data for that group.
5. Use `pixel_count` and the byte stride to move to the next frame.

## Command-Line Use

```bash
# Export individual frames.
cargo run -- sprite "path/to/file.spr"

# Export animation strips. The exporter aligns frames by their origin values.
cargo run -- sprite "path/to/file.spr" --mode animation

# Export animation strips without origin alignment.
cargo run -- sprite "path/to/file.spr" --mode animation --no-origin-offset

# Keep raw 0x0000 pixels as opaque black in exported PNG files.
cargo run -- sprite "path/to/file.spr" --preserve-black

# Print metadata as JSON.
cargo run -- sprite "path/to/file.spr" --info
```

## Library API

```rust
use dispel_core::sprite::{
    get_sequence_pngs_by_index,
    get_sprite_info,
    get_sprite_metadata,
};

let frame_counts = get_sprite_metadata(Path::new("file.spr"))?;
let info = get_sprite_info(Path::new("file.spr"))?;
let pngs = get_sequence_pngs_by_index(Path::new("file.spr"), 0)?;
```

`get_sequence_pngs_by_index` returns one PNG buffer for each frame.
