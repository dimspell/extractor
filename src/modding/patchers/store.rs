//! Field-level patcher for `CharacterInGame/Store.db` (shops & inns).
//!
//! Hand-written because [`Store`] has a conditional structure: when
//! `inn_night_cost > 0` the record is an inn (144 bytes of padding,
//! no products); otherwise it's a shop with up to 71 `(type, id)`
//! product pairs. Field patching only covers the scalar fields —
//! mutating the product list is best done by replacing the whole
//! file via `FileReplace`, since rebalancing 71 packed slots from
//! a single `FieldDelta` would be fragile.

use std::io::Cursor;

use crate::modding::error::{ModdingError, Result};
use crate::modding::patcher::{out_of_range, unknown_field, wrong_type, RecordPatcher};
use crate::modding::value::Value;
use crate::references::extractor::Extractor;
use crate::references::store_db::Store;

pub struct StorePatcher;

impl StorePatcher {
    pub const FILENAME: &'static str = "Store.db";
    pub const RECORD_NAME: &'static str = "Store";
}

impl RecordPatcher for StorePatcher {
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
        let mut stores = Store::parse(&mut cursor, bytes.len() as u64)?;

        let idx = record_id as usize;
        if idx >= stores.len() {
            return Err(out_of_range(Self::RECORD_NAME, record_id, stores.len()));
        }
        let rec = &mut stores[idx];

        match field {
            "store_name" => rec.store_name = parse_string(field, new)?,
            "invitation" => rec.invitation = parse_string(field, new)?,
            "haggle_success" => rec.haggle_success = parse_string(field, new)?,
            "haggle_fail" => rec.haggle_fail = parse_string(field, new)?,
            "inn_night_cost" => rec.inn_night_cost = parse_i32(field, new)?,
            "some_unknown_number" => rec.some_unknown_number = parse_i16(field, new)?,
            "index" => {
                return Err(ModdingError::Malformed(format!(
                    "{}.index is positional and cannot be patched",
                    Self::RECORD_NAME
                )));
            }
            "products" => {
                return Err(ModdingError::Malformed(format!(
                    "{}.products is a structured list; replace the whole \
                     Store.db via FileReplace to change inventories",
                    Self::RECORD_NAME
                )));
            }
            other => return Err(unknown_field(Self::RECORD_NAME, other)),
        }

        let mut out = Vec::with_capacity(bytes.len());
        Store::to_writer(&stores, &mut out)?;
        Ok(out)
    }
}

fn parse_string(field: &str, new: &Value) -> Result<String> {
    match new {
        Value::String(s) => Ok(s.clone()),
        _ => Err(wrong_type(StorePatcher::RECORD_NAME, field, "string", new)),
    }
}

fn parse_i32(field: &str, new: &Value) -> Result<i32> {
    match new {
        Value::I64(v) => i32::try_from(*v)
            .map_err(|_| wrong_type(StorePatcher::RECORD_NAME, field, "i32", new)),
        Value::String(s) => s
            .trim()
            .parse::<i32>()
            .map_err(|_| wrong_type(StorePatcher::RECORD_NAME, field, "i32", new)),
        _ => Err(wrong_type(StorePatcher::RECORD_NAME, field, "i32", new)),
    }
}

fn parse_i16(field: &str, new: &Value) -> Result<i16> {
    match new {
        Value::I64(v) => i16::try_from(*v)
            .map_err(|_| wrong_type(StorePatcher::RECORD_NAME, field, "i16", new)),
        Value::String(s) => s
            .trim()
            .parse::<i16>()
            .map_err(|_| wrong_type(StorePatcher::RECORD_NAME, field, "i16", new)),
        _ => Err(wrong_type(StorePatcher::RECORD_NAME, field, "i16", new)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_str(s: &str, len: usize) -> Vec<u8> {
        let mut buf = vec![0u8; len];
        let (cow, _, _) = encoding_rs::WINDOWS_1250.encode(s);
        let n = cow.len().min(len);
        buf[..n].copy_from_slice(&cow[..n]);
        buf
    }

    fn one_inn_blob(name: &str, night_cost: i32) -> Vec<u8> {
        let mut data = 1i32.to_le_bytes().to_vec(); // header
        let mut rec = Vec::with_capacity(948);
        rec.extend(encode_str(name, 32));
        rec.extend_from_slice(&night_cost.to_le_bytes());
        rec.extend(vec![0u8; 144]); // inn padding
        rec.extend(encode_str("Welcome!", 512));
        rec.extend(encode_str("Sure!", 128));
        rec.extend(encode_str("No.", 128));
        data.extend(&rec);
        data
    }

    fn parse_back(b: &[u8]) -> Vec<Store> {
        Store::parse(&mut Cursor::new(b), b.len() as u64).unwrap()
    }

    #[test]
    fn change_store_name() {
        let p = StorePatcher;
        let out = p
            .apply_field(
                &one_inn_blob("Tavern", 50),
                0,
                "store_name",
                &Value::String("Cozy Inn".into()),
            )
            .unwrap();
        let recs = parse_back(&out);
        assert_eq!(recs[0].store_name, "Cozy Inn");
        assert_eq!(recs[0].inn_night_cost, 50);
    }

    #[test]
    fn change_inn_night_cost() {
        let p = StorePatcher;
        let out = p
            .apply_field(
                &one_inn_blob("Tavern", 50),
                0,
                "inn_night_cost",
                &Value::I64(99),
            )
            .unwrap();
        assert_eq!(parse_back(&out)[0].inn_night_cost, 99);
    }

    #[test]
    fn change_haggle_text() {
        let p = StorePatcher;
        let out = p
            .apply_field(
                &one_inn_blob("Tavern", 50),
                0,
                "haggle_fail",
                &Value::String("Get out!".into()),
            )
            .unwrap();
        assert_eq!(parse_back(&out)[0].haggle_fail, "Get out!");
    }

    #[test]
    fn products_field_rejected_with_helpful_message() {
        let p = StorePatcher;
        let err = p
            .apply_field(&one_inn_blob("Tavern", 50), 0, "products", &Value::Null)
            .unwrap_err();
        assert!(err.to_string().contains("FileReplace"));
    }

    #[test]
    fn index_field_rejected() {
        let p = StorePatcher;
        let err = p
            .apply_field(&one_inn_blob("Tavern", 50), 0, "index", &Value::I64(0))
            .unwrap_err();
        assert!(err.to_string().contains("positional"));
    }

    #[test]
    fn out_of_range_id_errors() {
        let p = StorePatcher;
        let err = p
            .apply_field(&one_inn_blob("Tavern", 50), 99, "store_name", &Value::String("x".into()))
            .unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }
}
