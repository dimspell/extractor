//! Field-level patcher for `CharacterInGame/ChData.db` — a fixed
//! single-record binary blob (84 bytes) holding global character
//! statistics: a magic signature, 16 × u16 values, 4 × u32 counts,
//! and a u32 total.
//!
//! Hand-written because [`ChData`] has two `Vec<_>` fields with fixed
//! lengths. Element-level access is exposed via dotted field names:
//!
//! ```text
//! values.0 .. values.15      // u16
//! counts.0 .. counts.3       // u32
//! magic, total               // String, u32
//! ```
//!
//! The file holds exactly one record, so `record_id` must be 0.

use std::io::Cursor;

use crate::modding::error::{ModdingError, Result};
use crate::modding::patcher::{out_of_range, unknown_field, wrong_type, RecordPatcher};
use crate::modding::value::Value;
use crate::references::chdata_db::ChData;
use crate::references::extractor::Extractor;

pub struct ChDataPatcher;

impl ChDataPatcher {
    pub const FILENAME: &'static str = "ChData.db";
    pub const RECORD_NAME: &'static str = "ChData";
}

impl RecordPatcher for ChDataPatcher {
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
        let mut cursor = Cursor::new(bytes);
        let mut records = ChData::parse(&mut cursor, bytes.len() as u64)?;
        if record_id != 0 || records.is_empty() {
            return Err(out_of_range(Self::RECORD_NAME, record_id, records.len()));
        }
        let rec = &mut records[0];

        match field {
            "magic" => match new {
                Value::String(s) => rec.magic = s.clone(),
                _ => return Err(wrong_type(Self::RECORD_NAME, field, "string", new)),
            },
            "total" => rec.total = parse_u32(field, new)?,
            other => {
                if let Some(idx) = parse_indexed(other, "values.") {
                    if idx >= rec.values.len() {
                        return Err(ModdingError::Malformed(format!(
                            "{}.values.{} out of range (have {})",
                            Self::RECORD_NAME,
                            idx,
                            rec.values.len()
                        )));
                    }
                    rec.values[idx] = parse_u16(other, new)?;
                } else if let Some(idx) = parse_indexed(other, "counts.") {
                    if idx >= rec.counts.len() {
                        return Err(ModdingError::Malformed(format!(
                            "{}.counts.{} out of range (have {})",
                            Self::RECORD_NAME,
                            idx,
                            rec.counts.len()
                        )));
                    }
                    rec.counts[idx] = parse_u32(other, new)?;
                } else if other == "values" || other == "counts" {
                    return Err(ModdingError::Malformed(format!(
                        "{}.{} is a vector; address an element via `{}.<index>`",
                        Self::RECORD_NAME,
                        other,
                        other
                    )));
                } else {
                    return Err(unknown_field(Self::RECORD_NAME, other));
                }
            }
        }

        let mut out = Vec::with_capacity(bytes.len());
        ChData::to_writer(&records, &mut out)?;
        Ok(out)
    }
}

/// `field` like `"values.7"` returns `Some(7)` when prefixed with `prefix`.
fn parse_indexed(field: &str, prefix: &str) -> Option<usize> {
    field.strip_prefix(prefix).and_then(|s| s.parse().ok())
}

fn parse_u16(field: &str, new: &Value) -> Result<u16> {
    match new {
        Value::I64(v) => {
            u16::try_from(*v).map_err(|_| wrong_type(ChDataPatcher::RECORD_NAME, field, "u16", new))
        }
        Value::String(s) => s
            .trim()
            .parse::<u16>()
            .map_err(|_| wrong_type(ChDataPatcher::RECORD_NAME, field, "u16", new)),
        _ => Err(wrong_type(ChDataPatcher::RECORD_NAME, field, "u16", new)),
    }
}

fn parse_u32(field: &str, new: &Value) -> Result<u32> {
    match new {
        Value::I64(v) => {
            u32::try_from(*v).map_err(|_| wrong_type(ChDataPatcher::RECORD_NAME, field, "u32", new))
        }
        Value::String(s) => s
            .trim()
            .parse::<u32>()
            .map_err(|_| wrong_type(ChDataPatcher::RECORD_NAME, field, "u32", new)),
        _ => Err(wrong_type(ChDataPatcher::RECORD_NAME, field, "u32", new)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_blob() -> Vec<u8> {
        let mut buf = Vec::with_capacity(84);
        buf.extend_from_slice(b"Item");
        buf.extend(vec![0u8; 26]);
        buf.extend(vec![0u8; 32]); // 16 × u16
        buf.extend(vec![0u8; 2]);
        buf.extend(vec![0u8; 16]); // 4 × u32
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf
    }

    fn parse_back(b: &[u8]) -> ChData {
        ChData::parse(&mut Cursor::new(b), b.len() as u64)
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
    }

    #[test]
    fn change_total() {
        let p = ChDataPatcher;
        let out = p
            .apply_field(&empty_blob(), 0, "total", &Value::I64(42))
            .unwrap();
        assert_eq!(parse_back(&out).total, 42);
    }

    #[test]
    fn change_value_at_index() {
        let p = ChDataPatcher;
        let out = p
            .apply_field(&empty_blob(), 0, "values.7", &Value::I64(1234))
            .unwrap();
        let r = parse_back(&out);
        assert_eq!(r.values[7], 1234);
        assert_eq!(r.values[6], 0);
        assert_eq!(r.values[8], 0);
    }

    #[test]
    fn change_count_at_index() {
        let p = ChDataPatcher;
        let out = p
            .apply_field(&empty_blob(), 0, "counts.2", &Value::I64(99))
            .unwrap();
        assert_eq!(parse_back(&out).counts[2], 99);
    }

    #[test]
    fn values_index_out_of_range_errors() {
        let p = ChDataPatcher;
        let err = p
            .apply_field(&empty_blob(), 0, "values.16", &Value::I64(1))
            .unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn bare_vector_field_rejected_with_hint() {
        let p = ChDataPatcher;
        let err = p
            .apply_field(&empty_blob(), 0, "values", &Value::I64(1))
            .unwrap_err();
        assert!(err.to_string().contains("address an element"));
    }

    #[test]
    fn record_id_must_be_zero() {
        let p = ChDataPatcher;
        let err = p
            .apply_field(&empty_blob(), 1, "total", &Value::I64(1))
            .unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn unknown_field_errors() {
        let p = ChDataPatcher;
        let err = p
            .apply_field(&empty_blob(), 0, "luck", &Value::I64(1))
            .unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }
}
