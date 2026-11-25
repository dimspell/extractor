# Wave.ini

## Purpose

`Wave.ini` maps a sound identifier to an SNF filename and a simultaneous-playback limit.

## File structure

- **Location**: `Wave.ini`
- **Encoding**: EUC-KR
- **Format**: CSV (Comma-Separated Values)
- Comment prefix: `;`.
- Each non-comment row has three fields.

```text
<id>,<snf_filename>,<max_simultaneous_plays>
```

| Field                    | Type             | Description                                 |
|--------------------------|------------------|---------------------------------------------|
| `id`                     | `i32`            | Sound identifier.                           |
| `snf_filename`           | string or `null` | Referenced SNF filename.                    |
| `max_simultaneous_plays` | `i32`            | Maximum concurrent instances of this sound. |

## Field values

A value of `1` records one concurrent instance. Larger values record more concurrent instances. The field is not a priority, channel or loop flag.

## Parser

The Rust parser is in `src/references/wave_ini.rs`. It defines the `WaveIni` struct and implements the `Extractor` trait.
