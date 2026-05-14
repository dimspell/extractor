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
        assert_eq!(
            parse_heal(&out)[0].poison_heal,
            HealItemFlag::FullRestoration
        );
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

    // -------------------------------------------------------------------- ExtraRef (pattern + complex enums)

    fn extra_ref_blob() -> Vec<u8> {
        let mut data = 1i32.to_le_bytes().to_vec();
        let mut rec = vec![0u8; 184];
        rec[0] = 1; // number_in_file
        rec[2] = 3; // ext_id
        rec[3..3 + 6].copy_from_slice(b"Chest1");
        // x_pos at 36
        rec[36..40].copy_from_slice(&10i32.to_le_bytes());
        // y_pos at 40
        rec[40..44].copy_from_slice(&20i32.to_le_bytes());
        // gold at 80
        rec[80..84].copy_from_slice(&50i32.to_le_bytes());
        data.extend_from_slice(&rec);
        data
    }

    fn parse_extra(b: &[u8]) -> Vec<crate::references::extra_ref::ExtraRef> {
        crate::references::extra_ref::ExtraRef::parse(&mut Cursor::new(b), b.len() as u64).unwrap()
    }

    #[test]
    fn extra_ref_string_field() {
        let p = crate::modding::patchers::ExtraRefPatcher;
        let out = p
            .apply_field(
                &extra_ref_blob(),
                0,
                "name",
                &Value::String("Treasure".into()),
            )
            .unwrap();
        assert_eq!(parse_extra(&out)[0].name, "Treasure");
    }

    #[test]
    fn extra_ref_i32_primitive() {
        let p = crate::modding::patchers::ExtraRefPatcher;
        let out = p
            .apply_field(&extra_ref_blob(), 0, "gold_amount", &Value::I64(9999))
            .unwrap();
        assert_eq!(parse_extra(&out)[0].gold_amount, 9999);
    }

    #[test]
    fn extra_ref_vec_u8_field() {
        let p = crate::modding::patchers::ExtraRefPatcher;
        let bytes = vec![0xAA, 0xBB, 0xCC];
        let out = p
            .apply_field(
                &extra_ref_blob(),
                0,
                "unknown2",
                &Value::Bytes(bytes.clone()),
            )
            .unwrap();
        assert_eq!(parse_extra(&out)[0].unknown2, bytes);
    }

    #[test]
    fn extra_ref_enum_from_i32_via_i64() {
        // BooleanFlag::True discriminant = 1 (per references/enums.rs).
        use crate::references::enums::BooleanFlag;
        let p = crate::modding::patchers::ExtraRefPatcher;
        let out = p
            .apply_field(&extra_ref_blob(), 0, "closed", &Value::I64(1))
            .unwrap();
        let parsed = &parse_extra(&out)[0];
        assert_ne!(parsed.closed, BooleanFlag::default());
    }

    #[test]
    fn extra_ref_enum_from_i32_from_u8() {
        // visibility uses enum_from_i32_from_u8(VisibilityType) — wire is u8.
        let p = crate::modding::patchers::ExtraRefPatcher;
        let out = p
            .apply_field(&extra_ref_blob(), 0, "visibility", &Value::I64(1))
            .unwrap();
        // Round-trip should not panic; just verify parse succeeds.
        assert!(!parse_extra(&out).is_empty());
    }

    #[test]
    fn extra_ref_id_field_rejected() {
        let p = crate::modding::patchers::ExtraRefPatcher;
        let err = p
            .apply_field(&extra_ref_blob(), 0, "id", &Value::I64(7))
            .unwrap_err();
        assert!(err.to_string().contains("positional"));
    }

    // -------------------------------------------------------------------- Monster (binary, no header)

    fn monster_blob() -> Vec<u8> {
        // 24-byte name + 34 × i32 = 160 bytes; no record-count header
        // (`counter_size = 0`).
        let mut data = Vec::with_capacity(160);
        let mut name = [0u8; 24];
        name[..6].copy_from_slice(b"Goblin");
        data.extend_from_slice(&name);
        for _ in 0..34 {
            data.extend_from_slice(&0i32.to_le_bytes());
        }
        data
    }

    fn parse_monster(b: &[u8]) -> Vec<crate::references::monster_db::Monster> {
        crate::references::monster_db::Monster::parse(&mut Cursor::new(b), b.len() as u64).unwrap()
    }

    #[test]
    fn monster_string_field() {
        let p = crate::modding::patchers::MonsterPatcher;
        let out = p
            .apply_field(&monster_blob(), 0, "name", &Value::String("Dragon".into()))
            .unwrap();
        assert_eq!(parse_monster(&out)[0].name, "Dragon");
    }

    #[test]
    fn monster_i32_field() {
        let p = crate::modding::patchers::MonsterPatcher;
        let out = p
            .apply_field(&monster_blob(), 0, "health_points_max", &Value::I64(500))
            .unwrap();
        assert_eq!(parse_monster(&out)[0].health_points_max, 500);
    }

    #[test]
    fn monster_index_field_rejected() {
        let p = crate::modding::patchers::MonsterPatcher;
        let err = p
            .apply_field(&monster_blob(), 0, "id", &Value::I64(7))
            .unwrap_err();
        assert!(err.to_string().contains("positional"));
    }

    // -------------------------------------------------------------------- AllMap.ini (text, derived)

    fn allmap_blob() -> Vec<u8> {
        b"1,cat1,Forest,null,null,0\r\n2,cat2,Dungeon,pgp2,dlg2,1\r\n".to_vec()
    }

    fn parse_allmap(b: &[u8]) -> Vec<crate::references::all_map_ini::Map> {
        crate::references::all_map_ini::Map::parse(&mut Cursor::new(b), b.len() as u64).unwrap()
    }

    #[test]
    fn allmap_string_field() {
        let p = crate::modding::patchers::MapPatcher;
        let out = p
            .apply_field(
                &allmap_blob(),
                0,
                "map_name",
                &Value::String("Plains".into()),
            )
            .unwrap();
        let recs = parse_allmap(&out);
        assert_eq!(recs[0].map_name, "Plains");
        assert_eq!(recs[1].map_name, "Dungeon"); // row 1 untouched
    }

    #[test]
    fn allmap_option_string_set_to_some() {
        let p = crate::modding::patchers::MapPatcher;
        let out = p
            .apply_field(
                &allmap_blob(),
                0,
                "pgp_filename",
                &Value::String("new.pgp".into()),
            )
            .unwrap();
        assert_eq!(
            parse_allmap(&out)[0].pgp_filename.as_deref(),
            Some("new.pgp")
        );
    }

    #[test]
    fn allmap_option_string_set_to_null_via_string_literal() {
        let p = crate::modding::patchers::MapPatcher;
        let out = p
            .apply_field(
                &allmap_blob(),
                1,
                "pgp_filename",
                &Value::String("null".into()),
            )
            .unwrap();
        assert_eq!(parse_allmap(&out)[1].pgp_filename, None);
    }

    #[test]
    fn allmap_enum_field_via_i64() {
        // MapLighting::Dark == 1
        let p = crate::modding::patchers::MapPatcher;
        let out = p
            .apply_field(&allmap_blob(), 0, "lighting", &Value::I64(1))
            .unwrap();
        use crate::references::enums::MapLighting;
        assert_eq!(parse_allmap(&out)[0].lighting, MapLighting::Dark);
    }

    #[test]
    fn allmap_id_field_rejected() {
        let p = crate::modding::patchers::MapPatcher;
        let err = p
            .apply_field(&allmap_blob(), 0, "id", &Value::I64(99))
            .unwrap_err();
        assert!(err.to_string().contains("positional"));
    }

    // -------------------------------------------------------------------- Eventnpc.ref (text, derived)

    #[test]
    fn eventnpc_text_record() {
        use crate::references::event_npc_ref::EventNpcRef;
        let p = crate::modding::patchers::EventNpcRefPatcher;
        let blob = b"1,100,Guard Bob\r\n2,200,Merchant\r\n";
        let out = p
            .apply_field(blob, 1, "name", &Value::String("Hero".into()))
            .unwrap();
        let recs = EventNpcRef::parse(&mut Cursor::new(&out), out.len() as u64).unwrap();
        assert_eq!(recs[1].name, "Hero");
        assert_eq!(recs[0].name, "Guard Bob");
    }

    #[test]
    fn extra_ref_padding_field_rejected() {
        // unknown1 is declared as `padding(...)` in the struct.
        let p = crate::modding::patchers::ExtraRefPatcher;
        let err = p
            .apply_field(&extra_ref_blob(), 0, "unknown1", &Value::I64(0))
            .unwrap_err();
        assert!(err.to_string().contains("synthetic padding"));
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
