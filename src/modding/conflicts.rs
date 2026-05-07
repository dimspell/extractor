//! Detect conflicts between enabled mods.
//!
//! A conflict is two or more mods touching the *same destination* in
//! incompatible ways. The apply engine resolves these silently with
//! last-writer-wins; this module surfaces them so the user can see what
//! would be overridden and decide whether to reorder.
//!
//! ## Kinds
//!
//! * [`ConflictKind::Field`] — multiple mods write the same `(file, record_id,
//!   field)` with different `new` values. Soft: load-order resolves it.
//! * [`ConflictKind::Binary`] — multiple mods carry [`ChangeOp::BinaryDelta`]
//!   for the same file. Hard: only one survives because each delta is
//!   applied to the *vanilla* bytes (see `apply.rs`).
//! * [`ConflictKind::FileWhole`] — multiple mods carry `FileReplace` /
//!   `FileAdd` / `FileDelete` for the same file. Hard for the same reason.
//!
//! Same-mod repeated edits are *not* conflicts; flatten passes (see
//! [`ChangeLog::flatten_field_deltas`]) collapse intra-mod redundancy.
//!
//! [`ChangeOp::BinaryDelta`]: super::change::ChangeOp::BinaryDelta
//! [`ChangeLog::flatten_field_deltas`]: super::changelog::ChangeLog::flatten_field_deltas

use std::collections::BTreeMap;

use super::apply::ModEntry;
use super::change::ChangeOp;
use super::resolution::{FieldKey, ResolutionMap};
use super::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictKind {
    /// Same `(file, record_id, field)` written by 2+ mods with different
    /// values. Resolvable by load order.
    Field {
        record_id: u32,
        field: String,
    },
    /// Same file targeted by 2+ binary deltas across mods.
    Binary,
    /// Same file targeted by 2+ whole-file ops (replace/add/delete) across mods.
    FileWhole,
}

/// One contender in a conflict — a single mod's contribution to that key.
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictParticipant {
    pub mod_id: String,
    /// The op variant name (`FieldDelta`, `BinaryDelta`, `FileReplace`, ...).
    pub op: &'static str,
    /// For `Field` conflicts, the new value this mod proposes. `None` for
    /// non-field conflicts.
    pub field_new: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    pub file_path: String,
    pub kind: ConflictKind,
    /// Participants in load order — last entry wins under apply.
    pub participants: Vec<ConflictParticipant>,
    /// `Some(mod_slug)` when a per-field pin has been set for this conflict;
    /// always `None` for hard (Binary/FileWhole) conflicts.
    pub pinned_to: Option<String>,
}

impl Conflict {
    /// The mod_id of the mod whose value the apply engine would write —
    /// honors any pin, otherwise falls back to load order (last).
    pub fn winner(&self) -> &str {
        if let Some(pin) = self.pinned_to.as_deref() {
            // Make sure the pinned mod is actually a participant (otherwise
            // the pin is stale and apply will fall through to load order).
            if self.participants.iter().any(|p| p.mod_id == pin) {
                return pin;
            }
        }
        &self.participants.last().expect("conflict must have ≥2 participants").mod_id
    }

    /// `true` for binary / whole-file overlaps where load order is the only
    /// remediation (no per-field override possible).
    pub fn is_hard(&self) -> bool {
        matches!(self.kind, ConflictKind::Binary | ConflictKind::FileWhole)
    }
}

/// Walk every action in `mods` (in load order) and surface conflicts.
///
/// Output is sorted: by `file_path` ascending, then conflicts on the same
/// file ordered by record_id (for `Field`) before `Binary`/`FileWhole`.
pub fn detect_conflicts(mods: &[ModEntry<'_>]) -> Vec<Conflict> {
    detect_conflicts_with(mods, &ResolutionMap::default())
}

/// Same as [`detect_conflicts`] but annotates field conflicts with the
/// current pin (if any) from `resolutions`.
pub fn detect_conflicts_with(
    mods: &[ModEntry<'_>],
    resolutions: &ResolutionMap,
) -> Vec<Conflict> {
    // Field map: (file, record, field) -> Vec<participant>
    let mut field_map: BTreeMap<(String, u32, String), Vec<ConflictParticipant>> =
        BTreeMap::new();
    // Binary / whole-file: file -> Vec<participant>
    let mut binary_map: BTreeMap<String, Vec<ConflictParticipant>> = BTreeMap::new();
    let mut whole_map: BTreeMap<String, Vec<ConflictParticipant>> = BTreeMap::new();

    for entry in mods {
        // Per-mod dedupe: only the *last* contribution from any one mod on a
        // given key counts (prevents flagging intra-mod redundancy as a
        // conflict). The flatten pass should already have done this for
        // FieldDelta, but it's free insurance and matters for binary too.
        let mod_id = entry.mod_id.to_owned();
        for action in entry.changes.actions() {
            match &action.op {
                ChangeOp::FieldDelta {
                    record_id,
                    field,
                    new,
                    ..
                } => {
                    let key = (action.file_path.clone(), *record_id, field.clone());
                    let participants = field_map.entry(key).or_default();
                    upsert(
                        participants,
                        &mod_id,
                        "FieldDelta",
                        Some(new.clone()),
                    );
                }
                ChangeOp::BinaryDelta { .. } => {
                    let participants = binary_map
                        .entry(action.file_path.clone())
                        .or_default();
                    upsert(participants, &mod_id, "BinaryDelta", None);
                }
                ChangeOp::FileReplace { .. }
                | ChangeOp::FileAdd { .. }
                | ChangeOp::FileDelete => {
                    let participants = whole_map
                        .entry(action.file_path.clone())
                        .or_default();
                    upsert(participants, &mod_id, action.op.variant_name(), None);
                }
            }
        }
    }

    let mut out: Vec<Conflict> = Vec::new();

    for ((file, record_id, field), participants) in field_map {
        if !is_real_field_conflict(&participants) {
            continue;
        }
        let pinned_to = resolutions
            .winner(&FieldKey {
                file_path: file.clone(),
                record_id,
                field: field.clone(),
            })
            .map(str::to_owned);
        out.push(Conflict {
            file_path: file,
            kind: ConflictKind::Field { record_id, field },
            participants,
            pinned_to,
        });
    }
    for (file, participants) in binary_map {
        if participants.len() < 2 {
            continue;
        }
        out.push(Conflict {
            file_path: file,
            kind: ConflictKind::Binary,
            participants,
            pinned_to: None,
        });
    }
    for (file, participants) in whole_map {
        if participants.len() < 2 {
            continue;
        }
        out.push(Conflict {
            file_path: file,
            kind: ConflictKind::FileWhole,
            participants,
            pinned_to: None,
        });
    }

    out.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then_with(|| kind_order(&a.kind).cmp(&kind_order(&b.kind)))
    });
    out
}

fn upsert(
    participants: &mut Vec<ConflictParticipant>,
    mod_id: &str,
    op: &'static str,
    field_new: Option<Value>,
) {
    if let Some(existing) = participants.iter_mut().find(|p| p.mod_id == mod_id) {
        existing.op = op;
        existing.field_new = field_new;
    } else {
        participants.push(ConflictParticipant {
            mod_id: mod_id.to_owned(),
            op,
            field_new,
        });
    }
}

/// A field key is only a *real* conflict if at least two distinct mods
/// propose distinct `new` values. Same value from multiple mods = harmless.
fn is_real_field_conflict(participants: &[ConflictParticipant]) -> bool {
    if participants.len() < 2 {
        return false;
    }
    let first = match participants[0].field_new.as_ref() {
        Some(v) => v,
        None => return false,
    };
    participants
        .iter()
        .skip(1)
        .any(|p| p.field_new.as_ref() != Some(first))
}

fn kind_order(k: &ConflictKind) -> u8 {
    match k {
        ConflictKind::Field { .. } => 0,
        ConflictKind::Binary => 1,
        ConflictKind::FileWhole => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::change::ChangeAction;
    use crate::modding::changelog::ChangeLog;

    fn fd(rec: u32, field: &str, new: &str) -> ChangeAction {
        ChangeAction::new(
            "MiscItem.db",
            ChangeOp::FieldDelta {
                record_id: rec,
                field: field.into(),
                old: Value::Null,
                new: Value::String(new.into()),
            },
        )
    }

    fn log(actions: Vec<ChangeAction>) -> ChangeLog {
        ChangeLog::from_actions(actions)
    }

    #[test]
    fn no_overlap_no_conflict() {
        let a = log(vec![fd(1, "name", "Helmet")]);
        let b = log(vec![fd(2, "name", "Sword")]);
        let mods = [
            ModEntry { mod_id: "a", changes: &a },
            ModEntry { mod_id: "b", changes: &b },
        ];
        assert!(detect_conflicts(&mods).is_empty());
    }

    #[test]
    fn same_value_from_two_mods_is_not_a_conflict() {
        let a = log(vec![fd(1, "name", "Helmet")]);
        let b = log(vec![fd(1, "name", "Helmet")]);
        let mods = [
            ModEntry { mod_id: "a", changes: &a },
            ModEntry { mod_id: "b", changes: &b },
        ];
        assert!(detect_conflicts(&mods).is_empty());
    }

    #[test]
    fn different_values_on_same_field_conflict() {
        let a = log(vec![fd(1, "name", "Helmet")]);
        let b = log(vec![fd(1, "name", "Hat")]);
        let mods = [
            ModEntry { mod_id: "a", changes: &a },
            ModEntry { mod_id: "b", changes: &b },
        ];
        let conflicts = detect_conflicts(&mods);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].file_path, "MiscItem.db");
        assert!(matches!(
            conflicts[0].kind,
            ConflictKind::Field { record_id: 1, .. }
        ));
        assert_eq!(conflicts[0].participants.len(), 2);
        assert_eq!(conflicts[0].winner(), "b"); // last wins
        assert!(!conflicts[0].is_hard());
    }

    #[test]
    fn binary_overlap_is_hard_conflict() {
        let a = log(vec![ChangeAction::new(
            "Sprite/x.spr",
            ChangeOp::BinaryDelta { patch_bytes: vec![1] },
        )]);
        let b = log(vec![ChangeAction::new(
            "Sprite/x.spr",
            ChangeOp::BinaryDelta { patch_bytes: vec![2] },
        )]);
        let mods = [
            ModEntry { mod_id: "a", changes: &a },
            ModEntry { mod_id: "b", changes: &b },
        ];
        let c = detect_conflicts(&mods);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].kind, ConflictKind::Binary);
        assert!(c[0].is_hard());
    }

    #[test]
    fn whole_file_overlap_is_hard_conflict() {
        let a = log(vec![ChangeAction::new(
            "Map/cat1.map",
            ChangeOp::FileReplace { content: vec![1] },
        )]);
        let b = log(vec![ChangeAction::new(
            "Map/cat1.map",
            ChangeOp::FileDelete,
        )]);
        let mods = [
            ModEntry { mod_id: "a", changes: &a },
            ModEntry { mod_id: "b", changes: &b },
        ];
        let c = detect_conflicts(&mods);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].kind, ConflictKind::FileWhole);
        assert!(c[0].is_hard());
    }

    #[test]
    fn intra_mod_repeats_dont_inflate_participants() {
        // Same mod editing same field twice — only one participant counted.
        let a = log(vec![fd(1, "name", "Helm"), fd(1, "name", "Helmet")]);
        let b = log(vec![fd(1, "name", "Hat")]);
        let mods = [
            ModEntry { mod_id: "a", changes: &a },
            ModEntry { mod_id: "b", changes: &b },
        ];
        let c = detect_conflicts(&mods);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].participants.len(), 2);
        // Mod a's surviving value is the *last* one it proposed.
        let a_part = c[0].participants.iter().find(|p| p.mod_id == "a").unwrap();
        assert_eq!(a_part.field_new, Some(Value::String("Helmet".into())));
    }

    #[test]
    fn pin_annotates_winner_and_overrides_load_order() {
        let a = log(vec![fd(1, "name", "Helmet")]);
        let b = log(vec![fd(1, "name", "Hat")]);
        let mods = [
            ModEntry { mod_id: "a", changes: &a },
            ModEntry { mod_id: "b", changes: &b },
        ];
        let mut pins = ResolutionMap::default();
        pins.pin(
            FieldKey {
                file_path: "MiscItem.db".into(),
                record_id: 1,
                field: "name".into(),
            },
            "a",
        );
        let c = detect_conflicts_with(&mods, &pins);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].pinned_to.as_deref(), Some("a"));
        assert_eq!(c[0].winner(), "a"); // overrides load order
    }

    #[test]
    fn stale_pin_to_unknown_mod_falls_through_to_load_order() {
        let a = log(vec![fd(1, "name", "Helmet")]);
        let b = log(vec![fd(1, "name", "Hat")]);
        let mods = [
            ModEntry { mod_id: "a", changes: &a },
            ModEntry { mod_id: "b", changes: &b },
        ];
        let mut pins = ResolutionMap::default();
        pins.pin(
            FieldKey {
                file_path: "MiscItem.db".into(),
                record_id: 1,
                field: "name".into(),
            },
            "ghost",
        );
        let c = detect_conflicts_with(&mods, &pins);
        assert_eq!(c[0].pinned_to.as_deref(), Some("ghost"));
        // winner() ignores the stale pin and falls back to last-in-order.
        assert_eq!(c[0].winner(), "b");
    }

    #[test]
    fn output_sorted_deterministically() {
        let a = log(vec![
            ChangeAction::new("z.db", ChangeOp::FieldDelta {
                record_id: 0, field: "x".into(),
                old: Value::Null, new: Value::I64(1),
            }),
            ChangeAction::new("a.db", ChangeOp::FieldDelta {
                record_id: 0, field: "x".into(),
                old: Value::Null, new: Value::I64(1),
            }),
        ]);
        let b = log(vec![
            ChangeAction::new("z.db", ChangeOp::FieldDelta {
                record_id: 0, field: "x".into(),
                old: Value::Null, new: Value::I64(2),
            }),
            ChangeAction::new("a.db", ChangeOp::FieldDelta {
                record_id: 0, field: "x".into(),
                old: Value::Null, new: Value::I64(2),
            }),
        ]);
        let mods = [
            ModEntry { mod_id: "a", changes: &a },
            ModEntry { mod_id: "b", changes: &b },
        ];
        let c = detect_conflicts(&mods);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].file_path, "a.db");
        assert_eq!(c[1].file_path, "z.db");
    }
}
