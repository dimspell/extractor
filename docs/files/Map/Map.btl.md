# cat1.btl - Dispel Game Building Tile Layer Format

## File Information
- **Location**: `Map/*.btl` files
- **Format**: Binary tileset (same as .GTL)
- **Tile Size**: 32×32 pixels
- **Color Format**: RGB565 (2 bytes per pixel)
- **Tile Data Size**: 2048 bytes per tile (32×32×2)

## File Structure

### No Header
- Direct sequence of tile data
- No metadata or header information
- Tiles are stored sequentially

### Tile Structure (2048 bytes)
- `pixel_data`: 1024 × RGB565 pixels (32×32 grid)
- Each pixel: 2 bytes in RGB565 format

### RGB565 Color Format
- **5 bits**: Red channel (0-31)
- **6 bits**: Green channel (0-63)
- **5 bits**: Blue channel (0-31)
- **Storage**: u16 format `0xRRRRRGGGGGGBBBBB`

## File Purpose

### .BTL Files (Building Tile Layer)
- Structures and buildings
- Roofs and architectural elements
- Man-made objects
- Decorative elements
- Walls, doors, windows
- Interior furniture and fixtures

## Isometric Properties
- **Rendered Width**: 62 pixels
- **Rendered Height**: 32 pixels
- **Shape**: Diamond-shaped mask for isometric projection
- **Transparency**: RGB(0,0,0) treated as transparent

## Color Conversion
The RGB565 format is converted to RGB888 during extraction:
- 5-bit red → 8-bit red (scaled 0-31 to 0-255)
- 6-bit green → 8-bit green (scaled 0-63 to 0-255)
- 5-bit blue → 8-bit blue (scaled 0-31 to 0-255)

## File Size Calculation
```
Total tiles = file_size / 2048
```

## Related Files
- `*.map` - Map files that reference these tilesets
- `*.gtl` - Ground tileset files

## Building Tile Files
- **Main maps**: `map1.btl`, `map2.btl`, `map3.btl`
- **Catacombs**: `cat1.btl`, `cat2.btl`, `cat3.btl`, `catp.btl`
- **Dungeons**: `dun01.btl` through `dun25.btl`, `final.btl`

## Implementation
- **Rust Module**: `src/map/tileset.rs`
- **Extractor**: `extract` function (same as .GTL)
- **Renderer**: `plot_tileset_map` function
- **Tile Structure**: `Tile` struct with color data

## Example Usage

### Extract building tiles to individual PNGs:
```bash
cargo run -- map tiles "fixtures/Dispel/Map/cat1.btl" --output "out/cat1-btl"
```

### Generate building tileset atlas:
```bash
cargo run -- map atlas "fixtures/Dispel/Map/cat1.btl" cat1.btl.png
```

### Render map using building tiles:
```bash
cargo run -- map render \
  --map "fixtures/Dispel/Map/cat1.map" \
  --btl "fixtures/Dispel/Map/cat1.btl" \
  --gtl "fixtures/Dispel/Map/cat1.gtl" \
  --output cat1.png
```

## Building Construction
Buildings in Dispel are constructed using stacked BTL tiles:
- Multiple tiles stacked vertically to create walls
- Different tile patterns for different building types
- Roof tiles placed on top of wall stacks
- Doors and windows as special tile types

## Diamond Mask
The `create_mask` function generates a diamond-shaped mask for proper isometric tile rendering, creating the characteristic diamond shape of isometric tiles.

## Transparency Handling
RGB(0,0,0) pixels are treated as transparent during rendering, allowing for proper layering of building tiles and creating complex structures.

## Comparison with GTL Files

| Feature        | BTL Files | GTL Files |
|--------------|----------|----------|
| **Purpose**   | Buildings, structures | Terrain, natural features |
| **Content**   | Walls, roofs, doors | Grass, water, paths |
| **Layer**     | Above ground | Ground level |
| **Usage**     | Stacked for 3D effect | Single layer |
| **Examples**  | Houses, castles | Grass, dirt, water |
