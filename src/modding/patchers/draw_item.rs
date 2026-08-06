//! Field-level patcher for `Ref/DRAWITEM.ref` (parenthesised CSV).
//!
//! Hand-written because [`DrawItem`] uses an ad-hoc parse/serialize pair
//! (parenthesised CSV with a packed `(item_id, item_type)` byte pair shoved
//! into a single i32 field on the wire). The `TextRecordPatcher` derive
//! assumes flat CSV with `field = N` indices, which doesn't match.
//!
//! `record_id` is the row index (0-based) in document order.

use std::io::Cursor;

use crate::modding::error::Result;
use crate::modding::patcher::{RecordPatcher, out_of_range, unknown_field, wrong_type};
use crate::modding::value::Value;
use crate::references::draw_item::DrawItem;
use crate::references::enums::{InventoryItem, ItemTypeId};
use crate::references::extractor::Extractor;

pub struct DrawItemPatcher;

impl DrawItemPatcher {
    pub const FILENAME: &'static str = "DRAWITEM.ref";
    pub const RECORD_NAME: &'static str = "DrawItem";
}

impl RecordPatcher for DrawItemPatcher {
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
        let mut items = DrawItem::parse(&mut cursor, bytes.len() as u64)?;

        let idx = record_id as usize;
        if idx >= items.len() {
            return Err(out_of_range(Self::RECORD_NAME, record_id, items.len()));
        }
        let rec = &mut items[idx];

        match field {
            "map_id" => rec.map_id = parse_i32(field, new)?,
            "x_coord" => rec.x_coord = parse_i32(field, new)?,
            "y_coord" => rec.y_coord = parse_i32(field, new)?,
            "item" => {
                let raw = parse_i32(field, new)?;
                rec.item = InventoryItem::from(raw);
            }
            "item_id" => {
                let new_id = parse_i32(field, new)? as u8;
                let current_type = rec.item.item_type().unwrap_or(ItemTypeId::Other);
                rec.item = InventoryItem::new(current_type, new_id);
            }
            "item_type" => {
                let new_type_raw = parse_i32(field, new)? as u8;
                let new_type = ItemTypeId::from_u8(new_type_raw).unwrap_or(ItemTypeId::Other);
                let current_id = rec.item.item_id();
                rec.item = InventoryItem::new(new_type, current_id);
            }
            other => return Err(unknown_field(Self::RECORD_NAME, other)),
        }

        let mut out = Vec::new();
        DrawItem::to_writer(&items, &mut out)?;
        Ok(out)
    }
}

fn parse_i32(field: &str, new: &Value) -> Result<i32> {
    match new {
        Value::I64(v) => i32::try_from(*v)
            .map_err(|_| wrong_type(DrawItemPatcher::RECORD_NAME, field, "i32", new)),
        Value::String(s) => s
            .trim()
            .parse::<i32>()
            .map_err(|_| wrong_type(DrawItemPatcher::RECORD_NAME, field, "i32", new)),
        _ => Err(wrong_type(DrawItemPatcher::RECORD_NAME, field, "i32", new)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::enums::ItemTypeId;

    fn one_record_blob() -> Vec<u8> {
        // (1, 10, 20, encoded(item_id=5, item_type=Healing=2))
        // encoded = 5 + 2*256 = 517
        b"(1,10,20,517)\r\n".to_vec()
    }

    fn parse_back(b: &[u8]) -> Vec<DrawItem> {
        DrawItem::parse(&mut Cursor::new(b), b.len() as u64).unwrap()
    }

    #[test]
    fn change_coord() {
        let p = DrawItemPatcher;
        let out = p
            .apply_field(&one_record_blob(), 0, "x_coord", &Value::I64(99))
            .unwrap();
        let recs = parse_back(&out);
        assert_eq!(recs[0].x_coord, 99);
        assert_eq!(recs[0].item.item_id(), 5); // packed encoding preserved
        assert_eq!(recs[0].item.item_type(), Some(ItemTypeId::Healing));
    }

    #[test]
    fn change_item_id_preserves_item_type() {
        // The on-wire encoding packs id + type into a single i32; verify
        // that patching just `item_id` leaves the type intact through the
        // serialize/reparse round-trip.
        let p = DrawItemPatcher;
        let out = p
            .apply_field(&one_record_blob(), 0, "item_id", &Value::I64(42))
            .unwrap();
        let recs = parse_back(&out);
        assert_eq!(recs[0].item.item_id(), 42);
        assert_eq!(recs[0].item.item_type(), Some(ItemTypeId::Healing));
    }

    #[test]
    fn change_item_type_preserves_item_id() {
        let p = DrawItemPatcher;
        // ItemTypeId::Other = 0
        let out = p
            .apply_field(&one_record_blob(), 0, "item_type", &Value::I64(0))
            .unwrap();
        let recs = parse_back(&out);
        assert_eq!(recs[0].item.item_type(), Some(ItemTypeId::Other));
        assert_eq!(recs[0].item.item_id(), 5);
    }

    #[test]
    fn unknown_item_type_falls_back_to_other() {
        let p = DrawItemPatcher;
        let out = p
            .apply_field(&one_record_blob(), 0, "item_type", &Value::I64(250))
            .unwrap();
        assert_eq!(
            parse_back(&out)[0].item.item_type(),
            Some(ItemTypeId::Other)
        );
    }

    #[test]
    fn item_id_truncated_to_u8() {
        let p = DrawItemPatcher;
        // item_id is stored in the low byte; 300 as u8 = 44, type preserved
        let out = p
            .apply_field(&one_record_blob(), 0, "item_id", &Value::I64(300))
            .unwrap();
        let recs = parse_back(&out);
        assert_eq!(recs[0].item.item_id(), 44);
        assert_eq!(recs[0].item.item_type(), Some(ItemTypeId::Healing));
    }

    #[test]
    fn unknown_field_errors() {
        let p = DrawItemPatcher;
        let err = p
            .apply_field(&one_record_blob(), 0, "color", &Value::I64(1))
            .unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn out_of_range_record_id_errors() {
        let p = DrawItemPatcher;
        let _err = p
            .apply_field(&one_record_blob(), 0, "x_coord", &Value::I64(0))
            .ok();
        // record_id 0 should succeed; check 99 fails.
        let err = p
            .apply_field(&one_record_blob(), 99, "x_coord", &Value::I64(0))
            .unwrap_err();
        assert!(err.to_string().contains("out of range"));
        let _ = err;
    }
}
