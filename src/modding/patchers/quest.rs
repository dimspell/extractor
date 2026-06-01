//! Field-level patcher for `ExtraInGame/Quest.scr` — pipe-delimited quest
//! journal entries.
//!
//! Hand-written because [`Quest`] uses pipe (`|`) as its delimiter and a
//! literal `"null"` sentinel for absent strings; the `TextRecordPatcher`
//! derive currently bakes the comma delimiter into the parser it generates.
//! Could be folded into the derive once the delimiter becomes a struct attr,
//! but for now it's three small handlers.

use std::io::Cursor;

use crate::modding::error::{ModdingError, Result};
use crate::modding::patcher::{out_of_range, unknown_field, wrong_type, RecordPatcher};
use crate::modding::value::Value;
use crate::references::extractor::Extractor;
use crate::references::quest_scr::Quest;

pub struct QuestPatcher;

impl QuestPatcher {
    pub const FILENAME: &'static str = "Quest.scr";
    pub const RECORD_NAME: &'static str = "Quest";
}

impl RecordPatcher for QuestPatcher {
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
        let mut quests = Quest::parse(&mut cursor, bytes.len() as u64)?;

        let idx = record_id as usize;
        if idx >= quests.len() {
            return Err(out_of_range(Self::RECORD_NAME, record_id, quests.len()));
        }
        let rec = &mut quests[idx];

        match field {
            "id" => {
                return Err(ModdingError::Malformed(format!(
                    "{}.id is positional and cannot be patched",
                    Self::RECORD_NAME
                )));
            }
            "type_id" => match new {
                Value::I64(v) => rec.type_id = (*v) as i32,
                Value::String(s) => match s.trim().parse::<i32>() {
                    Ok(v) => rec.type_id = v,
                    Err(_) => return Err(wrong_type(Self::RECORD_NAME, field, "i32", new)),
                },
                _ => return Err(wrong_type(Self::RECORD_NAME, field, "i32", new)),
            },
            "title" => match new {
                Value::String(s) => rec.title = s.clone(),
                Value::Null => rec.title = "null".to_string(),
                _ => return Err(wrong_type(Self::RECORD_NAME, field, "string", new)),
            },
            "description" => match new {
                Value::String(s) => rec.description = s.clone(),
                Value::Null => rec.description = "null".to_string(),
                _ => return Err(wrong_type(Self::RECORD_NAME, field, "string", new)),
            },
            other => return Err(unknown_field(Self::RECORD_NAME, other)),
        }

        let mut out = Vec::new();
        Quest::to_writer(&quests, &mut out)?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blob() -> Vec<u8> {
        b"1|0|Main Quest|Kill the dragon\r\n2|1|null|null\r\n".to_vec()
    }

    fn parse_back(b: &[u8]) -> Vec<Quest> {
        Quest::parse(&mut Cursor::new(b), b.len() as u64).unwrap()
    }

    #[test]
    fn change_title() {
        let p = QuestPatcher;
        let out = p
            .apply_field(&blob(), 0, "title", &Value::String("New Title".into()))
            .unwrap();
        assert_eq!(parse_back(&out)[0].title, "New Title");
        assert_eq!(parse_back(&out)[1].title, "null"); // row 1 untouched
    }

    #[test]
    fn set_description_to_null_via_string_literal() {
        let p = QuestPatcher;
        let out = p
            .apply_field(&blob(), 0, "description", &Value::String("null".into()))
            .unwrap();
        assert_eq!(parse_back(&out)[0].description, "null");
    }

    #[test]
    fn set_description_to_null_via_value_null() {
        let p = QuestPatcher;
        let out = p
            .apply_field(&blob(), 0, "description", &Value::Null)
            .unwrap();
        assert_eq!(parse_back(&out)[0].description, "null");
    }

    #[test]
    fn change_type_id() {
        let p = QuestPatcher;
        let out = p
            .apply_field(&blob(), 1, "type_id", &Value::I64(2))
            .unwrap();
        assert_eq!(parse_back(&out)[1].type_id, 2);
    }

    #[test]
    fn id_field_rejected() {
        let p = QuestPatcher;
        let err = p
            .apply_field(&blob(), 0, "id", &Value::I64(99))
            .unwrap_err();
        assert!(err.to_string().contains("positional"));
    }

    #[test]
    fn unknown_field_errors() {
        let p = QuestPatcher;
        let err = p
            .apply_field(&blob(), 0, "reward", &Value::I64(0))
            .unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn out_of_range_id_errors() {
        let p = QuestPatcher;
        let err = p
            .apply_field(&blob(), 99, "type_id", &Value::I64(0))
            .unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }
}
