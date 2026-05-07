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
//! are dotted to navigate the section + index + sub-field:
//!
//! | Path | Type | Notes |
//! |---|---|---|
//! | `header_comments.N` | string | replaces line N of the header block |
//! | `variables.N.name` | string | rename of variable N |
//! | `variables.N.value` | string | new value for variable N |
//! | `map_content.N` | string | replace MAP line N |
//! | `chr_content.N` | string | replace CHR line N |
//! | `npc_content.N` | string | replace NPC line N |
//! | `spr_content.N.sprite_alias` | string | rename sprite alias |
//! | `spr_content.N.sprite_file` | string | swap sprite file |
//! | `wav_content.N` | string | replace WAV line N |
//! | `actions.N.prefix` | string \| null | object prefix (`Pope~setmappos`) |
//! | `actions.N.function_name` | string | rename function call |
//! | `actions.N.parameters` | string | comma-joined param list (replaces all) |
//! | `actions.N.parameters.M` | string | replace one parameter |
//! | `actions.N.raw_content` | string \| null | for control-flow lines (`{`, `}`, `if(...)`) |
//!
//! Indexes are zero-based and must already exist (no insertion). Out-of-range
//! indexes return a `Malformed` error rather than silently growing the vec —
//! a typo'd index in a recorded delta should fail loudly, not corrupt state.
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
use crate::references::event_scr::EventScript;
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
        let script = scripts
            .get_mut(0)
            .ok_or_else(|| ModdingError::Malformed(format!("{}: empty script", Self::RECORD_NAME)))?;

        // Top-level non-vector fields first.
        if field == "id" {
            return Err(ModdingError::Malformed(format!(
                "{}.id is derived from the filename and cannot be patched",
                Self::RECORD_NAME
            )));
        }

        let mut parts = field.split('.');
        let section = parts.next().ok_or_else(|| unknown_field(Self::RECORD_NAME, field))?;

        match section {
            "header_comments" => {
                let idx = take_index(field, &mut parts)?;
                expect_no_more(field, &mut parts)?;
                let s = expect_string(field, new)?;
                set_at(field, &mut script.header_comments, idx, s)?;
            }
            "map_content" => {
                let idx = take_index(field, &mut parts)?;
                expect_no_more(field, &mut parts)?;
                let s = expect_string(field, new)?;
                set_at(field, &mut script.map_content, idx, s)?;
            }
            "chr_content" => {
                let idx = take_index(field, &mut parts)?;
                expect_no_more(field, &mut parts)?;
                let s = expect_string(field, new)?;
                set_at(field, &mut script.chr_content, idx, s)?;
            }
            "npc_content" => {
                let idx = take_index(field, &mut parts)?;
                expect_no_more(field, &mut parts)?;
                let s = expect_string(field, new)?;
                set_at(field, &mut script.npc_content, idx, s)?;
            }
            "wav_content" => {
                let idx = take_index(field, &mut parts)?;
                expect_no_more(field, &mut parts)?;
                let s = expect_string(field, new)?;
                set_at(field, &mut script.wav_content, idx, s)?;
            }
            "variables" => {
                let idx = take_index(field, &mut parts)?;
                let sub = parts
                    .next()
                    .ok_or_else(|| unknown_field(Self::RECORD_NAME, field))?;
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
            "spr_content" => {
                let idx = take_index(field, &mut parts)?;
                let sub = parts
                    .next()
                    .ok_or_else(|| unknown_field(Self::RECORD_NAME, field))?;
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
            "actions" => {
                let idx = take_index(field, &mut parts)?;
                let sub = parts
                    .next()
                    .ok_or_else(|| unknown_field(Self::RECORD_NAME, field))?;
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
                    "parameters" => {
                        // Either `actions.N.parameters` (replace all,
                        // comma-joined) or `actions.N.parameters.M`
                        // (replace one).
                        match parts.next() {
                            None => {
                                let s = expect_string(field, new)?;
                                act.parameters = if s.is_empty() {
                                    Vec::new()
                                } else {
                                    s.split(',').map(|p| p.trim().to_string()).collect()
                                };
                            }
                            Some(m) => {
                                expect_no_more(field, &mut parts)?;
                                let pidx = parse_index(field, m)?;
                                let s = expect_string(field, new)?;
                                set_at(field, &mut act.parameters, pidx, s)?;
                            }
                        }
                    }
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
        _ => Err(wrong_type(EventScriptPatcher::RECORD_NAME, field, "string", new)),
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

fn set_at(field: &str, vec: &mut [String], idx: usize, val: String) -> Result<()> {
    let len = vec.len();
    let slot = vec.get_mut(idx).ok_or_else(|| index_oob(field, "", idx, len))?;
    *slot = val;
    Ok(())
}

fn index_oob(field: &str, section: &str, idx: usize, len: usize) -> ModdingError {
    let where_ = if section.is_empty() { field.to_string() } else { format!("{section} ({field})") };
    ModdingError::Malformed(format!(
        "{}: {where_} index {idx} out of range (have {len})",
        EventScriptPatcher::RECORD_NAME
    ))
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

    #[test]
    fn change_variable_value() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(&sample(), 0, "variables.0.value", &Value::String("99".into()))
            .unwrap();
        assert_eq!(parse(&out).variables[0].value, "99");
    }

    #[test]
    fn change_variable_name() {
        let p = EventScriptPatcher;
        let out = p
            .apply_field(&sample(), 0, "variables.0.name", &Value::String("respawn".into()))
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
        // action 1 is the `{` brace; rewrite to `}`.
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
            .apply_field(&sample(), 0, "map_content.0", &Value::String("map(null)".into()))
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
            .apply_field(&sample(), 1, "variables.0.value", &Value::String("x".into()))
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
            .apply_field(&sample(), 0, "variables.0.bogus", &Value::String("x".into()))
            .unwrap_err();
        assert!(err.to_string().contains("unknown field"), "got: {err}");
    }

    #[test]
    fn out_of_range_index_rejected() {
        let p = EventScriptPatcher;
        let err = p
            .apply_field(&sample(), 0, "variables.99.value", &Value::String("x".into()))
            .unwrap_err();
        assert!(err.to_string().contains("out of range"), "got: {err}");
    }

    #[test]
    fn non_numeric_index_rejected() {
        let p = EventScriptPatcher;
        let err = p
            .apply_field(&sample(), 0, "variables.foo.value", &Value::String("x".into()))
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
    fn full_round_trip_patch_changes_only_target_field() {
        let p = EventScriptPatcher;
        let original = sample();
        let out = p
            .apply_field(&original, 0, "wav_content.0", &Value::String("wav99".into()))
            .unwrap();
        let s = parse(&out);
        assert_eq!(s.wav_content[0], "wav99");
        // Other sections untouched.
        assert_eq!(s.variables[0].name, "spawn");
        assert_eq!(s.spr_content[0].sprite_alias, "Pope");
        assert_eq!(s.actions[0].function_name, "setmappos");
    }
}
