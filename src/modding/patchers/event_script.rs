//! Field-level patcher for `Event*.scr` event scripts.
//!
//! Hand-written because [`EventScript`] is structurally unlike any catalog
//! the derives know about: a single logical record per file, composed of
//! seven heterogenous sections (`[VAR]`, `[MAP]`, `[CHR]`, `[NPC]`, `[SPR]`,
//! `[WAV]`, `[ACT]`) plus a free-form header-comment block. Most sections
//! hold raw lines, but `[VAR]`, `[SPR]`, and `[ACT]` each parse into their
//! own struct shape — `Variable`, `SpriteDefinition`, and `ActionFunction`.
//!
//! ## Addressing model
//!
//! There is exactly one record per file (the file *is* the script; its `id`
//! is derived from the filename, e.g. `Eventcat1.scr` → `id = "cat1"`-ish).
//! So `record_id` is always `0`; anything else is rejected. Field paths
//! are dotted to navigate the section + index + sub-field.
//!
//! ### Element-level paths
//!
//! Address one element of a section's vec. Three operations:
//!
//! | Path & value | Effect |
//! |---|---|
//! | `<section>.N = String` with `N < len` | **replace** element N |
//! | `<section>.N = String` with `N == len` | **append** new element |
//! | `<section>.N = Null` with `N < len` | **delete** element N |
//!
//! For struct-shaped sections (`variables`, `spr_content`, `actions`), the
//! `String` is parsed in the on-disk format for that section:
//!
//! | Section | Insert/replace string format |
//! |---|---|
//! | `variables` | `name=value` (must contain `=`) |
//! | `spr_content` | `alias(file)` or just `alias` |
//! | `actions` | any line — function call, brace, `if(...)`, etc. |
//!
//! ### Subfield paths
//!
//! Address one field inside an existing struct element. The element must
//! already exist (insertion via subfield is not supported — use the bare
//! element path with the on-disk format string instead).
//!
//! | Path | Type |
//! |---|---|
//! | `variables.N.name` / `variables.N.value` | string |
//! | `spr_content.N.sprite_alias` / `spr_content.N.sprite_file` | string |
//! | `actions.N.prefix` / `actions.N.raw_content` | string \| null |
//! | `actions.N.function_name` | string |
//! | `actions.N.parameters` | string (comma-joined, replaces all) |
//! | `actions.N.parameters.M` | string (insert/replace) or null (delete) |
//!
//! Indexes are zero-based. Out-of-range writes (other than the one-past-end
//! append) and out-of-range deletes return `Malformed` rather than silently
//! growing or no-op'ing — a typo'd index in a recorded delta should fail
//! loudly, not corrupt state.
//!
//! ## Why not derive?
//!
//! `RecordPatcher` derives a flat field map from a record-shaped struct;
//! `EventScript` is essentially seven nested vectors of mixed types. There is
//! no path through the existing derive that produces this dotted-path API
//! without inventing a parallel mechanism.

use std::io::Cursor;

use crate::modding::error::{ModdingError, Result};
use crate::modding::patcher::{unknown_field, wrong_type, RecordPatcher};
use crate::modding::value::Value;
use crate::references::event_scr::{ActionFunction, EventScript, SpriteDefinition, Variable};
use crate::references::extractor::Extractor;

pub struct EventScriptPatcher;

impl EventScriptPatcher {
    /// Every `*.scr` whose stem starts with `event`. `Quest.scr` and
    /// `Message.scr` are registered explicitly with exact names so the
    /// exact-filename map wins over this pattern.
    pub const EXTENSION: &'static str = "scr";
    pub const STEM_PREFIX: &'static str = "event";
    pub const RECORD_NAME: &'static str = "EventScript";
}

impl RecordPatcher for EventScriptPatcher {
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
        if record_id != 0 {
            return Err(ModdingError::Malformed(format!(
                "{}: record_id must be 0 (one script per file), got {record_id}",
                Self::RECORD_NAME
            )));
        }

        let mut cursor = Cursor::new(bytes);
        let mut scripts = EventScript::parse(&mut cursor, bytes.len() as u64)?;
        let script = scripts.get_mut(0).ok_or_else(|| {
            ModdingError::Malformed(format!("{}: empty script", Self::RECORD_NAME))
        })?;

        if field == "id" {
            return Err(ModdingError::Malformed(format!(
                "{}.id is derived from the filename and cannot be patched",
                Self::RECORD_NAME
            )));
        }

        let mut parts = field.split('.');
        let section = parts
            .next()
            .ok_or_else(|| unknown_field(Self::RECORD_NAME, field))?;
        let idx = take_index(field, &mut parts)?;
        let sub = parts.next();

        match (section, sub) {
            // ---- string-line sections ----
            ("header_comments", None) => {
                string_element_op(field, &mut script.header_comments, idx, new)?
            }
            ("map_content", None) => string_element_op(field, &mut script.map_content, idx, new)?,
            ("chr_content", None) => string_element_op(field, &mut script.chr_content, idx, new)?,
            ("npc_content", None) => string_element_op(field, &mut script.npc_content, idx, new)?,
            ("wav_content", None) => string_element_op(field, &mut script.wav_content, idx, new)?,

            // ---- variables: insert/replace/delete a whole entry ----
            ("variables", None) => {
                if matches!(new, Value::Null) {
                    delete_at(field, "variables", &mut script.variables, idx)?;
                } else {
                    let raw = expect_string(field, new)?;
                    let var = parse_variable_line(field, &raw)?;
                    insert_or_replace(field, "variables", &mut script.variables, idx, var)?;
                }
            }
            ("variables", Some(sub)) => {
                expect_no_more(field, &mut parts)?;
                let len = script.variables.len();
                let var = script
                    .variables
                    .get_mut(idx)
                    .ok_or_else(|| index_oob(field, "variables", idx, len))?;
                match sub {
                    "name" => var.name = expect_string(field, new)?,
                    "value" => var.value = expect_string(field, new)?,
                    _ => return Err(unknown_field(Self::RECORD_NAME, field)),
                }
            }

            // ---- spr_content ----
            ("spr_content", None) => {
                if matches!(new, Value::Null) {
                    delete_at(field, "spr_content", &mut script.spr_content, idx)?;
                } else {
                    let raw = expect_string(field, new)?;
                    let spr = SpriteDefinition::parse(&raw);
                    insert_or_replace(field, "spr_content", &mut script.spr_content, idx, spr)?;
                }
            }
            ("spr_content", Some(sub)) => {
                expect_no_more(field, &mut parts)?;
                let len = script.spr_content.len();
                let spr = script
                    .spr_content
                    .get_mut(idx)
                    .ok_or_else(|| index_oob(field, "spr_content", idx, len))?;
                match sub {
                    "sprite_alias" => spr.sprite_alias = expect_string(field, new)?,
                    "sprite_file" => spr.sprite_file = expect_string(field, new)?,
                    _ => return Err(unknown_field(Self::RECORD_NAME, field)),
                }
            }

            // ---- actions ----
            ("actions", None) => {
                if matches!(new, Value::Null) {
                    delete_at(field, "actions", &mut script.actions, idx)?;
                } else {
                    let raw = expect_string(field, new)?;
                    let act = ActionFunction::parse(&raw);
                    insert_or_replace(field, "actions", &mut script.actions, idx, act)?;
                }
            }
            ("actions", Some(sub)) => {
                let len = script.actions.len();
                let act = script
                    .actions
                    .get_mut(idx)
                    .ok_or_else(|| index_oob(field, "actions", idx, len))?;
                match sub {
                    "prefix" => {
                        expect_no_more(field, &mut parts)?;
                        act.prefix = expect_optional_string(field, new)?;
                    }
                    "function_name" => {
                        expect_no_more(field, &mut parts)?;
                        act.function_name = expect_string(field, new)?;
                    }
                    "raw_content" => {
                        expect_no_more(field, &mut parts)?;
                        act.raw_content = expect_optional_string(field, new)?;
                    }
                    "parameters" => match parts.next() {
                        // `actions.N.parameters` — replace all (comma-joined).
                        None => {
                            let s = expect_string(field, new)?;
                            act.parameters = if s.is_empty() {
                                Vec::new()
                            } else {
                                s.split(',').map(|p| p.trim().to_string()).collect()
                            };
                        }
                        // `actions.N.parameters.M` — single-param op.
                        Some(m) => {
                            expect_no_more(field, &mut parts)?;
                            let pidx = parse_index(field, m)?;
                            if matches!(new, Value::Null) {
                                delete_at(field, "parameters", &mut act.parameters, pidx)?;
                            } else {
                                let s = expect_string(field, new)?;
                                insert_or_replace(
                                    field,
                                    "parameters",
                                    &mut act.parameters,
                                    pidx,
                                    s,
                                )?;
                            }
                        }
                    },
                    _ => return Err(unknown_field(Self::RECORD_NAME, field)),
                }
            }

            _ => return Err(unknown_field(Self::RECORD_NAME, field)),
        }

        let mut out = Vec::new();
        EventScript::to_writer(&scripts, &mut out)?;
        Ok(out)
    }
}

// =================================================================== helpers

fn take_index(field: &str, parts: &mut std::str::Split<'_, char>) -> Result<usize> {
    let raw = parts
        .next()
        .ok_or_else(|| unknown_field(EventScriptPatcher::RECORD_NAME, field))?;
    parse_index(field, raw)
}

fn parse_index(field: &str, raw: &str) -> Result<usize> {
    raw.parse::<usize>().map_err(|_| {
        ModdingError::Malformed(format!(
            "{}.{field}: expected numeric index, got `{raw}`",
            EventScriptPatcher::RECORD_NAME
        ))
    })
}

fn expect_no_more(field: &str, parts: &mut std::str::Split<'_, char>) -> Result<()> {
    if parts.next().is_some() {
        return Err(ModdingError::Malformed(format!(
            "{}.{field}: trailing path components are not supported",
            EventScriptPatcher::RECORD_NAME
        )));
    }
    Ok(())
}

fn expect_string(field: &str, new: &Value) -> Result<String> {
    match new {
        Value::String(s) => Ok(s.clone()),
        Value::Null => Ok(String::new()),
        _ => Err(wrong_type(
            EventScriptPatcher::RECORD_NAME,
            field,
            "string",
            new,
        )),
    }
}

fn expect_optional_string(field: &str, new: &Value) -> Result<Option<String>> {
    match new {
        Value::Null => Ok(None),
        Value::String(s) => Ok(Some(s.clone())),
        _ => Err(wrong_type(
            EventScriptPatcher::RECORD_NAME,
            field,
            "string|null",
            new,
        )),
    }
}

/// Element-level op for plain string vectors (`header_comments`, `map_content`, …).
/// `Null` deletes; any string value inserts (when `idx == len`) or replaces.
fn string_element_op(field: &str, vec: &mut Vec<String>, idx: usize, new: &Value) -> Result<()> {
    if matches!(new, Value::Null) {
        delete_at(field, "", vec, idx)
    } else {
        let s = expect_string(field, new)?;
        insert_or_replace(field, "", vec, idx, s)
    }
}

/// Replace the element at `idx`, or append when `idx == vec.len()`.
/// Anything past the end is rejected.
fn insert_or_replace<T>(
    field: &str,
    section: &str,
    vec: &mut Vec<T>,
    idx: usize,
    val: T,
) -> Result<()> {
    let len = vec.len();
    if idx == len {
        vec.push(val);
        Ok(())
    } else if idx < len {
        vec[idx] = val;
        Ok(())
    } else {
        Err(index_oob(field, section, idx, len))
    }
}

fn delete_at<T>(field: &str, section: &str, vec: &mut Vec<T>, idx: usize) -> Result<()> {
    let len = vec.len();
    if idx >= len {
        return Err(index_oob(field, section, idx, len));
    }
    vec.remove(idx);
    Ok(())
}

fn index_oob(field: &str, section: &str, idx: usize, len: usize) -> ModdingError {
    let where_ = if section.is_empty() {
        field.to_string()
    } else {
        format!("{section} ({field})")
    };
    ModdingError::Malformed(format!(
        "{}: {where_} index {idx} out of range (have {len})",
        EventScriptPatcher::RECORD_NAME
    ))
}

/// Parse `name=value` for a `[VAR]` insert. Requires a literal `=`; the
/// on-disk parser silently skips lines without one, but here that's
/// almost certainly a malformed delta and we should surface it.
fn parse_variable_line(field: &str, raw: &str) -> Result<Variable> {
    let trimmed = raw.trim();
    let eq = trimmed.find('=').ok_or_else(|| {
        ModdingError::Malformed(format!(
            "{}.{field}: variable insert requires `name=value`, got `{trimmed}`",
            EventScriptPatcher::RECORD_NAME
        ))
    })?;
    Ok(Variable {
        name: trimmed[..eq].trim().to_string(),
        value: trimmed[eq + 1..].trim().to_string(),
    })
}

// ===================================================================== tests

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<u8> {
        // header + every section populated, with one ACT call and one
        // control-flow brace so we exercise raw_content as well.
        let mut s = String::new();
        s.push_str("; Authored by tests\r\n");
        s.push_str("\r\n");
        s.push_str("[VAR]\r\n\r\n");
        s.push_str("spawn=5\r\n");
        s.push_str("\r\n");
        s.push_str("[MAP]\r\n\r\n");
        s.push_str("map_cmd1\r\n");
        s.push_str("[CHR]\r\n\r\n");
        s.push_str("hero\r\n");
        s.push_str("[NPC]\r\n\r\n");
        s.push_str("npc1\r\n");
        s.push_str("[SPR]\r\n\r\n");
        s.push_str("Pope(PopeBlessing.spr)\r\n");
        s.push_str("[WAV]\r\n\r\n");
        s.push_str("wav1\r\n");
        s.push_str("[ACT]\r\n");
        s.push_str("Pope~setmappos(10,20)\r\n");
        s.push_str("{\r\n");
        s.into_bytes()
    }

    fn parse(b: &[u8]) -> EventScript {
        EventScript::parse(&mut Cursor::new(b), b.len() as u64)
            .unwrap()
            .remove(0)
    }

    // --------------------------------------------------------- replace (existing)

    #[test]
    fn change_variable_value() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(
                &sample(),
                0,
                "variables.0.value",
                &Value::String("99".into()),
            )
            .unwrap();
        assert_eq!(parse(&out).variables[0].value, "99");
    }

    #[test]
    fn change_variable_name() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(
                &sample(),
                0,
                "variables.0.name",
                &Value::String("respawn".into()),
            )
            .unwrap();
        assert_eq!(parse(&out).variables[0].name, "respawn");
    }

    #[test]
    fn change_sprite_file() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(
                &sample(),
                0,
                "spr_content.0.sprite_file",
                &Value::String("PopeWave.spr".into()),
            )
            .unwrap();
        let s = parse(&out);
        assert_eq!(s.spr_content[0].sprite_alias, "Pope");
        assert_eq!(s.spr_content[0].sprite_file, "PopeWave.spr");
    }

    #[test]
    fn change_action_function_name() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(
                &sample(),
                0,
                "actions.0.function_name",
                &Value::String("setpos".into()),
            )
            .unwrap();
        let s = parse(&out);
        assert_eq!(s.actions[0].function_name, "setpos");
        assert_eq!(s.actions[0].prefix.as_deref(), Some("Pope"));
        assert_eq!(s.actions[0].parameters, vec!["10", "20"]);
    }

    #[test]
    fn change_single_action_parameter() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(
                &sample(),
                0,
                "actions.0.parameters.1",
                &Value::String("99".into()),
            )
            .unwrap();
        assert_eq!(parse(&out).actions[0].parameters, vec!["10", "99"]);
    }

    #[test]
    fn replace_full_parameter_list() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(
                &sample(),
                0,
                "actions.0.parameters",
                &Value::String("a, b, c".into()),
            )
            .unwrap();
        assert_eq!(parse(&out).actions[0].parameters, vec!["a", "b", "c"]);
    }

    #[test]
    fn clear_action_prefix_via_null() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(&sample(), 0, "actions.0.prefix", &Value::Null)
            .unwrap();
        assert_eq!(parse(&out).actions[0].prefix, None);
    }

    #[test]
    fn change_raw_control_flow_line() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(
                &sample(),
                0,
                "actions.1.raw_content",
                &Value::String("}".into()),
            )
            .unwrap();
        assert_eq!(parse(&out).actions[1].raw_content.as_deref(), Some("}"));
    }

    #[test]
    fn change_map_content_line() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(
                &sample(),
                0,
                "map_content.0",
                &Value::String("map(null)".into()),
            )
            .unwrap();
        assert_eq!(parse(&out).map_content[0], "map(null)");
    }

    #[test]
    fn change_header_comment() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(
                &sample(),
                0,
                "header_comments.0",
                &Value::String("; Patched by tests".into()),
            )
            .unwrap();
        assert_eq!(parse(&out).header_comments[0], "; Patched by tests");
    }

    // --------------------------------------------------------- append

    #[test]
    fn append_string_line_at_end_of_section() {
        let p = EventScriptPatcher;
        // sample has 1 wav line; index 1 == len → append.
        let out = p
            .apply_field(&sample(), 0, "wav_content.1", &Value::String("wav2".into()))
            .unwrap();
        assert_eq!(parse(&out).wav_content, vec!["wav1", "wav2"]);
    }

    #[test]
    fn append_variable_via_name_equals_value() {
        let p = EventScriptPatcher;
        // sample has 1 variable; index 1 == len → append.
        let out = p
            .apply_field(
                &sample(),
                0,
                "variables.1",
                &Value::String("kills=0".into()),
            )
            .unwrap();
        let s = parse(&out);
        assert_eq!(s.variables.len(), 2);
        assert_eq!(s.variables[1].name, "kills");
        assert_eq!(s.variables[1].value, "0");
    }

    #[test]
    fn append_sprite_definition() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(
                &sample(),
                0,
                "spr_content.1",
                &Value::String("King(KingWave.spr)".into()),
            )
            .unwrap();
        let s = parse(&out);
        assert_eq!(s.spr_content.len(), 2);
        assert_eq!(s.spr_content[1].sprite_alias, "King");
        assert_eq!(s.spr_content[1].sprite_file, "KingWave.spr");
    }

    #[test]
    fn append_action_function_call() {
        let p = EventScriptPatcher;
        // sample.actions.len() == 2 → index 2 appends.
        let out = p
            .apply_field(
                &sample(),
                0,
                "actions.2",
                &Value::String("King~saypopup(42,1)".into()),
            )
            .unwrap();
        let s = parse(&out);
        assert_eq!(s.actions.len(), 3);
        assert_eq!(s.actions[2].prefix.as_deref(), Some("King"));
        assert_eq!(s.actions[2].function_name, "saypopup");
        assert_eq!(s.actions[2].parameters, vec!["42", "1"]);
    }

    #[test]
    fn append_action_parameter() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(
                &sample(),
                0,
                "actions.0.parameters.2",
                &Value::String("30".into()),
            )
            .unwrap();
        assert_eq!(parse(&out).actions[0].parameters, vec!["10", "20", "30"]);
    }

    // --------------------------------------------------------- delete

    #[test]
    fn delete_string_line() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(&sample(), 0, "wav_content.0", &Value::Null)
            .unwrap();
        assert!(parse(&out).wav_content.is_empty());
    }

    #[test]
    fn delete_variable() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(&sample(), 0, "variables.0", &Value::Null)
            .unwrap();
        assert!(parse(&out).variables.is_empty());
    }

    #[test]
    fn delete_sprite_definition() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(&sample(), 0, "spr_content.0", &Value::Null)
            .unwrap();
        assert!(parse(&out).spr_content.is_empty());
    }

    #[test]
    fn delete_action() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(&sample(), 0, "actions.0", &Value::Null)
            .unwrap();
        let s = parse(&out);
        assert_eq!(s.actions.len(), 1);
        // The remaining action is the `{` brace that originally lived at idx 1.
        assert_eq!(s.actions[0].raw_content.as_deref(), Some("{"));
    }

    #[test]
    fn delete_single_action_parameter() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(&sample(), 0, "actions.0.parameters.0", &Value::Null)
            .unwrap();
        assert_eq!(parse(&out).actions[0].parameters, vec!["20"]);
    }

    #[test]
    fn delete_then_append_round_trips() {
        let p = EventScriptPatcher;
        let after_delete = p
            .apply_field(&sample(), 0, "variables.0", &Value::Null)
            .unwrap();
        let after_append = p
            .apply_field(
                &after_delete,
                0,
                "variables.0",
                &Value::String("respawn=10".into()),
            )
            .unwrap();
        let s = parse(&after_append);
        assert_eq!(s.variables.len(), 1);
        assert_eq!(s.variables[0].name, "respawn");
        assert_eq!(s.variables[0].value, "10");
    }

    // --------------------------------------------------------- error paths

    #[test]
    fn id_field_rejected() {
        let p = EventScriptPatcher;
        let err = p
            .apply_field(&sample(), 0, "id", &Value::I64(7))
            .unwrap_err();
        assert!(err.to_string().contains("filename"), "got: {err}");
    }

    #[test]
    fn nonzero_record_id_rejected() {
        let p = EventScriptPatcher;
        let err = p
            .apply_field(
                &sample(),
                1,
                "variables.0.value",
                &Value::String("x".into()),
            )
            .unwrap_err();
        assert!(err.to_string().contains("record_id"), "got: {err}");
    }

    #[test]
    fn unknown_section_rejected() {
        let p = EventScriptPatcher;
        let err = p
            .apply_field(&sample(), 0, "ghosts.0", &Value::String("x".into()))
            .unwrap_err();
        assert!(err.to_string().contains("unknown field"), "got: {err}");
    }

    #[test]
    fn unknown_subfield_rejected() {
        let p = EventScriptPatcher;
        let err = p
            .apply_field(
                &sample(),
                0,
                "variables.0.bogus",
                &Value::String("x".into()),
            )
            .unwrap_err();
        assert!(err.to_string().contains("unknown field"), "got: {err}");
    }

    #[test]
    fn write_past_end_rejected() {
        let p = EventScriptPatcher;
        // sample.variables.len() == 1 → idx 99 is neither replace (idx<len)
        // nor append (idx==len), so reject.
        let err = p
            .apply_field(&sample(), 0, "variables.99", &Value::String("x=1".into()))
            .unwrap_err();
        assert!(err.to_string().contains("out of range"), "got: {err}");
    }

    #[test]
    fn delete_past_end_rejected() {
        let p = EventScriptPatcher;
        let err = p
            .apply_field(&sample(), 0, "variables.99", &Value::Null)
            .unwrap_err();
        assert!(err.to_string().contains("out of range"), "got: {err}");
    }

    #[test]
    fn out_of_range_subfield_rejected() {
        let p = EventScriptPatcher;
        let err = p
            .apply_field(
                &sample(),
                0,
                "variables.99.value",
                &Value::String("x".into()),
            )
            .unwrap_err();
        assert!(err.to_string().contains("out of range"), "got: {err}");
    }

    #[test]
    fn non_numeric_index_rejected() {
        let p = EventScriptPatcher;
        let err = p
            .apply_field(
                &sample(),
                0,
                "variables.foo.value",
                &Value::String("x".into()),
            )
            .unwrap_err();
        assert!(err.to_string().contains("numeric index"), "got: {err}");
    }

    #[test]
    fn wrong_value_type_rejected() {
        let p = EventScriptPatcher;
        let err = p
            .apply_field(&sample(), 0, "variables.0.value", &Value::I64(5))
            .unwrap_err();
        assert!(err.to_string().contains("expected string"), "got: {err}");
    }

    #[test]
    fn variable_insert_without_equals_rejected() {
        let p = EventScriptPatcher;
        let err = p
            .apply_field(&sample(), 0, "variables.1", &Value::String("kills".into()))
            .unwrap_err();
        assert!(err.to_string().contains("name=value"), "got: {err}");
    }

    #[test]
    fn full_round_trip_patch_changes_only_target_field() {
        let p = EventScriptPatcher;
        let original = sample();
        let out = p
            .apply_field(
                &original,
                0,
                "wav_content.0",
                &Value::String("wav99".into()),
            )
            .unwrap();
        let s = parse(&out);
        assert_eq!(s.wav_content[0], "wav99");
        assert_eq!(s.variables[0].name, "spawn");
        assert_eq!(s.spr_content[0].sprite_alias, "Pope");
        assert_eq!(s.actions[0].function_name, "setmappos");
    }
}
