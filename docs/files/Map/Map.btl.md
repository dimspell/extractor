# DISPEL® Building Tile Layer (`.btl`)

The tool reads `.btl` files as raw tilesets for structures and roofs. The binary format matches `.gtl`.

## Quick Facts

| Property      | Value                           |
|---------------|---------------------------------|
| Location      | `Map/*.btl`                     |
| Header        | None                            |
| Tile size     | 32×32 px, RGB565                |
| Tile data     | 2048 bytes per tile (32×32 × 2) |
| Rendered size | 62×32 px isometric diamond      |
| Transparency  | per ref: opaque copy, or skip-zero blit |

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

The renderer picks a blit mode from the building overlay entry. Transparency 0 selects an opaque copy. A non-zero value selects a skip-zero blit, which copies only non-zero pixels. The draw-enable byte must be set before the renderer draws the cell. Only shadowed cells use this pass.

An atlas bakes in one mode. `map atlas` uses the skip-zero blit for `.btl` files. In `dun01`, 764 of 1419 references use opaque mode.

## Relationship to `.map` Files

A `.map` file references BTL tile IDs in two places:

- **Tiled Objects block**: building stacks drawn from anchor points
- **Access-Ref block**: occlusion and access lookup per tile

The tool stacks BTL tile IDs bottom-to-top. It skips negative IDs when rendering.

## Related Files

| File    | Role                                              |
|---------|---------------------------------------------------|
| `*.gtl` | Ground tileset (same binary format, terrain role) |
| `*.map` | Map files that reference BTL tile IDs             |

## Implementation

- Parser: `extract()` in `src/map/tileset.rs`
- Atlas: `plot_tileset_map()` in `src/map/tileset.rs`

```bash
# Extract building tiles to individual PNGs
cargo run -- map tiles "path/to/file.btl" --output "out/btl-tiles"

# Generate building tileset atlas
cargo run -- map atlas "path/to/file.btl" btl-atlas.png
```
