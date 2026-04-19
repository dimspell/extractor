# ChData.db - Character Statistics

## File Information
- **Location**: `CharacterInGame/ChData.db`
- **Format**: Binary (Little-Endian)
- **Record Size**: 84 bytes
- **Single-record file**: Contains one comprehensive record

## File Structure

### Header (30 bytes)
- `magic`: 4 bytes - File signature ("Item" ASCII)
- `padding`: 26 bytes - Zero-padding for alignment

### Data Section (54 bytes)
- `values`: 16 × u16 (32 bytes) - Statistical values
- `padding`: 2 bytes - Alignment padding
- `counts`: 4 × u32 (16 bytes) - Count statistics
- `total`: u32 (4 bytes) - Total aggregate value

## Field Details

### magic
- File signature "Item"
- Identifies file type
- Validation marker
- Always 4 bytes ASCII

### values (16 × u16)
- Array of 16 unsigned 16-bit values
- Statistical counters and metrics
- Game state tracking
- Character performance data

### counts (4 × u32)
- Array of 4 unsigned 32-bit counters
- Aggregate statistics
- Event counts
- Achievement tracking

### total (u32)
- Single aggregate total
- Sum of important metrics
- Overall progress indicator
- Final calculation result

## Implementation
- **Rust Module**: `src/references/chdata_db.rs`
- **Extractor**: `ChData` struct implementing `Extractor` trait
- **Data Structure**: `ChData` with comprehensive statistics
- **Single Record**: File contains exactly one record

## Example Usage

### Extract and display character data:
```bash
cargo run -- extract -i "fixtures/Dispel/CharacterInGame/ChData.db"
```

### Format Structure
```
Bytes 0-3:    "Item" signature
Bytes 4-29:   26 bytes padding
Bytes 30-61:  16 × u16 values
Bytes 62-63:  2 bytes padding
Bytes 64-79:  4 × u32 counts
Bytes 80-83:  u32 total
```

## File Layout Visualization

```
+-------------------------------+
| ChData.db File Structure      |
+-------------------------------+
| Bytes 0-3:    "Item"         |
| Bytes 4-29:   Padding          |
| Bytes 30-61:  16 × u16 values  |
| Bytes 62-63:  Padding          |
| Bytes 64-79:  4 × u32 counts   |
| Bytes 80-83:  u32 total         |
+-------------------------------+
| Total Size: 84 bytes           |
+-------------------------------+
```

## Binary Structure Details

### Byte Offsets
- `0x00-0x03`: Magic signature
- `0x04-0x1D`: Padding (26 bytes)
- `0x1E-0x3D`: Values array (32 bytes)
- `0x3E-0x3F`: Padding (2 bytes)
- `0x40-0x4F`: Counts array (16 bytes)
- `0x50-0x53`: Total value (4 bytes)

### Data Types
- `magic`: [u8; 4] ASCII
- `values`: [u16; 16]
- `counts`: [u32; 4]
- `total`: u32

### Endianness
- All numeric values: Little-Endian
- Consistent byte order
- Standard x86 format

## Validation
- Magic signature verification
- File size check (must be 84 bytes)
- Data integrity validation
- Range checking for values

## Extractor

An extractor is available in `src/references/chdata_db.rs` to parse this file format.

### How to Run

```bash
# Extract ChData.db to JSON
cargo run -- extract -i "fixtures/Dispel/CharacterInGame/ChData.db"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
