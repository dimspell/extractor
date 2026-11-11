# MiscItem.db

> DISPEL® is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
> owner.

## File Information

### Overview

Binary database file that defines generic miscellaneous items with names, descriptions, and economic values for the
game's crafting, inventory, and utility systems.

### File Structure

**Location**: `CharacterInGame/MiscItem.db`
**Encoding**: Binary (Little-Endian)
**Text Encodings**: Mixed (WINDOWS-1250 and EUC-KR)
**Header**: 4-byte record count **Record Size**: 256 bytes (64 × 4-byte fields)
**Total Records**: Variable (determined by header)

### Binary Format

```
[Header: 4 bytes]
- record_count: i32 (number of miscellaneous item entries)

[Records: 256 bytes each]
- name: 30 bytes (WINDOWS-1250, null-padded)
- description: 202 bytes (EUC-KR, null-padded)
- base_price: i32 (economic value)
- reserved_bytes: 16 bytes (preserved raw data)
- runtime_record_index_slot: i32 (overwritten with the sequential record index at load time)
```

### Field Definitions

| Field                     | Size | Type   | Description                                                                                 |
|---------------------------|------|--------|---------------------------------------------------------------------------------------------|
| id                        | N/A  | i32    | Record index (assigned during parsing)                                                      |
| name                      | 30   | string | Item name (WINDOWS-1250 encoded)                                                            |
| description               | 202  | string | Item description (EUC-KR encoded)                                                           |
| base_price                | 4    | i32    | Economic value (0 = non-tradable, -1 = quest item)                                          |
| reserved_bytes            | 16   | bytes  | Unused bytes                                                                                |
| runtime_record_index_slot | 4    | i32    | Slot at offsets 252–255. The game overwrites it in memory with the sequential record index. |

### Data Structure

The codebase defines the generic item structure as:

```rust
pub struct MiscItem {
    id: i32,                   // Record index
    name: String,              // Item name (30 chars max)
    description: String,       // Item description (202 chars max)
    base_price: i32,           // Economic value
    reserved_bytes: [u8; 16],
    runtime_record_index_slot: i32,
}
```

### Binary Record Layout

```
Offset | Size | Field | Description
-------|------|-------|-------------
0      | 30   | name  | Null-padded WINDOWS-1250 string
30     | 202  | desc  | Null-padded EUC-KR string
232    | 4    | price | Economic value (i32)
236    | 16   | reserved_bytes | Raw reserved bytes
252    | 4    | runtime_record_index_slot | Replaced with the record index in memory
```
