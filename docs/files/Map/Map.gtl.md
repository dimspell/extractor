# DISPEL® Ground Tile Layer (`.gtl`)

The tool reads `.gtl` files as raw tilesets for terrain, paths and natural features. The binary format matches `.btl`.

## Quick Facts

| Property      | Value                           |
|---------------|---------------------------------|
| Location      | `Map/*.gtl`                     |
| Header        | None                            |
| Tile size     | 32×32 px, RGB565                |
| Tile data     | 2048 bytes per tile (32×32 × 2) |
| Rendered size | 62×32 px isometric diamond      |
| Transparency  | none — the ground pass is an opaque row-copy |

## File Layout

No header. The file is a contiguous sequence of 32×32 pixel tiles in RGB565 (little-endian u16) format. Tile count =
`file_size / 2048`.

```
┌──────────────────────┐
│ TILE #0              │
│   pixels: u16 × 1024 │  RGB565, little-endian
├──────────────────────┤
│ TILE #1              │
│   pixels: u16 × 1024 │
├──────────────────────┤
│ ...                  │
├──────────────────────┤
│ TILE #N              │
│   pixels: u16 × 1024 │
└──────────────────────┘
```

## RGB565 → RGB888 Conversion

```
red   = bits 11–15 of pixel   → scale 0–31  to 0–255
green = bits 5–10 of pixel    → scale 0–63  to 0–255
blue  = bits 0–4 of pixel     → scale 0–31  to 0–255
```

## Isometric Rendering

The renderer projects tiles as isometric diamonds of 62×32 pixels. A diamond-shaped mask clips each tile.

The ground pass copies 32 rows without filtering. The tool draws RGB(0,0,0) pixels like any other pixel. `map atlas` defaults to opaque mode for `.gtl` files.

## Relationship to `.map` Files

Each tile in a `.map` file's Tile & Access block encodes a GTL index in bits 10–24 of the packed u32. The renderer blits
from
`gtl_base + index × 2048` (one 32×32 RGB565 tile).

## Related Files

| File    | Role                                                  |
|---------|-------------------------------------------------------|
| `*.btl` | Building tileset (same binary format, structure role) |
| `*.map` | Map files that reference GTL tile IDs                 |

## Implementation

- Parser: `extract()` in `src/map/tileset.rs`
- Atlas: `plot_tileset_map()` in `src/map/tileset.rs`

```bash
# Extract ground tiles to individual PNGs
cargo run -- map tiles "path/to/file.gtl" --output "out/gtl-tiles"

# Generate ground tileset atlas
cargo run -- map atlas "path/to/file.gtl" gtl-atlas.png
```
