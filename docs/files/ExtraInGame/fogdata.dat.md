# ExtraInGame/fogdata.dat Documentation

## File Information

### Overview

The file holds brightness factors in a flat array. Each row covers one light level and has 512 bytes. The renderer reads a row for each shadowed tile.

The map lighting pass reads this file on maps flagged **Dark** in `AllMap.ini` (see `docs/rendering.md` and `src/map/render.rs::plot_shadows`).

### File Structure

**Location**: `ExtraInGame/fogdata.dat` (single file, shared by all maps)
**Encoding**: Binary (raw bytes, no header)
**Total Size**: 62,976 bytes = 123 rows × 512 bytes

### Format Specification

```
[Row 1]   – serves light level 1
- factor: 512 × u8 (one brightness factor per pixel pair)

[Row 2]   – serves light level 2
... (same structure) ...

[Row 123] – serves light level 123
```

| Property    | Value                    |
|-------------|--------------------------|
| Rows        | 123                      |
| Row length  | 512 bytes                |
| Total       | 62,976 bytes             |
| Header      | None                     |
| Endianness  | N/A (byte-granular data) |

### Level → Row Mapping

Row `L-1` serves light level `L`. A consumer indexes the file as:

```
byte[(level - 1) * 512 + pair]
```

Here `pair` is the pixel-pair index (`0..512`) within a shadowed tile. Each factor byte covers two horizontally adjacent pixels.

**Range:** light levels `1..=199` occur in the map's access-ref grid. The lighting pass skips levels of 200 or more. The file covers levels `1..=123`. Levels above 123 read past the table. Tooling must clamp or skip them.

### Factor Semantics

Each byte is a brightness factor `f` in `0..=31`. This is a 5-bit fixed-point value:

- Effective multiplier on shadowed pixels: `f / 32`.
- Applied to **all three colour channels**.
- `f = 0` blacks out the pixel pair. `f = 31` gives nearly full brightness.
- Values do not rise with level. Adjacent levels can have very different brightness at the same pair index.

### File Purpose

The tool reads per-level brightness patterns. It applies them to shadowed tiles on Dark maps.

### Rendering Dark Maps

The tool does not use the `.map` light-level bits (access-ref word bits 15 to 29) for rendering. It applies a fixed 500-byte viewport template per visible slot. A static render that uses the file's bits shows every dark map as black.

`map render --light-template <png> [--camera x,y]` applies that template before the fog pass. The template is a 500×1 grayscale PNG. `tools/convert_lighting.py` writes it. `make map-render-lit map_id=dun01` wraps the command.

### Cross-References

| Consumer            | References                                                        |
|---------------------|-------------------------------------------------------------------|
| Map lighting pass   | `src/map/render.rs` (`plot_shadows`, `FogData`)                   |
| Shadow levels       | `.map` access-ref grid bits 15–29 (`src/map/mod.rs::shadow_levels`) |
| Dark flag           | `AllMap.ini` (`lighting == Dark` per map)                         |

See also `docs/rendering.md` (isometric rendering pipeline) and
`docs/files/CROSS_REFERENCES.md`.

### Technical Details

- No header and no padding. The file holds exactly 62,976 raw bytes.
- Byte order does not apply because fields are single bytes.
- Parsed by `dispel_core::map::fogdata::FogData` (re-exported as
  `dispel_core::map::render::FogData`), which validates the length and provides
  bounds-checked accessors plus an editor-safe `set_factor` that rejects values > 31
  (out-of-range values wrap when consumed).

### Notes

- The tool does not look up level 0 tiles. It draws them black on Dark maps.
- When you edit the file, keep every byte in `0..=31`. Larger values are invalid in this table.
