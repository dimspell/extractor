# Wave.ini

## Purpose

`Wave.ini` maps a sound identifier to an SNF filename and a simultaneous-playback limit.

## File Structure

- Encoding: EUC-KR.
- Format: comma-separated text.
- Comment prefix: `;`.
- Each non-comment row has three fields.

```text
<id>,<snf_filename>,<max_simultaneous_plays>
```

| Field | Type | Description |
|---|---|---|
| `id` | `i32` | Sound identifier. |
| `snf_filename` | string or `null` | Referenced SNF filename. |
| `max_simultaneous_plays` | `i32` | Maximum concurrent instances of this sound. |

## Runtime Behavior

The loader creates `max_simultaneous_plays` audio-buffer copies for each entry. Playback uses a free copy.

A limit of one prevents a sound from overlapping with itself. Larger limits allow concurrent playback.

This value is not a priority, channel, or loop flag.

## Parser

The Rust parser is [wave_ini.rs](../../src/references/wave_ini.rs).

## Legal Notice

This document describes a file format. It contains no game records, asset names, or other game-content data.

**DISPEL®** is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark owner.
