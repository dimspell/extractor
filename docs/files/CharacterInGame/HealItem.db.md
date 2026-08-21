# HealItem.db

> DISPEL® is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
> owner.

## Purpose

`HealItem.db` stores fixed-size definitions for consumable items that restore health or mana and cure status effects.

## File Structure

- Header: one little-endian `i32` record count.
- Record size: 252 bytes.
- Name encoding: WINDOWS-1250.
- Description encoding: EUC-KR.

## Record Layout

| Offset | Size | Field                     | Type   | Description                    |
|--------|-----:|---------------------------|--------|--------------------------------|
| 0      |   30 | `name`                    | string | Null-padded item name.         |
| 30     |  202 | `description`             | string | Null-padded item description.  |
| 232    |    4 | `base_price`              | `i32`  | Item price.                    |
| 236    |    4 | `runtime_item_index_slot` | `i32`  | Loader-owned index slot.       |
| 240    |    2 | `health_points`           | `i16`  | Health-point change.           |
| 242    |    2 | `mana_points`             | `i16`  | Mana-point change.             |
| 244    |    1 | `restores_full_health`    | flag   | Full-health restore.           |
| 245    |    1 | `restores_full_mana`      | flag   | Full-mana restore.             |
| 246    |    1 | `cures_poison`            | flag   | Poison cure.                   |
| 247    |    1 | `cures_petrification`     | flag   | Petrification cure.            |
| 248    |    1 | `cures_polymorph`         | flag   | Polymorph cure.                |
| 249    |    3 | `reserved_trailer`        | bytes  | Preserved opaque data. Unused. |

## Runtime Behavior

The loader replaces `runtime_item_index_slot` with the sequential record index. Do not use this field as item data.

Each effect flag uses `HealItemFlag`. `0` disables the effect. `1` enables the effect.

## Parser

The Rust parser is [heal_item_db.rs](../../../src/references/heal_item_db.rs).

## Legal Notice

This document describes a file format. It contains no game records, asset names, or other game-content data.
