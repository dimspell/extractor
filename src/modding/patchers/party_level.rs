//! Field-level patcher for `NpcInGame/PrtLevel.db` — the per-NPC
//! per-level character progression table.
//!
//! Hand-written rather than generated because `PartyLevelNpc` has a
//! fixed 8 × 20 × 36-byte shape with no per-field `#[extractor(...)]`
//! attributes for the derive to walk: parsing happens monolithically
//! inside [`PartyLevelNpc::parse`].
//!
//! ## Addressing
//!
//! The trait gives us one `record_id: u32` but the native shape is
//! `(npc_index, level)`. We pack:
//!
//! ```text
//! record_id = npc_index * LEVELS_PER_NPC + (level - 1)
//! ```
//!
//! so IDs `0..160` index every level row in document order. This
//! matches how the GUI's flat editor list already enumerates them,
//! so a recorded `FieldDelta { record_id: 22, .. }` means "NPC 1,
//! level 3" without any callers needing to know the packing.
//!
//! `level` itself is rejected as positional (it's derived from the
//! block index during parse, not stored on the wire).

use std::io::Cursor;

use crate::modding::error::{ModdingError, Result};
use crate::modding::patcher::{out_of_range, unknown_field, wrong_type, RecordPatcher};
use crate::modding::value::Value;
use crate::references::extractor::Extractor;
use crate::references::party_level_db::PartyLevelNpc;

pub struct PartyLevelDbPatcher;

impl PartyLevelDbPatcher {
    pub const FILENAME: &'static str = "PrtLevel.db";
    pub const RECORD_NAME: &'static str = "PartyLevelRecord";
    pub const NPCS: u32 = 8;
    pub const LEVELS_PER_NPC: u32 = 20;
}

impl RecordPatcher for PartyLevelDbPatcher {
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
        let mut npcs = PartyLevelNpc::parse(&mut cursor, bytes.len() as u64)?;

        let total = Self::NPCS * Self::LEVELS_PER_NPC;
        if record_id >= total {
            return Err(out_of_range(Self::RECORD_NAME, record_id, total as usize));
        }
        let npc_idx = (record_id / Self::LEVELS_PER_NPC) as usize;
        let lvl_idx = (record_id % Self::LEVELS_PER_NPC) as usize;

        // The struct guarantees these by construction (parse always
        // produces 8 NPCs × 20 levels), but defend anyway in case a
        // future format change loosens that.
        let rec = npcs
            .get_mut(npc_idx)
            .and_then(|n| n.records.get_mut(lvl_idx))
            .ok_or_else(|| out_of_range(Self::RECORD_NAME, record_id, total as usize))?;

        match field {
            "strength" => set_u32(&mut rec.strength, field, new)?,
            "constitution" => set_u32(&mut rec.constitution, field, new)?,
            "wisdom" => set_u32(&mut rec.wisdom, field, new)?,
            "health_points" => set_u16(&mut rec.health_points, field, new)?,
            "mana_points" => set_u16(&mut rec.mana_points, field, new)?,
            "agility" => set_u32(&mut rec.agility, field, new)?,
            "attack" => set_u32(&mut rec.attack, field, new)?,
            "mana_recharge" => set_u32(&mut rec.mana_recharge, field, new)?,
            "defense" => set_u16(&mut rec.defense, field, new)?,
            "level" | "npc_index" => {
                return Err(ModdingError::Malformed(format!(
                    "{}.{} is positional and cannot be patched",
                    Self::RECORD_NAME,
                    field
                )));
            }
            other => return Err(unknown_field(Self::RECORD_NAME, other)),
        }

        let mut out = Vec::with_capacity(bytes.len());
        PartyLevelNpc::to_writer(&npcs, &mut out)?;
        Ok(out)
    }
}

fn set_u32(slot: &mut u32, field: &str, new: &Value) -> Result<()> {
    *slot = parse_numeric::<u32>(field, new)?;
    Ok(())
}

fn set_u16(slot: &mut u16, field: &str, new: &Value) -> Result<()> {
    *slot = parse_numeric::<u16>(field, new)?;
    Ok(())
}

fn parse_numeric<T>(field: &str, new: &Value) -> Result<T>
where
    T: std::str::FromStr + TryFrom<i64>,
{
    let expected = std::any::type_name::<T>()
        .rsplit("::")
        .next()
        .unwrap_or("int");
    match new {
        Value::I64(v) => T::try_from(*v)
            .map_err(|_| wrong_type(PartyLevelDbPatcher::RECORD_NAME, field, expected, new)),
        Value::String(s) => s
            .trim()
            .parse::<T>()
            .map_err(|_| wrong_type(PartyLevelDbPatcher::RECORD_NAME, field, expected, new)),
        _ => Err(wrong_type(
            PartyLevelDbPatcher::RECORD_NAME,
            field,
            expected,
            new,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::party_level_db::PartyLevelNpc;
    use std::io::Cursor;

    fn empty_file() -> Vec<u8> {
        // 8 NPCs × 20 levels × 36 bytes = 5760 bytes, all zero.
        vec![0u8; 5760]
    }

    fn parse_back(b: &[u8]) -> Vec<PartyLevelNpc> {
        PartyLevelNpc::parse(&mut Cursor::new(b), b.len() as u64).unwrap()
    }

    #[test]
    fn record_id_zero_writes_npc0_level1() {
        let p = PartyLevelDbPatcher;
        let out = p
            .apply_field(&empty_file(), 0, "strength", &Value::I64(42))
            .unwrap();
        let npcs = parse_back(&out);
        assert_eq!(npcs[0].records[0].strength, 42);
        assert_eq!(npcs[0].records[1].strength, 0);
    }

    #[test]
    fn record_id_packs_npc_and_level() {
        let p = PartyLevelDbPatcher;
        // NPC 3, level 7 → 3*20 + (7-1) = 66.
        let out = p
            .apply_field(&empty_file(), 66, "strength", &Value::I64(123))
            .unwrap();
        let npcs = parse_back(&out);
        assert_eq!(npcs[3].records[6].strength, 123);
        // Neighbors untouched.
        assert_eq!(npcs[3].records[5].strength, 0);
        assert_eq!(npcs[3].records[7].strength, 0);
        assert_eq!(npcs[2].records[6].strength, 0);
    }

    #[test]
    fn last_record_id_is_npc7_level20() {
        let p = PartyLevelDbPatcher;
        let out = p
            .apply_field(&empty_file(), 159, "defense", &Value::I64(99))
            .unwrap();
        assert_eq!(parse_back(&out)[7].records[19].defense, 99);
    }

    #[test]
    fn out_of_range_id_errors() {
        let p = PartyLevelDbPatcher;
        let err = p
            .apply_field(&empty_file(), 160, "strength", &Value::I64(1))
            .unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn u16_field_via_string() {
        let p = PartyLevelDbPatcher;
        let out = p
            .apply_field(
                &empty_file(),
                0,
                "health_points",
                &Value::String("250".into()),
            )
            .unwrap();
        assert_eq!(parse_back(&out)[0].records[0].health_points, 250);
    }

    #[test]
    fn out_of_u16_range_errors() {
        let p = PartyLevelDbPatcher;
        let err = p
            .apply_field(&empty_file(), 0, "health_points", &Value::I64(99_999))
            .unwrap_err();
        assert!(err.to_string().contains("expected u16"));
    }

    #[test]
    fn negative_value_into_u32_errors() {
        let p = PartyLevelDbPatcher;
        let err = p
            .apply_field(&empty_file(), 0, "strength", &Value::I64(-1))
            .unwrap_err();
        assert!(err.to_string().contains("expected u32"));
    }

    #[test]
    fn level_field_rejected() {
        let p = PartyLevelDbPatcher;
        let err = p
            .apply_field(&empty_file(), 0, "level", &Value::I64(5))
            .unwrap_err();
        assert!(err.to_string().contains("positional"));
    }

    #[test]
    fn npc_index_field_rejected() {
        let p = PartyLevelDbPatcher;
        let err = p
            .apply_field(&empty_file(), 0, "npc_index", &Value::I64(2))
            .unwrap_err();
        assert!(err.to_string().contains("positional"));
    }

    #[test]
    fn unknown_field_errors() {
        let p = PartyLevelDbPatcher;
        let err = p
            .apply_field(&empty_file(), 0, "luck", &Value::I64(1))
            .unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn round_trip_preserves_unrelated_records() {
        // Write one field deep into the file; verify the rest stays
        // bit-for-bit identical (proves we don't smear any padding).
        let p = PartyLevelDbPatcher;
        let original = empty_file();
        let out = p
            .apply_field(&original, 100, "agility", &Value::I64(7))
            .unwrap();
        assert_eq!(out.len(), original.len());
        // Every byte should still be zero except where we wrote.
        // Block 100 starts at offset 100 * 36 = 3600; agility lives
        // at sub-offset 4+4+4+4+2+2 = 20 within the block as a u32,
        // so bytes 3620..3624 carry `7u32.to_le_bytes() == [7, 0, 0, 0]`.
        let mut expected = vec![0u8; original.len()];
        expected[3620..3624].copy_from_slice(&7u32.to_le_bytes());
        assert_eq!(out, expected);
    }
}
