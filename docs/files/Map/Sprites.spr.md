# M_BODY1.spr - Dispel Sprite File Format

## File Information
- **Location**: `CharacterInGame/*.spr` and `NpcInGame/*.spr` files
- **Format**: Binary (Little-Endian)
- **Color Format**: RGB565 (2 bytes per pixel)
- **Header Offset**: 268 bytes

## File Structure

### File Header (268 bytes)
- 268 bytes of unknown data
- No documented structure
- Skipped during parsing

### Sequence Structure
Each sprite file contains one or more animation sequences:

#### Sequence Header
- `stamp`: i32 - Sequence marker (8 or 0)
- `frame_count`: i32 - Number of frames in sequence
- 2 × i32 unknown data

#### Frame Structure (per frame)
- 6 × i32 unknown metadata
- `origin_x`: i32 - X offset from origin point
- `origin_y`: i32 - Y offset from origin point
- `width`: i32 - Frame width in pixels
- `height`: i32 - Frame height in pixels
- `size_bytes`: u32 - Pixel data size in bytes
- `pixel_data`: RGB565 pixels (width × height × 2 bytes)

## Color Format

### RGB565 Format
- **5 bits**: Red channel (0-31)
- **6 bits**: Green channel (0-63)
- **5 bits**: Blue channel (0-31)
- **Storage**: u16 format `0xRRRRRGGGGGGBBBBB`
- **Transparency**: `0x0000` (black) is treated as transparent

### Color Conversion
RGB565 is converted to RGB888 during extraction:
- 5-bit red → 8-bit red (scaled 0-31 to 0-255)
- 6-bit green → 8-bit green (scaled 0-63 to 0-255)
- 5-bit blue → 8-bit blue (scaled 0-31 to 0-255)

## Sequence Detection
The parser uses a heuristic to detect valid sprite sequences:
- Looks for patterns in 15 consecutive i32 values
- Valid sequence patterns:
  - `(0, frame_count>0, 0, ..., width>0, height>0, width×height==size)`
  - `(8, 0, frame_count>0, 0, ..., width>0, height>0, width×height==size)`

## File Purpose
Stores character sprites, animations, and visual effects for:
- NPCs and monsters
- Party members
- Special effects
- Interactive objects

## Sprite Types

### Character Sprites
- `M_BODY1.spr` through `M_BODY8.spr` - Male body types
- `F_BODY1.spr` through `F_BODY8.spr` - Female body types
- `Party1.spr` through `Party8.spr` - Party member sprites
- `guard1.spr`, `guard2.spr`, `guard3.spr` - Guard NPCs
- `King1.spr`, `King2.spr`, `King3.spr` - Royal NPCs

### Monster Sprites
- `ANIMAL1.spr` through `ANIMAL8.spr` - Animals
- `GUARD1.spr` through `GUARD3.spr` - Guard monsters
- `KNIGHTS.spr` - Knight monsters
- `MAGE1.spr` through `MAGE3.spr` - Mage monsters
- `PEOPLE1.spr` through `PEOPLE13.spr` - Human-like monsters
- `Spooky.spr` - Ghostly/spectral monsters

### Special Sprites
- `Abel.spr`, `Kasra.spr`, `Lonin.spr`, `Mace.spr` - Named characters
- `PopeNpc.spr`, `PopeNpc2.spr` - Religious figures
- `traveller1.spr` through `traveller3.spr` - Traveler NPCs
- `VENDER1.spr` through `VENDER6.spr` - Merchant NPCs

## Implementation
- **Rust Module**: `src/sprite.rs`
- **Extractor**: `extract` function for individual frames
- **Animation**: `animation` function for animated sequences
- **Data Structures**:
  - `SequenceInfo` - Sequence metadata
  - `ImageInfo` - Frame metadata
  - `Color` - RGB color structure

## Example Usage

### Extract individual frames:
```bash
cargo run -- sprite "fixtures/Dispel/CharacterInGame/M_BODY1.spr" "M_BODY1"
```

### Extract as animation:
```bash
cargo run -- sprite "fixtures/Dispel/CharacterInGame/M_BODY1.spr" --mode animation
```

## Animation Sequences
Sprites can contain multiple animation sequences:
- Walking cycles
- Attack animations
- Special ability effects
- Idle animations
- Death sequences

## Origin Points
Each frame has an origin point (`origin_x`, `origin_y`) that defines:
- The reference point for positioning
- Rotation center
- Collision detection point

## Frame Dimensions
Frames can have variable dimensions:
- Typical sizes: 64×64, 128×128, 256×256 pixels
- Larger frames for complex animations
- Smaller frames for simple objects

## Related Files
- `Npc.ini` - NPC definitions referencing sprites
- `Monster.ini` - Monster definitions referencing sprites
- `Extra.ini` - Object definitions referencing sprites
- `*.map` files - Map files containing embedded sprites

## Extractor

An extractor is available in `src/sprite.rs` to parse this file format.

### How to Run

```bash
# Extract individual frames
cargo run -- sprite "fixtures/Dispel/CharacterInGame/M_BODY1.spr" "M_BODY1"

# Extract as animation
cargo run -- sprite "fixtures/Dispel/CharacterInGame/M_BODY1.spr" --mode animation
```
