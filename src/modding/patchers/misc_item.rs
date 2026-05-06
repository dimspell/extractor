//! Field-level patcher for `CharacterInGame/MiscItem.db`.

use std::io::Cursor;

use crate::modding::error::Result;
use crate::modding::patcher::{out_of_range, unknown_field, wrong_type, RecordPatcher};
use crate::modding::value::Value;
use crate::references::extractor::Extractor;
use crate::references::misc_item_db::MiscItem;

const NAME: &str = "MiscItem";

pub struct MiscItemPatcher;

impl RecordPatcher for MiscItemPatcher {
    fn name(&self) -> &'static str {
        NAME
    }

    fn apply_field(
        &self,
        bytes: &[u8],
        record_id: u32,
        field: &str,
        new: &Value,
    ) -> Result<Vec<u8>> {
        // Parse → mutate → serialise.
        let mut cursor = Cursor::new(bytes);
        let mut records = MiscItem::parse(&mut cursor, bytes.len() as u64)?;

        let idx = record_id as usize;
        if idx >= records.len() {
            return Err(out_of_range(NAME, record_id, records.len()));
        }
        let rec = &mut records[idx];

        match field {
            "name" => match new {
                Value::String(s) => rec.name = s.clone(),
                _ => return Err(wrong_type(NAME, field, "string", new)),
            },
            "description" => match new {
                Value::String(s) => rec.description = s.clone(),
                _ => return Err(wrong_type(NAME, field, "string", new)),
            },
            "base_price" => match new {
                Value::I64(v) => rec.base_price = (*v) as i32,
                _ => return Err(wrong_type(NAME, field, "i64", new)),
            },
            "padding" => match new {
                Value::Bytes(b) if b.len() == 20 => {
                    let mut arr = [0u8; 20];
                    arr.copy_from_slice(b);
                    rec.padding = arr;
                }
                Value::Bytes(_) => {
                    return Err(crate::modding::error::ModdingError::Malformed(format!(
                        "{NAME}.padding: expected 20 bytes, got {}",
                        if let Value::Bytes(b) = new {
                            b.len()
                        } else {
                            0
                        }
                    )));
                }
                _ => return Err(wrong_type(NAME, field, "bytes(20)", new)),
            },
            // `id` is positional; you can't change it via a delta.
            "id" => {
                return Err(crate::modding::error::ModdingError::Malformed(format!(
                    "{NAME}.id is positional and cannot be patched"
                )));
            }
            other => return Err(unknown_field(NAME, other)),
        }

        let mut out = Vec::with_capacity(bytes.len());
        MiscItem::to_writer(&records, &mut out)?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_item_blob(name: &str, base_price: i32) -> Vec<u8> {
        let mut data = 1i32.to_le_bytes().to_vec();
        let mut name_buf = [0u8; 30];
        let n = name.len().min(29);
        name_buf[..n].copy_from_slice(&name.as_bytes()[..n]);
        data.extend_from_slice(&name_buf);
        data.extend(vec![0u8; 202]);
        data.extend_from_slice(&base_price.to_le_bytes());
        data.extend(vec![0u8; 20]);
        data
    }

    fn parse_back(bytes: &[u8]) -> Vec<MiscItem> {
        let mut c = Cursor::new(bytes);
        MiscItem::parse(&mut c, bytes.len() as u64).unwrap()
    }

    #[test]
    fn rename_field() {
        let p = MiscItemPatcher;
        let original = one_item_blob("Helt", 15);
        let patched = p
            .apply_field(&original, 0, "name", &Value::String("Helmet".into()))
            .unwrap();
        let recs = parse_back(&patched);
        assert_eq!(recs[0].name, "Helmet");
        assert_eq!(recs[0].base_price, 15);
    }

    #[test]
    fn change_price() {
        let p = MiscItemPatcher;
        let original = one_item_blob("Torch", 15);
        let patched = p
            .apply_field(&original, 0, "base_price", &Value::I64(99))
            .unwrap();
        let recs = parse_back(&patched);
        assert_eq!(recs[0].base_price, 99);
        assert_eq!(recs[0].name, "Torch");
    }

    #[test]
    fn unknown_field_errors() {
        let p = MiscItemPatcher;
        let bytes = one_item_blob("X", 1);
        let err = p
            .apply_field(&bytes, 0, "nope", &Value::String("".into()))
            .unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn wrong_type_errors() {
        let p = MiscItemPatcher;
        let bytes = one_item_blob("X", 1);
        let err = p
            .apply_field(&bytes, 0, "base_price", &Value::String("99".into()))
            .unwrap_err();
        assert!(err.to_string().contains("expected i64"));
    }

    #[test]
    fn out_of_range_errors() {
        let p = MiscItemPatcher;
        let bytes = one_item_blob("X", 1);
        let err = p
            .apply_field(&bytes, 5, "name", &Value::String("Y".into()))
            .unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn id_field_rejected() {
        let p = MiscItemPatcher;
        let bytes = one_item_blob("X", 1);
        let err = p.apply_field(&bytes, 0, "id", &Value::I64(99)).unwrap_err();
        assert!(err.to_string().contains("positional"));
    }
}
