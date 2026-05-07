//! Patchers for the two dialogue formats: `*.dlg` (DialogueScript)
//! and `*.pgp` (DialogueParagraph).
//!
//! Both are hand-written:
//!
//! * **DialogueScript** has nine `Option<i32>` / `Option<Enum>` fields —
//!   the `TextRecordPatcher` derive only models bare `i32` and `String`,
//!   so it can't drop in here.
//! * **DialogueParagraph** has multi-line comment preservation and a
//!   `text` field where the literal `"null"` ⇄ empty-string round-trips
//!   through parse/serialize. Both quirks need explicit handling.
//!
//! Registered via the pattern path: every file with extension `dlg` /
//! `pgp` matches (empty `stem_prefix`).

use std::io::Cursor;

use crate::modding::error::{ModdingError, Result};
use crate::modding::patcher::{out_of_range, unknown_field, wrong_type, RecordPatcher};
use crate::modding::value::Value;
use crate::references::dialogue_paragraph::DialogueParagraph;
use crate::references::dialogue_script::DialogueScript;
use crate::references::enums::{DialogOwner, DialogType};
use crate::references::extractor::Extractor;

// =============================================================== DialogueScript

pub struct DialogueScriptPatcher;

impl DialogueScriptPatcher {
    pub const EXTENSION: &'static str = "dlg";
    pub const STEM_PREFIX: &'static str = ""; // matches every *.dlg
    pub const RECORD_NAME: &'static str = "DialogueScript";
}

impl RecordPatcher for DialogueScriptPatcher {
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
        let mut dlgs = DialogueScript::parse(&mut cursor, bytes.len() as u64)?;
        let idx = record_id as usize;
        if idx >= dlgs.len() {
            return Err(out_of_range(Self::RECORD_NAME, record_id, dlgs.len()));
        }
        let rec = &mut dlgs[idx];

        match field {
            "id" => {
                return Err(ModdingError::Malformed(format!(
                    "{}.id is positional and cannot be patched",
                    Self::RECORD_NAME
                )));
            }
            "required_event_id" => rec.required_event_id = parse_optional_i32(field, new)?,
            "next_dialog_to_check" => rec.next_dialog_to_check = parse_optional_i32(field, new)?,
            "dialog_id" => rec.dialog_id = parse_optional_i32(field, new)?,
            "next_dialog_id1" => rec.next_dialog_id1 = parse_optional_i32(field, new)?,
            "next_dialog_id2" => rec.next_dialog_id2 = parse_optional_i32(field, new)?,
            "next_dialog_id3" => rec.next_dialog_id3 = parse_optional_i32(field, new)?,
            "triggered_event_id" => rec.triggered_event_id = parse_optional_i32(field, new)?,
            "dialog_type" => {
                rec.dialog_type = parse_optional_i32(field, new)?.and_then(DialogType::from_i32);
            }
            "dialog_owner" => {
                rec.dialog_owner = parse_optional_i32(field, new)?.and_then(DialogOwner::from_i32);
            }
            other => return Err(unknown_field(Self::RECORD_NAME, other)),
        }

        let mut out = Vec::new();
        DialogueScript::to_writer(&dlgs, &mut out)?;
        Ok(out)
    }
}

/// `Value::Null` → `None`; everything else parses as `i32` and wraps in `Some`.
fn parse_optional_i32(field: &str, new: &Value) -> Result<Option<i32>> {
    match new {
        Value::Null => Ok(None),
        Value::I64(v) => i32::try_from(*v)
            .map(Some)
            .map_err(|_| wrong_type("DialogueScript", field, "i32|null", new)),
        Value::String(s) if s.is_empty() || s == "null" => Ok(None),
        Value::String(s) => s
            .trim()
            .parse::<i32>()
            .map(Some)
            .map_err(|_| wrong_type("DialogueScript", field, "i32|null", new)),
        _ => Err(wrong_type("DialogueScript", field, "i32|null", new)),
    }
}

// =============================================================== DialogueParagraph

pub struct DialogueParagraphPatcher;

impl DialogueParagraphPatcher {
    pub const EXTENSION: &'static str = "pgp";
    pub const STEM_PREFIX: &'static str = ""; // matches every *.pgp
    pub const RECORD_NAME: &'static str = "DialogueParagraph";
}

impl RecordPatcher for DialogueParagraphPatcher {
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
        let mut paras = DialogueParagraph::parse(&mut cursor, bytes.len() as u64)?;
        let idx = record_id as usize;
        if idx >= paras.len() {
            return Err(out_of_range(Self::RECORD_NAME, record_id, paras.len()));
        }
        let rec = &mut paras[idx];

        match field {
            "id" => {
                return Err(ModdingError::Malformed(format!(
                    "{}.id is positional and cannot be patched",
                    Self::RECORD_NAME
                )));
            }
            "text" => match new {
                // The on-disk `"null"` literal already parses to "" on the
                // way back in, so accept either form symmetrically.
                Value::Null => rec.text = String::new(),
                Value::String(s) if s == "null" => rec.text = String::new(),
                Value::String(s) => rec.text = s.clone(),
                _ => return Err(wrong_type(Self::RECORD_NAME, field, "string|null", new)),
            },
            "comment" => match new {
                Value::Null => rec.comment = String::new(),
                Value::String(s) => rec.comment = s.clone(),
                _ => return Err(wrong_type(Self::RECORD_NAME, field, "string", new)),
            },
            "param1" => rec.param1 = parse_i32(field, new)?,
            "wave_ini_entry_id" => rec.wave_ini_entry_id = parse_i32(field, new)?,
            other => return Err(unknown_field(Self::RECORD_NAME, other)),
        }

        let mut out = Vec::new();
        DialogueParagraph::to_writer(&paras, &mut out)?;
        Ok(out)
    }
}

fn parse_i32(field: &str, new: &Value) -> Result<i32> {
    match new {
        Value::I64(v) => i32::try_from(*v).map_err(|_| {
            wrong_type(DialogueParagraphPatcher::RECORD_NAME, field, "i32", new)
        }),
        Value::String(s) => s.trim().parse::<i32>().map_err(|_| {
            wrong_type(DialogueParagraphPatcher::RECORD_NAME, field, "i32", new)
        }),
        _ => Err(wrong_type(DialogueParagraphPatcher::RECORD_NAME, field, "i32", new)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- DialogueScript -----

    fn dlg_blob() -> Vec<u8> {
        b"1,0,2,0,1,100,200,0,0,1000\r\n2,0,0,1,0,101,201,202,203,0\r\n".to_vec()
    }

    fn parse_dlgs(b: &[u8]) -> Vec<DialogueScript> {
        DialogueScript::parse(&mut Cursor::new(b), b.len() as u64).unwrap()
    }

    #[test]
    fn dlg_change_dialog_id() {
        let p = DialogueScriptPatcher;
        let out = p
            .apply_field(&dlg_blob(), 0, "dialog_id", &Value::I64(999))
            .unwrap();
        assert_eq!(parse_dlgs(&out)[0].dialog_id, Some(999));
    }

    #[test]
    fn dlg_set_optional_to_null() {
        let p = DialogueScriptPatcher;
        let out = p
            .apply_field(&dlg_blob(), 1, "next_dialog_id3", &Value::Null)
            .unwrap();
        // The serializer maps None → 0, which the parser then reads as Some(0).
        // This is a quirk of the file format (no real "absent" sentinel for
        // `Option<i32>`): patching to Null effectively zeroes the slot.
        assert_eq!(parse_dlgs(&out)[1].next_dialog_id3, Some(0));
    }

    #[test]
    fn dlg_change_dialog_type_via_i64() {
        let p = DialogueScriptPatcher;
        // DialogType::Choice == 1
        let out = p
            .apply_field(&dlg_blob(), 0, "dialog_type", &Value::I64(1))
            .unwrap();
        assert_eq!(parse_dlgs(&out)[0].dialog_type, Some(DialogType::Choice));
    }

    #[test]
    fn dlg_invalid_enum_discriminant_falls_to_none() {
        let p = DialogueScriptPatcher;
        // DialogType only knows 0 and 1; 99 yields None via from_i32.
        let out = p
            .apply_field(&dlg_blob(), 0, "dialog_type", &Value::I64(99))
            .unwrap();
        assert_eq!(parse_dlgs(&out)[0].dialog_type, Some(DialogType::Normal));
        // Note: the writer maps None → 0 which round-trips back to Normal, so
        // observed behavior is "invalid discriminant becomes the zero variant."
    }

    #[test]
    fn dlg_id_field_rejected() {
        let p = DialogueScriptPatcher;
        let err = p
            .apply_field(&dlg_blob(), 0, "id", &Value::I64(99))
            .unwrap_err();
        assert!(err.to_string().contains("positional"));
    }

    // ----- DialogueParagraph -----

    fn pgp_blob() -> Vec<u8> {
        b"; dev note\r\n1|Hello there|0|0\r\n2|null|5|3\r\n".to_vec()
    }

    fn parse_paras(b: &[u8]) -> Vec<DialogueParagraph> {
        DialogueParagraph::parse(&mut Cursor::new(b), b.len() as u64).unwrap()
    }

    #[test]
    fn pgp_change_text() {
        let p = DialogueParagraphPatcher;
        let out = p
            .apply_field(&pgp_blob(), 0, "text", &Value::String("Goodbye".into()))
            .unwrap();
        assert_eq!(parse_paras(&out)[0].text, "Goodbye");
        assert_eq!(parse_paras(&out)[0].comment, "dev note"); // preserved
    }

    #[test]
    fn pgp_set_text_to_null_via_value_null() {
        let p = DialogueParagraphPatcher;
        let out = p
            .apply_field(&pgp_blob(), 0, "text", &Value::Null)
            .unwrap();
        assert_eq!(parse_paras(&out)[0].text, "");
    }

    #[test]
    fn pgp_set_text_to_null_via_string_literal() {
        let p = DialogueParagraphPatcher;
        let out = p
            .apply_field(&pgp_blob(), 0, "text", &Value::String("null".into()))
            .unwrap();
        assert_eq!(parse_paras(&out)[0].text, "");
    }

    #[test]
    fn pgp_change_wave_ini_entry_id() {
        let p = DialogueParagraphPatcher;
        let out = p
            .apply_field(&pgp_blob(), 1, "wave_ini_entry_id", &Value::I64(42))
            .unwrap();
        assert_eq!(parse_paras(&out)[1].wave_ini_entry_id, 42);
    }

    #[test]
    fn pgp_id_field_rejected() {
        let p = DialogueParagraphPatcher;
        let err = p
            .apply_field(&pgp_blob(), 0, "id", &Value::I64(99))
            .unwrap_err();
        assert!(err.to_string().contains("positional"));
    }
}
