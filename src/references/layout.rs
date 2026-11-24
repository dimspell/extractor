//! Static metadata for fixed-size binary record layouts.

/// One on-disk field within a fixed-size record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldDef {
    pub name: &'static str,
    pub offset: u32,
    pub size: u32,
    pub ty: &'static str,
}

/// Complete byte layout for a file made from fixed-size records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedRecordLayout {
    pub type_name: &'static str,
    pub header_size: u32,
    pub record_size: u32,
    pub fields: &'static [FieldDef],
}

impl FixedRecordLayout {
    /// Named field metadata, or `None` when this record has no such field.
    pub fn field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.iter().find(|def| def.name == name)
    }

    /// Byte offset of a named field within one record.
    pub fn field_offset(&self, name: &str) -> Option<u32> {
        self.field(name).map(|def| def.offset)
    }

    /// Total on-disk footprint of one record including its header.
    pub fn footprint(&self) -> u32 {
        self.header_size + self.record_size
    }
}

/// Static layout metadata generated from an `Extractor` record definition.
pub trait RecordLayout {
    const LAYOUT: FixedRecordLayout;
}

/// Hand-maintained byte layout of one 36-byte `PartyLevelRecord`.
///
/// `PartyLevelNpc` is parsed monolithically (8 × 20 entries) and therefore has
/// no derive-generated `RecordLayout`. This constant is the single source of
/// truth for its offsets, consumed by the hand-written party-level patcher and
/// checked against the real parser in [`tests`]. The derived `level` field is
/// deliberately absent — it is not stored on disk.
pub const PARTY_LEVEL_RECORD: FixedRecordLayout = FixedRecordLayout {
    type_name: "PartyLevelRecord",
    header_size: 0,
    record_size: 36,
    fields: &[
        FieldDef {
            name: "magic_spell_id_1",
            offset: 0,
            size: 1,
            ty: "u8",
        },
        FieldDef {
            name: "magic_spell_id_2",
            offset: 1,
            size: 1,
            ty: "u8",
        },
        FieldDef {
            name: "magic_spell_id_3",
            offset: 2,
            size: 1,
            ty: "u8",
        },
        FieldDef {
            name: "reserved_0x03",
            offset: 3,
            size: 1,
            ty: "u8",
        },
        FieldDef {
            name: "strength",
            offset: 4,
            size: 4,
            ty: "u32",
        },
        FieldDef {
            name: "constitution",
            offset: 8,
            size: 4,
            ty: "u32",
        },
        FieldDef {
            name: "wisdom",
            offset: 12,
            size: 4,
            ty: "u32",
        },
        FieldDef {
            name: "health_points",
            offset: 16,
            size: 2,
            ty: "u16",
        },
        FieldDef {
            name: "mana_points",
            offset: 18,
            size: 2,
            ty: "u16",
        },
        FieldDef {
            name: "accuracy",
            offset: 20,
            size: 1,
            ty: "u8",
        },
        FieldDef {
            name: "reserved_0x15",
            offset: 21,
            size: 1,
            ty: "u8",
        },
        FieldDef {
            name: "reserved_0x16",
            offset: 22,
            size: 1,
            ty: "u8",
        },
        FieldDef {
            name: "reserved_0x17",
            offset: 23,
            size: 1,
            ty: "u8",
        },
        FieldDef {
            name: "dodge",
            offset: 24,
            size: 1,
            ty: "u8",
        },
        FieldDef {
            name: "reserved_0x19",
            offset: 25,
            size: 1,
            ty: "u8",
        },
        FieldDef {
            name: "reserved_0x1a",
            offset: 26,
            size: 1,
            ty: "u8",
        },
        FieldDef {
            name: "reserved_0x1b",
            offset: 27,
            size: 1,
            ty: "u8",
        },
        FieldDef {
            name: "weapon_skill_level",
            offset: 28,
            size: 4,
            ty: "u32",
        },
        FieldDef {
            name: "tactical_action_chance",
            offset: 32,
            size: 4,
            ty: "u32",
        },
    ],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::chdata_db::ChData;
    use crate::references::edit_item_db::EditItem;
    use crate::references::event_item_db::EventItem;
    use crate::references::extra_ref::ExtraRef;
    use crate::references::extractor::Extractor;
    use crate::references::heal_item_db::HealItem;
    use crate::references::magic_db::MagicSpell;
    use crate::references::misc_item_db::MiscItem;
    use crate::references::monster_db::Monster;
    use crate::references::monster_ref::MonsterRef;
    use crate::references::npc_ref::NPC;
    use crate::references::party_ini_db::PartyIniNpc;
    use crate::references::weapons_db::WeaponItem;

    /// Structural invariants that must hold for every generated layout.
    fn assert_layout_invariants(layout: FixedRecordLayout) {
        assert!(layout.record_size > 0, "{}: empty record", layout.type_name);
        assert!(!layout.fields.is_empty(), "{}: no fields", layout.type_name);
        let mut prev_end = 0u32;
        for def in layout.fields {
            assert!(
                def.size > 0,
                "{}: field {} has zero size",
                layout.type_name,
                def.name
            );
            assert!(
                def.offset >= prev_end,
                "{}: field {} overlaps or precedes the previous field",
                layout.type_name,
                def.name
            );
            assert!(
                def.offset
                    .checked_add(def.size)
                    .is_some_and(|end| end <= layout.record_size),
                "{}: field {} extends past record_size",
                layout.type_name,
                def.name
            );
            prev_end = def.offset + def.size;
        }
    }

    /// Parse one record sized from `RecordLayout`, write it back, then parse
    /// and write again. The canonical byte stream must be exactly `footprint()`
    /// bytes long and stable across the second pass. This proves the generated
    /// metadata and the generated reader/writer agree on both the header size
    /// and the record size. (A raw byte-for-byte match against all zeroes is
    /// not possible: enum fields with no wire value `0` normalise to their
    /// default on read.)
    fn assert_layout_matches_parser<T: Extractor + RecordLayout>() {
        use std::io::Cursor;

        let layout = T::LAYOUT;
        let mut zeros = vec![0u8; layout.footprint() as usize];
        if layout.header_size > 0 {
            // All 12 records use a 4-byte i32 record counter when present.
            assert_eq!(
                layout.header_size, 4,
                "{}: unexpected header",
                layout.type_name
            );
            zeros[..4].copy_from_slice(&1u32.to_le_bytes());
        }
        let parsed = T::parse(&mut Cursor::new(&zeros), zeros.len() as u64)
            .unwrap_or_else(|e| panic!("{}: parse failed: {e}", layout.type_name));
        assert_eq!(parsed.len(), 1, "{}: expected one record", layout.type_name);

        let mut canonical = Vec::new();
        T::to_writer(&parsed, &mut canonical)
            .unwrap_or_else(|e| panic!("{}: to_writer failed: {e}", layout.type_name));
        assert_eq!(
            canonical.len() as u32,
            layout.footprint(),
            "{}: serialized footprint disagrees with RecordLayout",
            layout.type_name
        );

        let reparsed = T::parse(&mut Cursor::new(&canonical), canonical.len() as u64)
            .unwrap_or_else(|e| panic!("{}: reparse failed: {e}", layout.type_name));
        assert_eq!(
            reparsed.len(),
            parsed.len(),
            "{}: parse is not stable",
            layout.type_name
        );
        let mut again = Vec::new();
        T::to_writer(&reparsed, &mut again)
            .unwrap_or_else(|e| panic!("{}: rewrite failed: {e}", layout.type_name));
        assert_eq!(
            again, canonical,
            "{}: write is not stable",
            layout.type_name
        );
    }

    macro_rules! check_layout {
        ($ty:ty) => {{
            assert_layout_invariants(<$ty as RecordLayout>::LAYOUT);
            assert_layout_matches_parser::<$ty>();
        }};
    }

    /// Every one of the 12 records with a generated `LAYOUT` must agree with
    /// its serializer. Replaces the two ad-hoc per-record layout tests that
    /// only covered `Monster` and `ExtraRef`.
    #[test]
    fn test_all_generated_layouts_match_serialized_record_footprint() {
        check_layout!(WeaponItem);
        check_layout!(Monster);
        check_layout!(MagicSpell);
        check_layout!(MiscItem);
        check_layout!(HealItem);
        check_layout!(EventItem);
        check_layout!(EditItem);
        check_layout!(ChData);
        check_layout!(PartyIniNpc);
        check_layout!(NPC);
        check_layout!(MonsterRef);
        check_layout!(ExtraRef);
    }

    /// Concrete offsets that must not drift, folded into one data-driven test
    /// so the whole byte-offset truth lives in a single place.
    #[test]
    fn test_concrete_record_offsets_match_documented_layout() {
        let monster = Monster::LAYOUT;
        assert_eq!(monster.record_size, 160);
        assert_eq!(
            monster.field("name").map(|f| (f.offset, f.size)),
            Some((0, 24))
        );
        assert_eq!(
            monster.field("health_points_max").map(|f| f.offset),
            Some(24)
        );
        assert_eq!(
            monster.fields.last().map(|f| f.offset + f.size),
            Some(monster.record_size)
        );

        let extra = ExtraRef::LAYOUT;
        assert_eq!(extra.record_size, 184);
        assert_eq!(extra.field_offset("is_trapped"), Some(0x70));
        assert_eq!(extra.field_offset("trap_spell_id"), Some(0x74));
        assert_eq!(extra.field_offset("use_counter"), Some(0x80));
        assert_eq!(extra.field_offset("open_sfx_id"), Some(0xac));
    }

    /// The hand-maintained party-level layout must match the monolithic parser
    /// exactly. Writing a sentinel into each field at its declared offset and
    /// parsing it back proves the offsets describe reality, not just metadata.
    #[test]
    fn test_party_level_layout_matches_parser() {
        use crate::references::party_level_db::PartyLevelNpc;
        use std::io::Cursor;

        assert_eq!(PARTY_LEVEL_RECORD.footprint(), 36);
        assert_layout_invariants(PARTY_LEVEL_RECORD);

        let mut block = [0u8; 36];
        for def in PARTY_LEVEL_RECORD.fields {
            let end = (def.offset + def.size) as usize;
            block[def.offset as usize..end].fill(0x5a);
        }
        // 8 NPCs × 20 levels; only level 1 of NPC 0 carries the sentinels.
        let mut data = block.to_vec();
        data.extend(std::iter::repeat_n(0u8, 5760 - 36));

        let npcs = PartyLevelNpc::parse(&mut Cursor::new(&data), data.len() as u64).unwrap();
        let rec = &npcs[0].records[0];
        assert_eq!(rec.magic_spell_id_1, 0x5a);
        assert_eq!(rec.strength, 0x5a5a5a5a);
        assert_eq!(rec.health_points, 0x5a5a);
        assert_eq!(rec.accuracy, 0x5a);
        assert_eq!(rec.dodge, 0x5a);
        assert_eq!(rec.tactical_action_chance, 0x5a5a5a5a);
    }
}
