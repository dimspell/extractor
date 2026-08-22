# ExtraInGame/fogdata.dat Documentation

> DISPEL® is a registered trademark. This project is not affiliated with,
> endorsed by, or sponsored by the trademark owner.

## File Information

### Overview

Map-lighting fade tables. The file is a flat array of brightness factors used
to shade tiles that carry a shadow (darkness) level. Each row of the table is one
animated flicker pattern for a single light level; the renderer walks the row per rendered frame
to make torch-lit darkness shimmer.

This file feeds the map lighting pass, which only runs on maps flagged **Dark** in `AllMap.ini`
(see `docs/rendering.md` and `src/map/render.rs::plot_shadows`).

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

where `pair` is the pixel-pair index (`0..512`) within a shadowed tile — each factor byte covers
two horizontally adjacent pixels.

**Range nuance:** light levels `1..=199` occur in the map's access-ref grid (levels ≥ 200 are
skipped by the lighting pass entirely), but the file only covers levels `1..=123`.
Levels above 123 would read past the table; tooling must clamp or skip.

### Factor Semantics

Each byte is a brightness factor `f` in `0..=31` — a 5-bit fixed-point value:

- Effective multiplier on shadowed pixels: `f / 32`.
- Applied to the **red and green channels only**; blue stays untouched.
- `f = 0` → pixel pair blacked out; `f = 31` → nearly full brightness.
- Values are **not monotonic in level**: each row is an animated flicker pattern, so adjacent
  levels can have very different brightness at the same pair index.

### File Purpose

Provides the per-level darkness fade/flicker patterns that the map lighting pass applies to
shadowed tiles on Dark maps, producing an animated torch-light falloff.

### Cross-References

| Consumer            | References                                                        |
|---------------------|-------------------------------------------------------------------|
| Map lighting pass   | `src/map/render.rs` (`plot_shadows`, `FogData`)                   |
| Shadow levels       | `.map` access-ref grid bits 15–29 (`src/map/mod.rs::shadow_levels`) |
| Dark flag           | `AllMap.ini` (`lighting == Dark` per map)                         |

See also `docs/rendering.md` (isometric rendering pipeline) and
`docs/files/CROSS_REFERENCES.md`.

### Technical Details

- No header, no padding — exactly 62,976 raw bytes.
- Byte order is irrelevant (single-byte fields).
- Parsed by `dispel_core::map::fogdata::FogData` (re-exported as
  `dispel_core::map::render::FogData`), which validates the length and provides
  bounds-checked accessors plus an editor-safe `set_factor` that rejects values > 31
  (out-of-range values wrap when consumed).

### Notes

- Level 0 tiles are not looked up here: on Dark maps they are blacked out outright.
- When editing, keep every byte in `0..=31`; larger values are invalid in this table even though
  they wrap silently when consumed.
