use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::change::{ChangeAction, ChangeOp};
use super::resolution::FieldKey;
use super::value::Value;

/// Per-mod cap on retained actions. Adding a 101st action drops the oldest.
pub const HISTORY_CAP: usize = 100;

/// The destination a change contributes to, used to decide which of a mod's
/// several edits to the same place is the effective one.
///
/// This is the single key derivation shared by changelog flattening, conflict
/// detection and the apply engine. Field edits are keyed by field identity;
/// each whole-file op family is keyed by file path so a mod's last edit to a
/// file wins without erasing unrelated field edits on that same file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChangeTarget {
    Field(FieldKey),
    Binary(String),
    WholeFile(String),
}

impl ChangeTarget {
    pub fn of(action: &ChangeAction) -> Option<Self> {
        match &action.op {
            ChangeOp::FieldDelta { .. } => action.op.field_key(&action.file_path).map(Self::Field),
            ChangeOp::BinaryDelta { .. } => Some(Self::Binary(action.file_path.clone())),
            ChangeOp::FileReplace { .. } | ChangeOp::FileAdd { .. } | ChangeOp::FileDelete => {
                Some(Self::WholeFile(action.file_path.clone()))
            }
        }
    }
}

/// The actions of one mod that are still effective after per-mod dedup: the
/// *last* action for each [`ChangeTarget`] wins.
///
/// Conflict detection and the apply engine both consume this view, so the
/// reported winner is always the contribution whose bytes get written — even
/// for a changelog loaded from disk that was never flattened. Writers flatten
/// the stored changelog through [`ChangeLog::flatten_field_deltas`]; readers
/// rely on this function instead.
pub fn effective_actions(actions: &[ChangeAction]) -> Vec<&ChangeAction> {
    let mut last: HashMap<ChangeTarget, usize> = HashMap::with_capacity(actions.len());
    let mut keep: Vec<usize> = Vec::with_capacity(actions.len());
    for (idx, action) in actions.iter().enumerate() {
        match ChangeTarget::of(action) {
            Some(target) => {
                last.insert(target, idx);
            }
            None => keep.push(idx),
        }
    }
    keep.extend(last.into_values());
    keep.sort_unstable();
    keep.into_iter().map(|idx| &actions[idx]).collect()
}

/// Linear, append-only history of [`ChangeAction`]s for a single mod.
///
/// Bounded at [`HISTORY_CAP`] entries (oldest pruned on overflow).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChangeLog {
    actions: Vec<ChangeAction>,
}

impl ChangeLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a changelog from actions, returning it together with the number
    /// of oldest actions dropped by the cap.
    pub fn from_actions_reporting(actions: Vec<ChangeAction>) -> (Self, usize) {
        let mut log = Self { actions };
        let pruned = log.enforce_cap();
        (log, pruned)
    }

    /// Build a changelog from actions, silently applying the cap.
    pub fn from_actions(actions: Vec<ChangeAction>) -> Self {
        Self::from_actions_reporting(actions).0
    }

    /// Append an action and return the number of oldest actions the cap dropped.
    pub fn push(&mut self, action: ChangeAction) -> usize {
        self.actions.push(action);
        self.enforce_cap()
    }

    /// Remove a change action by its [`Uuid`]. Returns `true` if found and
    /// removed, `false` if no action with that id existed.
    pub fn remove_by_id(&mut self, id: uuid::Uuid) -> bool {
        let before = self.actions.len();
        self.actions.retain(|a| a.id != id);
        if self.actions.len() < before {
            self.flatten_field_deltas();
            true
        } else {
            false
        }
    }

    pub fn actions(&self) -> &[ChangeAction] {
        &self.actions
    }

    pub fn into_actions(self) -> Vec<ChangeAction> {
        self.actions
    }

    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ChangeAction> {
        self.actions.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, ChangeAction> {
        self.actions.iter_mut()
    }

    /// Coalesce multiple [`ChangeOp::FieldDelta`] entries that target the same
    /// `(file_path, record_id, field)` into a single entry whose `old` is the
    /// earliest captured value and `new` is the latest. The collapsed entry
    /// keeps the *latest* timeline position so the mod's history reads as a
    /// single edit at the most recent moment. Resulting no-ops (where
    /// `old == new` after collapse) are dropped entirely.
    ///
    /// Non-FieldDelta entries are left untouched.
    ///
    /// Returns the number of actions removed.
    pub fn flatten_field_deltas(&mut self) -> usize {
        // FieldKey -> index of the last FieldDelta seen. The key derivation is
        // the same one `effective_actions` uses for conflict detection/apply.
        let mut last_seen: HashMap<FieldKey, usize> = HashMap::new();
        // Carries the original `old` forward as we walk; keyed identically.
        let mut original_old: HashMap<FieldKey, Value> = HashMap::new();
        let mut to_drop: Vec<usize> = Vec::new();

        for (idx, action) in self.actions.iter().enumerate() {
            let ChangeOp::FieldDelta { old, .. } = &action.op else {
                continue;
            };
            let Some(ChangeTarget::Field(key)) = ChangeTarget::of(action) else {
                continue;
            };
            if let Some(prev_idx) = last_seen.insert(key.clone(), idx) {
                to_drop.push(prev_idx);
            } else {
                original_old.insert(key, old.clone());
            }
        }

        // Rewrite surviving entries' `old` to the original captured at the
        // first occurrence.
        for (key, &idx) in last_seen.iter() {
            let Some(orig) = original_old.get(key) else {
                continue;
            };
            if let ChangeOp::FieldDelta { old, .. } = &mut self.actions[idx].op {
                *old = orig.clone();
            }
        }

        // Mark surviving entries that are now no-ops for removal too.
        for &idx in last_seen.values() {
            if let ChangeOp::FieldDelta { old, new, .. } = &self.actions[idx].op
                && old == new
            {
                to_drop.push(idx);
            }
        }

        to_drop.sort_unstable();
        to_drop.dedup();
        let removed = to_drop.len();
        for idx in to_drop.into_iter().rev() {
            self.actions.remove(idx);
        }
        removed
    }

    /// Drop the oldest actions beyond [`HISTORY_CAP`] and return how many were
    /// removed. The surviving actions keep their order.
    pub fn enforce_cap(&mut self) -> usize {
        if self.actions.len() > HISTORY_CAP {
            let removed = self.actions.len() - HISTORY_CAP;
            self.actions.drain(..removed);
            removed
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modding::change::ChangeOp;
    use crate::modding::value::Value;

    fn dummy_action(i: u32) -> ChangeAction {
        ChangeAction::new(
            "MiscItem.db",
            ChangeOp::FieldDelta {
                record_id: i,
                field: "name".into(),
                old: Value::Null,
                new: Value::I64(i as i64),
            },
        )
    }

    #[test]
    fn push_appends_and_preserves_order() {
        let mut log = ChangeLog::new();
        for i in 0..5 {
            log.push(dummy_action(i));
        }
        assert_eq!(log.len(), 5);
        for (i, a) in log.iter().enumerate() {
            let ChangeOp::FieldDelta { record_id, .. } = &a.op else {
                unreachable!();
            };
            assert_eq!(*record_id, i as u32);
        }
    }

    #[test]
    fn cap_drops_oldest() {
        let mut log = ChangeLog::new();
        for i in 0..(HISTORY_CAP as u32 + 5) {
            log.push(dummy_action(i));
        }
        assert_eq!(log.len(), HISTORY_CAP);

        let ChangeOp::FieldDelta { record_id, .. } = &log.actions()[0].op else {
            unreachable!();
        };
        // First 5 entries dropped, so the oldest surviving record_id is 5.
        assert_eq!(*record_id, 5);
    }

    #[test]
    fn from_actions_enforces_cap() {
        let actions: Vec<_> = (0..(HISTORY_CAP as u32 + 10)).map(dummy_action).collect();
        let log = ChangeLog::from_actions(actions);
        assert_eq!(log.len(), HISTORY_CAP);
    }

    #[test]
    fn push_reports_exactly_one_pruned_at_the_cap() {
        let mut log = ChangeLog::new();
        for i in 0..(HISTORY_CAP as u32) {
            assert_eq!(log.push(dummy_action(i)), 0);
        }
        assert_eq!(log.push(dummy_action(HISTORY_CAP as u32)), 1);

        assert_eq!(log.len(), HISTORY_CAP);
        let ChangeOp::FieldDelta {
            record_id: first, ..
        } = &log.actions()[0].op
        else {
            unreachable!();
        };
        let ChangeOp::FieldDelta {
            record_id: last, ..
        } = &log.actions()[HISTORY_CAP - 1].op
        else {
            unreachable!();
        };
        // The dropped action is the oldest; ids 1..=HISTORY_CAP survive.
        assert_eq!(*first, 1);
        assert_eq!(*last, HISTORY_CAP as u32);
    }

    #[test]
    fn from_actions_reporting_reports_pruned_and_keeps_newest() {
        let actions: Vec<_> = (0..(HISTORY_CAP as u32 + 1)).map(dummy_action).collect();
        let (log, pruned) = ChangeLog::from_actions_reporting(actions);
        assert_eq!(pruned, 1);
        assert_eq!(log.len(), HISTORY_CAP);

        let ChangeOp::FieldDelta {
            record_id: first, ..
        } = &log.actions()[0].op
        else {
            unreachable!();
        };
        let ChangeOp::FieldDelta {
            record_id: last, ..
        } = &log.actions()[HISTORY_CAP - 1].op
        else {
            unreachable!();
        };
        assert_eq!(*first, 1);
        assert_eq!(*last, HISTORY_CAP as u32);
    }

    fn fd(file: &str, rec: u32, field: &str, old: Value, new: Value) -> ChangeAction {
        ChangeAction::new(
            file,
            ChangeOp::FieldDelta {
                record_id: rec,
                field: field.into(),
                old,
                new,
            },
        )
    }

    #[test]
    fn flatten_collapses_consecutive_same_key() {
        let mut log = ChangeLog::new();
        log.push(fd(
            "MiscItem.db",
            42,
            "name",
            Value::String("Hel".into()),
            Value::String("Helm".into()),
        ));
        log.push(fd(
            "MiscItem.db",
            42,
            "name",
            Value::String("Helm".into()),
            Value::String("Helmet".into()),
        ));

        let removed = log.flatten_field_deltas();
        assert_eq!(removed, 1);
        assert_eq!(log.len(), 1);
        let ChangeOp::FieldDelta { old, new, .. } = &log.actions()[0].op else {
            unreachable!();
        };
        assert_eq!(*old, Value::String("Hel".into()));
        assert_eq!(*new, Value::String("Helmet".into()));
    }

    #[test]
    fn flatten_drops_full_round_trip_to_original() {
        let mut log = ChangeLog::new();
        log.push(fd(
            "MiscItem.db",
            1,
            "name",
            Value::String("a".into()),
            Value::String("b".into()),
        ));
        log.push(fd(
            "MiscItem.db",
            1,
            "name",
            Value::String("b".into()),
            Value::String("a".into()),
        ));

        let removed = log.flatten_field_deltas();
        assert_eq!(removed, 2);
        assert!(log.is_empty());
    }

    #[test]
    fn flatten_preserves_independent_keys_and_position() {
        let mut log = ChangeLog::new();
        log.push(fd(
            "MiscItem.db",
            1,
            "name",
            Value::String("a".into()),
            Value::String("b".into()),
        ));
        log.push(fd(
            "MiscItem.db",
            2,
            "name",
            Value::String("x".into()),
            Value::String("y".into()),
        ));
        log.push(fd(
            "MiscItem.db",
            1,
            "name",
            Value::String("b".into()),
            Value::String("c".into()),
        ));

        log.flatten_field_deltas();
        assert_eq!(log.len(), 2);
        // Order: rec 2 then the collapsed rec 1 (because the rec-1 entry's
        // latest position is index 2, after the rec-2 entry at index 1).
        let ChangeOp::FieldDelta { record_id: r0, .. } = &log.actions()[0].op else {
            unreachable!()
        };
        let ChangeOp::FieldDelta {
            record_id: r1,
            new: n1,
            ..
        } = &log.actions()[1].op
        else {
            unreachable!()
        };
        assert_eq!(*r0, 2);
        assert_eq!(*r1, 1);
        assert_eq!(*n1, Value::String("c".into()));
    }

    #[test]
    fn flatten_leaves_non_field_actions_alone() {
        let mut log = ChangeLog::new();
        log.push(ChangeAction::new(
            "Sprite/x.spr",
            ChangeOp::BinaryDelta {
                patch_bytes: vec![1, 2],
            },
        ));
        log.push(fd(
            "MiscItem.db",
            1,
            "name",
            Value::String("a".into()),
            Value::String("b".into()),
        ));
        log.push(fd(
            "MiscItem.db",
            1,
            "name",
            Value::String("b".into()),
            Value::String("c".into()),
        ));
        log.push(ChangeAction::new(
            "Sprite/x.spr",
            ChangeOp::BinaryDelta {
                patch_bytes: vec![3, 4],
            },
        ));

        log.flatten_field_deltas();
        assert_eq!(log.len(), 3);
    }

    #[test]
    fn serde_round_trip() {
        let mut log = ChangeLog::new();
        log.push(dummy_action(1));
        log.push(dummy_action(2));
        let json = serde_json::to_string(&log).unwrap();
        // transparent: serialises as a bare array.
        assert!(json.starts_with('['));
        let back: ChangeLog = serde_json::from_str(&json).unwrap();
        assert_eq!(log, back);
    }

    #[test]
    fn effective_actions_keep_only_last_per_target() {
        let first = fd(
            "MiscItem.db",
            1,
            "name",
            Value::String("a".into()),
            Value::String("b".into()),
        );
        let last = fd(
            "MiscItem.db",
            1,
            "name",
            Value::String("b".into()),
            Value::String("c".into()),
        );
        let binary = ChangeAction::new(
            "Sprite/x.spr",
            ChangeOp::BinaryDelta {
                patch_bytes: vec![1],
            },
        );
        let old_binary_id = binary.id;
        let binary2 = ChangeAction::new(
            "Sprite/x.spr",
            ChangeOp::BinaryDelta {
                patch_bytes: vec![2],
            },
        );
        let last_id = last.id;
        let actions = [first, last, binary, binary2];
        let keep = effective_actions(&actions);
        assert_eq!(keep.len(), 2);
        assert_eq!(keep[0].id, last_id);
        assert_ne!(keep[1].id, old_binary_id);
    }

    #[test]
    fn effective_actions_preserve_distinct_targets_and_position() {
        let name = fd(
            "MiscItem.db",
            1,
            "name",
            Value::Null,
            Value::String("Helmet".into()),
        );
        let price = fd("MiscItem.db", 1, "price", Value::Null, Value::I64(9));
        let whole = ChangeAction::new("MiscItem.db", ChangeOp::FileDelete);
        let name_id = name.id;
        let price_id = price.id;
        let whole_id = whole.id;
        let actions = [name.clone(), price, whole, name];
        let keep = effective_actions(&actions);
        assert_eq!(keep.len(), 3);
        assert_eq!(keep[0].id, price_id);
        assert_eq!(keep[1].id, whole_id);
        assert_eq!(keep[2].id, name_id);
    }

    #[test]
    fn effective_actions_agree_with_flattened_log() {
        let actions = vec![
            fd(
                "MiscItem.db",
                1,
                "name",
                Value::String("a".into()),
                Value::String("b".into()),
            ),
            fd(
                "MiscItem.db",
                2,
                "name",
                Value::String("x".into()),
                Value::String("y".into()),
            ),
            fd(
                "MiscItem.db",
                1,
                "name",
                Value::String("b".into()),
                Value::String("c".into()),
            ),
        ];
        let mut log = ChangeLog::from_actions(actions.clone());
        log.flatten_field_deltas();
        let effective = effective_actions(&actions);
        assert_eq!(effective.len(), log.len());
        for (action, flattened) in effective.iter().zip(log.actions()) {
            // Flattening additionally rewrites `old` to the earliest captured
            // value; the identity and the final `new` must still line up.
            assert_eq!(action.id, flattened.id);
            let (ChangeOp::FieldDelta { new: a, .. }, ChangeOp::FieldDelta { new: b, .. }) =
                (&action.op, &flattened.op)
            else {
                panic!("expected field deltas");
            };
            assert_eq!(a, b);
        }
    }
}
