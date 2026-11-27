# Rendering logic

## Map rendering (`src/map/`)

The tool can render a full map into one image with `render_map`.

### Isometric projection

The map code converts map coordinates on a grid to image coordinates in pixels.

- Tile dimensions: 62x32 pixels.
- The tool uses this formula:

```rust
start_x = (x + y) * 32; // TILE_HORIZONTAL_OFFSET_HALF
start_y = (-x + y) * 16 + (diagonal / 2 * 16); // TILE_HEIGHT_HALF
```

### Layers

1. Base ground: The tool iterates diagonally and renders `gtl_tiles`. It applies a blue tint to event tiles and an optional tint to collision tiles. It uses a mask to draw the diamond tile shape.
2. Objects: The tool renders sprites and tiled objects. It sorts them by `ground_y` and draws them in that order. This order follows the painter's algorithm. Sprites come from `internal_sprites` in the map file. Tiled objects group tiles through `tiled_infos`.
3. Roofs: The tool renders `btl_tiles`.

## Sprite rendering (`src/sprite.rs`)

- The tool skips pixels with value `0`.
- Sprite position uses `origin_x` and `origin_y` relative to a calculated bounding box for animations.

## Tile rendering (`src/map/tileset.rs`)

`decode::tile_mask()` in `src/decode.rs` generates an offset array for each line `y`. The GUI also uses this mask. `mask[0]` gives the start x position. `mask[1]` gives the width. The array draws the diamond shape from the 32x32 source square.
