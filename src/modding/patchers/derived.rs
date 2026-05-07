//! Coverage tests for the auto-generated `RecordPatcher` impls — proves the
//! `#[derive(RecordPatcher)]` macro handles every field shape currently in
//! use across `references/`: strings, primitives of varying widths, u8
//! arrays, and enum-discriminants. Hand-rolled patchers (multi-section
//! INIs, scripts, dialogues) live in their own files alongside these.

#[cfg(test)]
mod tests {
    use crate::modding::patcher::RecordPatcher;
    use crate::modding::patchers::{EditItemPatcher, EventItemPatcher, HealItemPatcher};
    use crate::modding::value::Value;
    use crate::references::edit_item_db::EditItem;
    use crate::references::enums::{EditItemEffect, HealItemFlag};
    use crate::references::event_item_db::EventItem;
    use crate::references::extractor::Extractor;
    use crate::references::heal_item_db::HealItem;
    use std::io::Cursor;

    // -------------------------------------------------------------------- HealItem

    fn heal_blob() -> Vec<u8> {
        let mut data = 1i32.to_le_bytes().to_vec();
        let mut name_buf = [0u8; 30];
        name_buf[..6].copy_from_slice(b"Potion");
        data.extend_from_slice(&name_buf);
        data.extend(vec![0u8; 202]); // description
        data.extend_from_slice(&50i16.to_le_bytes()); // base_price
        data.extend(vec![0u8; 6]); // pad1-3 (i16s)
        data.extend_from_slice(&100i16.to_le_bytes()); // health_points
        data.extend(vec![0u8; 10]); // mp + 5 flag bytes + pad
        data
    }

    fn parse_heal(b: &[u8]) -> Vec<HealItem> {
        HealItem::parse(&mut Cursor::new(b), b.len() as u64).unwrap()
    }

    #[test]
    fn heal_string_field() {
        let p = HealItemPatcher;
        let out = p
            .apply_field(&heal_blob(), 0, "name", &Value::String("Elixir".into()))
            .unwrap();
        assert_eq!(parse_heal(&out)[0].name, "Elixir");
    }

    #[test]
    fn heal_i16_primitive_via_i64() {
        let p = HealItemPatcher;
        let out = p
            .apply_field(&heal_blob(), 0, "health_points", &Value::I64(250))
            .unwrap();
        assert_eq!(parse_heal(&out)[0].health_points, 250);
    }

    #[test]
    fn heal_i16_primitive_via_string() {
        let p = HealItemPatcher;
        let out = p
            .apply_field(&heal_blob(), 0, "base_price", &Value::String("999".into()))
            .unwrap();
        assert_eq!(parse_heal(&out)[0].base_price, 999);
    }

    #[test]
    fn heal_enum_from_u8_via_i64() {
        // HealItemFlag::FullRestoration discriminant.
        let enabled_disc = u8::from(HealItemFlag::FullRestoration);
        let p = HealItemPatcher;
        let out = p
            .apply_field(
                &heal_blob(),
                0,
                "poison_heal",
                &Value::I64(enabled_disc as i64),
            )
            .unwrap();
        assert_eq!(parse_heal(&out)[0].poison_heal, HealItemFlag::FullRestoration);
    }

    #[test]
    fn heal_enum_invalid_discriminant_falls_back_to_default() {
        // Per the parse-side behavior, an unknown discriminant becomes
        // `Default::default()`. We mirror that on patch.
        let p = HealItemPatcher;
        let out = p
            .apply_field(&heal_blob(), 0, "poison_heal", &Value::I64(255))
            .unwrap();
        assert_eq!(parse_heal(&out)[0].poison_heal, HealItemFlag::default());
    }

    #[test]
    fn heal_padding_field_rejected() {
        let p = HealItemPatcher;
        let err = p
            .apply_field(&heal_blob(), 0, "padding5", &Value::I64(0))
            .unwrap_err();
        assert!(err.to_string().contains("synthetic padding"));
    }

    // -------------------------------------------------------------------- EditItem

    fn edit_blob() -> Vec<u8> {
        // 4-byte header + 268-byte record.
        let mut data = 1i32.to_le_bytes().to_vec();
        let mut name_buf = [0u8; 30];
        name_buf[..5].copy_from_slice(b"Sword");
        data.extend_from_slice(&name_buf);
        data.extend(vec![0u8; 202]);
        // 13 i16 fields (base_price, padding1-3, hp, mp, str, agi, wis, con,
        // dodge, hit, offense), defense, magical_power, item_destroying_power
        // = 16 i16 = 32 bytes
        data.extend(vec![0u8; 32]);
        // padding4 u8 + modifies_item u8 + additional_effect i16 = 4 bytes
        data.extend(vec![0u8; 4]);
        data
    }

    fn parse_edit(b: &[u8]) -> Vec<EditItem> {
        EditItem::parse(&mut Cursor::new(b), b.len() as u64).unwrap()
    }

    #[test]
    fn edit_index_field_rejected() {
        let p = EditItemPatcher;
        let err = p
            .apply_field(&edit_blob(), 0, "index", &Value::I64(7))
            .unwrap_err();
        assert!(err.to_string().contains("positional"));
    }

    #[test]
    fn edit_enum_from_i16_via_i64() {
        let p = EditItemPatcher;
        let target = i16::from(EditItemEffect::ManaDrain);
        let out = p
            .apply_field(
                &edit_blob(),
                0,
                "additional_effect",
                &Value::I64(target as i64),
            )
            .unwrap();
        assert_eq!(
            parse_edit(&out)[0].additional_effect,
            EditItemEffect::ManaDrain
        );
    }

    #[test]
    fn edit_unknown_field_errors() {
        let p = EditItemPatcher;
        let err = p
            .apply_field(&edit_blob(), 0, "nonexistent", &Value::I64(1))
            .unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }

    // -------------------------------------------------------------------- EventItem (array field)

    fn event_blob() -> Vec<u8> {
        let mut data = 1i32.to_le_bytes().to_vec();
        let mut name_buf = [0u8; 30];
        name_buf[..3].copy_from_slice(b"Key");
        data.extend_from_slice(&name_buf);
        data.extend(vec![0u8; 202]); // description
        data.extend(vec![0u8; 8]); // padding [u8; 8]
        data
    }

    fn parse_event(b: &[u8]) -> Vec<EventItem> {
        EventItem::parse(&mut Cursor::new(b), b.len() as u64).unwrap()
    }

    #[test]
    fn event_array_u8_via_bytes() {
        let p = EventItemPatcher;
        let bytes = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let out = p
            .apply_field(&event_blob(), 0, "padding", &Value::Bytes(bytes.clone()))
            .unwrap();
        assert_eq!(&parse_event(&out)[0].padding[..], &bytes[..]);
    }

    #[test]
    fn event_array_wrong_length_errors() {
        let p = EventItemPatcher;
        let err = p
            .apply_field(&event_blob(), 0, "padding", &Value::Bytes(vec![1, 2, 3]))
            .unwrap_err();
        assert!(err.to_string().contains("expected 8 bytes"));
    }
}
