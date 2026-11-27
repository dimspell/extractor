# Save.ifo

## Purpose

`Save.ifo` is the save-slot metadata index. It describes six save slots (`0.sav` … `5.sav`). It records a timestamp and an occupied flag for each slot. It also holds a small global tail.

## File structure

- **Location**: `Save.ifo` (game root directory)
- **Encoding**: binary, little-endian
- **Format**: fixed size of 224 bytes with six 32-byte slot records followed by a 32-byte global tail. The tool addresses two blocks: bytes 0–191 for slots and 192–223 for the tail.

### Slot records (offsets 0–191, record *p* at *p* × 32)

| Rel off | Type   | Meaning                                                            |
|---------|--------|--------------------------------------------------------------------|
| +0      | u8[12] | reserved — always zero in known files                              |
| +12     | u32    | save time: month                                                   |
| +16     | u32    | day                                                                |
| +20     | u32    | hour                                                               |
| +24     | u32    | minute                                                             |
| +28     | u8[4]  | flags: byte 0 = slot-occupied flag (1 = used), remaining 3 padding |

### Global tail (offsets 192–223)

| Off | Type | Meaning                                                                                       |
|-----|------|-----------------------------------------------------------------------------------------------|
| 192 | f32  | game version — written as 1.4 before every save                                               |
| 196 | u32  | `game.tmp` journal key identifying the current session's payload                              |
| 200 | u32  | map/world id active at save time                                                              |
| 204 | u32  | reserved; zero in known files                                                                 |
| 208 | u32  | payload element count A — snapshot used as size multiplier when traversing the `game.tmp` log |
| 212 | u32  | payload element count B (same role)                                                           |
| 216 | u32  | payload element count C (same role)                                                           |
| 220 | u32  | payload element count D (same role)                                                           |

## Usage

The tool reads and writes this file alongside the `%d.sav` payload files. Each occupied slot carries month, day, hour and minute values. The documented display format is `MM / DD HH : MM`. The tool preserves the tail values.

## Parser

The Rust parser is in `src/references/save_ifo.rs`. It defines the `SaveIfo` and `SaveSlotInfo` structs and implements the `Extractor` trait.
