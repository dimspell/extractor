//! Field-level patcher for `Save.ifo` (save-slot metadata index).
//!
//! Hand-written because the editable surface is the 32-byte global tail of a
//! single fixed-size record, addressed by dotted field paths
//! (`tail.game_version`, `tail.payload_counts.0`, …) rather than the flat
//! field names the derive macros generate.

use crate::modding::error::{ModdingError, Result};
use crate::modding::patcher::{RecordPatcher, unknown_field, wrong_type};
use crate::modding::value::Value;
use crate::references::save_ifo::SaveIfo;

pub struct SaveIfoPatcher;

impl SaveIfoPatcher {
    pub const FILENAME: &'static str = "Save.ifo";
    pub const RECORD_NAME: &'static str = "SaveIfo";
}

impl RecordPatcher for SaveIfoPatcher {
    fn name(&self) -> &'static str {
        Self::RECORD_NAME
    }

    fn apply_field(
        &self,
        bytes: &[u8],
        record_id: u32,
        field: &str,
        new: &Value,
    ) -> Result<Vec<u8>> {
        if record_id != 0 {
            return Err(ModdingError::Malformed(format!(
                "{}: record_id {record_id} out of range (single-record file)",
                Self::RECORD_NAME
            )));
        }

        let mut ifo = SaveIfo::parse(bytes)?;

        match field {
            "tail.game_version" => ifo.game_version = parse_f32(field, new)?,
            "tail.game_tmp_key" => ifo.game_tmp_key = parse_u32(field, new)?,
            "tail.map_id" => ifo.map_id = parse_u32(field, new)?,
            "tail.reserved" => ifo.reserved = parse_u32(field, new)?,
            other => {
                let index = other
                    .strip_prefix("tail.payload_counts.")
                    .and_then(|s| s.parse::<usize>().ok())
                    .filter(|&i| i < ifo.payload_counts.len())
                    .ok_or_else(|| unknown_field(Self::RECORD_NAME, other))?;
                ifo.payload_counts[index] = parse_u32(field, new)?;
            }
        }

        let mut out = Vec::with_capacity(bytes.len());
        ifo.write_to(&mut out)?;
        Ok(out)
    }
}

fn parse_f32(field: &str, new: &Value) -> Result<f32> {
    match new {
        Value::F64(v) => Ok(*v as f32),
        Value::String(s) => s
            .trim()
            .parse::<f32>()
            .map_err(|_| wrong_type(SaveIfoPatcher::RECORD_NAME, field, "f32", new)),
        _ => Err(wrong_type(SaveIfoPatcher::RECORD_NAME, field, "f32", new)),
    }
}

fn parse_u32(field: &str, new: &Value) -> Result<u32> {
    match new {
        Value::I64(v) => u32::try_from(*v)
            .map_err(|_| wrong_type(SaveIfoPatcher::RECORD_NAME, field, "u32", new)),
        Value::String(s) => s
            .trim()
            .parse::<u32>()
            .map_err(|_| wrong_type(SaveIfoPatcher::RECORD_NAME, field, "u32", new)),
        _ => Err(wrong_type(SaveIfoPatcher::RECORD_NAME, field, "u32", new)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::save_ifo::{SLOT_COUNT, SaveSlotInfo};

    fn sample_bytes() -> Vec<u8> {
        let ifo = SaveIfo {
            slots: vec![SaveSlotInfo::default(); SLOT_COUNT],
            game_version: 1.4,
            game_tmp_key: 6,
            map_id: 12,
            reserved: 0,
            payload_counts: [329, 349, 0, 200],
        };
        let mut out = Vec::new();
        ifo.write_to(&mut out).unwrap();
        out
    }

    fn apply(bytes: &[u8], field: &str, value: Value) -> Vec<u8> {
        SaveIfoPatcher.apply_field(bytes, 0, field, &value).unwrap()
    }

    #[test]
    fn patches_each_tail_field() {
        let original = sample_bytes();

        let patched = apply(&original, "tail.map_id", Value::String("13".into()));
        let ifo = SaveIfo::parse(&patched).unwrap();
        assert_eq!(ifo.map_id, 13);
        assert_eq!(ifo.game_tmp_key, 6); // untouched

        let patched = apply(&original, "tail.game_version", Value::F64(1.5));
        let ifo = SaveIfo::parse(&patched).unwrap();
        assert!((ifo.game_version - 1.5).abs() < f32::EPSILON);

        let patched = apply(&original, "tail.payload_counts.2", Value::I64(42));
        let ifo = SaveIfo::parse(&patched).unwrap();
        assert_eq!(ifo.payload_counts, [329, 349, 42, 200]);
    }

    #[test]
    fn rejects_unknown_fields_and_bad_ids() {
        let bytes = sample_bytes();
        assert!(
            SaveIfoPatcher
                .apply_field(&bytes, 0, "slots", &Value::Null)
                .is_err()
        );
        assert!(
            SaveIfoPatcher
                .apply_field(&bytes, 0, "tail.nope", &Value::Null)
                .is_err()
        );
        assert!(
            SaveIfoPatcher
                .apply_field(&bytes, 0, "tail.payload_counts.9", &Value::I64(1))
                .is_err()
        );
        assert!(
            SaveIfoPatcher
                .apply_field(&bytes, 1, "tail.map_id", &Value::I64(1))
                .is_err()
        );
        assert!(
            SaveIfoPatcher
                .apply_field(&bytes, 0, "tail.map_id", &Value::Bool(true))
                .is_err()
        );
    }
}
