# File Formats & Parsing

This document details the binary formats parsed by the extractor. This information is critical for re-implementation or generating parsers.

## Common Data Types
- **LittleEndian Integers**: Checking `read_i32::<LittleEndian>()` or `read_i16` confirms data is stored in Little Endian.
- **Color Format (RGB 565)**:
    - 16-bit integer.
    - **Red**: 5 bits (Mask `0xF800` >> 11)
    - **Green**: 6 bits (Mask `0x07E0` >> 5)
    - **Blue**: 5 bits (Mask `0x001F`)
    - Expansion to 8-bit: `val << 3` (for R/B) or `val << 2` (for G).

---

## 1. SNF Audio (`snf.rs`)

The SNF format appears to be a raw PCM container very similar to WAV but with a custom header.

### Header Structure
| Byte Offset | Type | Description |
|---|---|---|
| 0 | i32 | Data Size |
| 4 | i16 | Audio Format (PCM) |
| 6 | i16 | Num Channels |
| 8 | i32 | Sample Rate |
| 12 | i32 | Byte Rate |
| 16 | i16 | Block Align |
| 18 | i16 | Bits Per Sample |
| 20 | i16 | *Unknown / Padding* (skipped) |

### Conversion
The extractor writes a standard RIFF WAVE header followed by the raw PCM data read from the SNF file.

---

## 2. Map Format (`map.rs`)

Map files are complex binary structures containing multiple blocks of data.

### 2.1 Header / Model
- `Width` (i32)
- `Height` (i32)
- Derived "Tiled Map Size": `Width * 25 - 1` by `Height * 25 - 1` (implying 25x25 chunks).

### 2.2 Data Blocks
The file is read sequentially.

#### Block 1 (Unknown)
- `Multiplier` (i32)
- `Size` (i32)
- Skips 8 bytes.
- Skips `Multiplier * Size * 4` bytes.

#### Block 2 (Unknown)
- `Size` (i32)
- Skips `Size * 2` bytes.

#### Internal Sprites Block
- `Count` (i32)
- For each sprite:
    - `Image Stamp` (i32): Determines a static offset (1904 if 6, 2996 if 9).
    - Skip 264 bytes.
    - **Sequence Info**: Parsed (see Sprite Format).
    - Seek past image data based on the static offset logic.

#### Sprite Position Block
- `Count` (i32)
- For each entry:
    - `Sprite ID` (i32): Index into internal sprites.
    - Skip 2x i32.
    - Skip 2x i32 (bottom-right coords?).
    - `Sprite X` (i32)
    - `Sprite Y` (i32)
    - Skip calculated based on frames in the referenced sprite sequence.

#### Tiled Objects Block
- `Count` (i32)
- `Number1` (i32)
- For each object:
    - Skip 264 bytes.
    - Read 4x i32 (s8, s0_1, s1, s0_2)
    - Read 4x i32 (v1..v4)
    - `X`, `Y` (i32)
    - Read 2x i32 (v7, v8)
    - `C1`, `C2`, `C3` (i32)
    - Read `C3` count of `i16` (IDs).
    - Skip 84 bytes.
    - Skip `(C1 + C2 + C3) * 4` bytes.
- **Correction Logic**: Contains a specific seek-back-and-scan logic to handle a variable length footer or padding (looking for byte `1`).

### 2.3 Tile & Event Layers
The file reading jumps to the END of the file and seeks backwards to find tile data.

- **Seek**: `-(TiledHeight * TiledWidth * 4 * 3)` from End.
- **Event Block**: `TiledWidth * TiledHeight` entries of:
    - `Event ID` (i16)
    - `Unknown` (i16)
- **GTL (Ground Tile Layer) / Collision**: `TiledWidth * TiledHeight` entries of:
    - `Value` (i32)
    - `ID` = `Value >> 10`
    - `Collision` = `(Value & 1) == 1`
- **BTL (Building/Roof Tile Layer)**: `TiledWidth * TiledHeight` entries of:
    - `Tile ID` (i16)
    - `Flag` (i16)

---

## 3. Tileset Format (`tileset.rs`)

Simple contiguous array of raw pixels.
- **File Length** is used to determine tile count.
- **Tile Size**: 32x32 pixels.
- **Pixel Format**: u16 (RGB 565).
- Loop until EOF: Read 32*32*2 bytes.

### Isometry Mask
The code procedurally generates a diamond-shape mask (`create_mask`) to render the 32x32 square tile into a 62x32 isometric projection.

---

## 4. Sprite / Animation Format (`sprite.rs`)

Starts at byte **268**.

### Sequence Loop
- Checks if file has enough bytes left.
- **Validation**: Reads 15x `i32`. Checks specific values to determine if it's a valid sequence header (e.g., `ints[11] * ints[12] == ints[13]`).
- If valid:
    - Read `Stamp` (i32).
    - If Stamp == 0, read `FrameCount` (i32).
    - **Frame Loop**:
        - Skip 24 bytes (6 * 4).
        - Read `Origin X`, `Origin Y`, `Width`, `Height` (i32).
        - Read `Size` (u32).
        - Actual Data Size = `Size * 2`.
        - **Image Data**: u16 pixels (RGB 565). `Width * Height`.
