# Save.ifo

> **DISPEL®** is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
> owner.

## Purpose

`Save.ifo` is the save-slot metadata index. It describes the six save slots (`0.sav` … `5.sav`): whether each slot is
occupied and when it was last written. It also snapshots a small amount of global state needed to resume the session
from the `game.tmp` append-log.

## File Structure

- **Location**: `Save.ifo` (game root directory)
- **Encoding**: binary, little-endian
- **Format**: fixed size of 224 bytes — six 32-byte slot records followed by a 32-byte global tail,
  addressed as two blocks: bytes 0–191 (slots) and 192–223 (tail).

### Slot records (offsets 0–191, record *p* at *p* × 32)

| Rel off | Type    | Meaning                                                                                              |
|---------|---------|------------------------------------------------------------------------------------------------------|
| +0      | u8[12]  | reserved — always zero in known files                                                                |
| +12     | u32     | save time: month                                                                                     |
| +16     | u32     | day                                                                                                  |
| +20     | u32     | hour                                                                                                 |
| +24     | u32     | minute                                                                                               |
| +28     | u8[4]   | flags: byte 0 = slot-occupied flag (1 = used), remaining 3 padding                                   |

### Global tail (offsets 192–223)

| Off | Type | Meaning                                                                                          |
|-----|------|--------------------------------------------------------------------------------------------------|
| 192 | f32  | game version — written as 1.4 before every save                                                  |
| 196 | u32  | `game.tmp` journal key identifying the current session's payload                                 |
| 200 | u32  | map/world id active at save time                                                                 |
| 204 | u32  | unknown; zero by default                                                                         |
| 208 | u32  | payload element count A — snapshot used as size multiplier when traversing the `game.tmp` log    |
| 212 | u32  | payload element count B (same role)                                                              |
| 216 | u32  | payload element count C (same role)                                                              |
| 220 | u32  | payload element count D (same role)                                                              |

## Usage

Written on every save alongside the `%d.sav` payload file. The save/load menu reads it at startup and lists each
occupied slot as `MM / DD HH : MM`. The tail snapshots the state needed to traverse the current session's payloads
inside the `game.tmp` append-log.

## Parser

The Rust parser is in `src/references/save_ifo.rs` (`SaveIfo` / `SaveSlotInfo` structs, `Extractor` trait).
