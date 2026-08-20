# HealItem.db

## Purpose

`HealItem.db` stores fixed-size definitions for consumable items that restore health or mana and cure status effects.

## File Structure

- Header: one little-endian `i32` record count.
- Record size: 252 bytes.
- Name encoding: WINDOWS-1250.
- Description encoding: EUC-KR.

## Record Layout

| Offset | Size | Field | Type | Description |
|---|---:|---|---|---|
| `0x00` | 30 | `name` | string | Null-padded item name. |
| `0x1E` | 202 | `description` | string | Null-padded item description. |
| `0xE8` | 4 | `base_price` | `i32` | Item price. |
| `0xEC` | 4 | `runtime_item_index_slot` | `i32` | Loader-owned index slot. |
| `0xF0` | 2 | `health_points` | `i16` | Health-point change. |
| `0xF2` | 2 | `mana_points` | `i16` | Mana-point change. |
| `0xF4` | 1 | `restores_full_health` | flag | Full-health restore. |
| `0xF5` | 1 | `restores_full_mana` | flag | Full-mana restore. |
| `0xF6` | 1 | `cures_poison` | flag | Poison cure. |
| `0xF7` | 1 | `cures_petrification` | flag | Petrification cure. |
| `0xF8` | 1 | `cures_polymorph` | flag | Polymorph cure. |
| `0xF9` | 3 | `reserved_trailer` | bytes | Preserved opaque data. Unused. |

## Runtime Behavior

The loader replaces `runtime_item_index_slot` with the sequential record index. Do not use this field as item data.

Each effect flag uses `HealItemFlag`. `0` disables the effect. `1` enables the effect.

## Parser

The Rust parser is [heal_item_db.rs](../../../src/references/heal_item_db.rs).

## Legal Notice

This document describes a file format. It contains no game records, asset names, or other game-content data.
