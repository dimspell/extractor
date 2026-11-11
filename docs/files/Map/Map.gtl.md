# DISPEL® Ground Tile Layer (`.gtl`)

Raw tileset binary used for terrain, paths, and natural features. The format is identical to `.btl`; only the semantic
role differs.

> DISPEL® is a registered trademark. This project is not affiliated with,
> endorsed by, or sponsored by the trademark owner.

## Quick Facts

| Property      | Value                           |
|---------------|---------------------------------|
| Location      | `Map/*.gtl`                     |
| Header        | None                            |
| Tile size     | 32×32 px, RGB565                |
| Tile data     | 2048 bytes per tile (32×32 × 2) |
| Rendered size | 62×32 px isometric diamond      |
| Transparency  | RGB(0,0,0) = transparent        |

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

Tiles are projected as isometric diamonds (62×32 px). A diamond-shaped mask clips each tile so that corners blend
seamlessly. RGB (0,0,0) pixels are treated as transparent, allowing terrain tiles to layer with sprites and building
tiles.

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
