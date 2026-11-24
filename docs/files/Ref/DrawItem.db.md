# DRAWITEM.ref

> DISPEL® is a registered trademark. This project is not affiliated with,
> endorsed by, or sponsored by the trademark owner.

## Purpose

`DRAWITEM.ref` stores item placements for maps.

## File Structure

- Encoding: EUC-KR.
- Format: parenthesized comma-separated text.
- Comment prefix: `;`.
- Each non-comment row has four fields.

```text
(<map_id>,<x_coord>,<y_coord>,<encoded_item>)
```

| Field          | Type  | Description            |
|----------------|-------|------------------------|
| `map_id`       | `i32` | Target map identifier. |
| `x_coord`      | `i32` | Map X coordinate.      |
| `y_coord`      | `i32` | Map Y coordinate.      |
| `encoded_item` | `i32` | Packed item reference. |

## Item Encoding

`encoded_item` is an `InventoryItem` value.

| Bytes | Meaning                                   |
|-------|-------------------------------------------|
| `1`   | Item ID.                                  |
| `2`   | Item type.                                |
| `3-4` | Preserved as part of the raw `i32` value. |

The parser keeps the complete encoded value in `DrawItem::item`. The editor uses `CompositeItem` to edit its item ID and
type together.

## Parser Behavior

The parser ignores empty rows, comment rows, malformed rows, and rows that do not have exactly four fields. It writes
normalized CRLF line endings.

The Rust parser is [draw_item.rs](../../../src/references/draw_item.rs).

## Legal Notice

This document describes a file format. It contains no game records, asset names, or other game-content data.
