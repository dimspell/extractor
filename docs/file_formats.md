# File formats and parsing

This document records the binary layouts that the extractor reads.

## Common data types

The tool reads all integers as little-endian values.

Colors use a 16-bit RGB 565 value:

- Red takes 5 bits from mask `0xF800` after a shift of 11 places.
- Green takes 6 bits from mask `0x07E0` after a shift of 5 places.
- Blue takes 5 bits from mask `0x001F`.
- Expansion to 8-bit values uses `val << 3` for red and blue, and `val << 2` for green.

## 1. SNF audio (`snf.rs`)

The tool reads SNF files as PCM data with a 22-byte header.

### Header structure

| Byte Offset | Type | Description |
|---|---|---|
| 0 | i32 | Data Size |
| 4 | i16 | Audio Format (PCM) |
| 6 | i16 | Num Channels |
| 8 | i32 | Sample Rate |
| 12 | i32 | Byte Rate |
| 16 | i16 | Block Align |
| 18 | i16 | Bits Per Sample |
| 20 | i16 | Unknown value that the tool skips |

### Conversion

The tool writes a standard RIFF WAVE header. It then appends the raw PCM data from the SNF file.

## 2. Map format (`map.rs`)

Map files hold several blocks of binary data.

### 2.1 Header and model

- `Width` (i32)
- `Height` (i32)
- The tool derives the tiled map size as `Width * 25 - 1` by `Height * 25 - 1`. The formula uses 25x25 chunks.

### 2.2 Data blocks

The tool reads these blocks in sequence.

#### Block 1

- `Multiplier` (i32)
- `Size` (i32)
- The tool skips 8 bytes.
- The tool skips `Multiplier * Size * 4` bytes.

#### Block 2

- `Size` (i32)
- The tool skips `Size * 2` bytes.

#### Internal sprites block

- `Count` (i32)
- For each sprite, the tool reads `Image Stamp` (i32). It uses a static offset of 1904 for stamp 6 and 2996 for stamp 9.
- The tool skips 264 bytes.
- The tool parses the sequence information (see Sprite format).
- The tool seeks past the image data using that static offset.

#### Sprite position block

- `Count` (i32)
- `Sprite ID` (i32) indexes the internal sprites.
- The tool skips two `i32` values, then two more `i32` values.
- `Sprite X` (i32)
- `Sprite Y` (i32)
- The tool skips a length based on the frame count in the referenced sprite sequence.

#### Tiled objects block

- `Count` (i32)
- `Number1` (i32)
- For each object, the tool skips 264 bytes.
- It reads four `i32` values: `s8`, `s0_1`, `s1` and `s0_2`.
- It reads four `i32` values: `v1` through `v4`.
- It reads `X` and `Y` (i32).
- It reads two `i32` values: `v7` and `v8`.
- It reads `C1`, `C2` and `C3` (i32).
- It reads `C3` values of type `i16` as IDs.
- It skips 84 bytes.
- It skips `(C1 + C2 + C3) * 4` bytes.
- The tool seeks back and scans for byte `1` to handle a variable-length footer or padding.

### 2.3 Tile and event layers

The tool jumps to the end of the file. It seeks backwards to find the tile data.

- Seek: `-(TiledHeight * TiledWidth * 4 * 3)` from End.
- Event block: `TiledWidth * TiledHeight` entries, each with `Event ID` (i16) and `Unknown` (i16).
- GTL (ground tile layer) and collision: `TiledWidth * TiledHeight` entries, each with `Value` (i32). The tool computes `ID = Value >> 10` and `Collision = (Value & 1) == 1`.
- BTL (building and roof tile layer): `TiledWidth * TiledHeight` entries, each with `Tile ID` (i16) and `Flag` (i16).

## 3. Tileset format (`tileset.rs`)

A tileset file holds a contiguous array of raw pixels.

- The file length determines the tile count.
- Each tile is 32x32 pixels.
- Each pixel is a u16 RGB 565 value.
- The tool reads 32*32*2 bytes at a time until it reaches EOF.

### Isometry mask

The tool generates a diamond-shape mask with `decode::tile_mask()` in `src/decode.rs`. The mask draws the 32x32 square tile as a 62x32 isometric projection.

## 4. Sprite and animation format (`sprite.rs`)

Sprite data starts at byte 268.

### Sequence loop

- The tool checks that enough bytes remain in the file.
- The tool reads 15 `i32` values. It treats the header as a valid sequence when `ints[11] * ints[12] == ints[13]`.
- For a valid sequence, the tool reads `Stamp` (i32).
- When `Stamp` is 0, the tool reads `FrameCount` (i32).

### Frame loop

- The tool skips 24 bytes (6 * 4).
- It reads `Origin X`, `Origin Y`, `Width` and `Height` (i32).
- It reads `Size` (u32).
- The actual data size is `Size * 2`.
- The tool reads `Width * Height` pixels of type u16 in RGB 565 format.
