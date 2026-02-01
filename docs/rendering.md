# Rendering Logic

## Map Rendering (`map.rs`)
The tool can render a full map into a single image (`render_map`).

### Isometric Projection
The map logic handles conversion between "Map Coords" (grid) and "Image Coords" (pixel).

- **Tile Dimensions**: 62x32 pixels.
- **Formula**:
    ```rust
    start_x = (x + y) * 32; // TILE_HORIZONTAL_OFFSET_HALF
    start_y = (-x + y) * 16 + (diagonal / 2 * 16); // TILE_HEIGHT_HALF
    ```

### Layers
1.  **Base (Ground)**: Iterates diagonally. Renders `gtl_tiles`.
    - Applies color mixing for Event tiles (Blue tint) or Collision tiles (optional).
    - Uses a mask to draw the diamond tile shape.
2.  **Objects**: Sprites and Tiled Objects.
    - Sorted by `ground_y` coordinate to handle depth/occlusion correctly (painter's algorithm).
    - **Sprites**: Rendered from `internal_sprites` extracted from the map file.
    - **Tiled Objects**: Collections of tiles designated by `tiled_infos` (multi-tile structures).
3.  **Roofs (BTL)**: Renders `btl_tiles`.

## Sprite Rendering (`sprite.rs`)
- **Transparency**: Pixels with value `0` are skipped/transparent.
- **Positioning**: Uses `origin_x`, `origin_y` relative to a calculated bounding box for animations.

## Tile Rendering (`tileset.rs`)
- **Masking**: Since the source data is a 32x32 square but the output is isometric, `create_mask` generates an offset array (`mask[0]` for start x, `mask[1]` for width) per line `y` to draw a diamond shape.
