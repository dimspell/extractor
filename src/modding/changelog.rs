use serde::{Deserialize, Serialize};

use super::change::ChangeAction;

/// Per-mod cap on retained actions. Adding a 101st action drops the oldest.
pub const HISTORY_CAP: usize = 100;

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

    pub fn from_actions(actions: Vec<ChangeAction>) -> Self {
        let mut log = Self { actions };
        log.enforce_cap();
        log
    }

    pub fn push(&mut self, action: ChangeAction) {
        self.actions.push(action);
        self.enforce_cap();
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
        use std::collections::HashMap;
        use crate::modding::change::ChangeOp;
        use crate::modding::value::Value;

        // Map (file, record, field) -> index of the last FieldDelta seen.
        let mut last_seen: HashMap<(String, u32, String), usize> = HashMap::new();
        // Carries the original `old` forward as we walk; keyed identically.
        let mut original_old: HashMap<(String, u32, String), Value> = HashMap::new();
        let mut to_drop: Vec<usize> = Vec::new();

        for (idx, action) in self.actions.iter().enumerate() {
            let ChangeOp::FieldDelta { record_id, field, old, .. } = &action.op else {
                continue;
            };
            let key = (action.file_path.clone(), *record_id, field.clone());
            if let Some(prev_idx) = last_seen.insert(key.clone(), idx) {
                to_drop.push(prev_idx);
            } else {
                original_old.insert(key, old.clone());
            }
        }

        // Rewrite surviving entries' `old` to the original captured at the
        // first occurrence.
        for (&(ref file, record_id, ref field), &idx) in last_seen.iter() {
            let key = (file.clone(), record_id, field.clone());
            let Some(orig) = original_old.get(&key) else {
                continue;
            };
            if let ChangeOp::FieldDelta { old, .. } = &mut self.actions[idx].op {
                *old = orig.clone();
            }
        }

        // Mark surviving entries that are now no-ops for removal too.
        for (_, &idx) in last_seen.iter() {
            if let ChangeOp::FieldDelta { old, new, .. } = &self.actions[idx].op {
                if old == new {
                    to_drop.push(idx);
                }
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

    fn enforce_cap(&mut self) {
        if self.actions.len() > HISTORY_CAP {
            let drop = self.actions.len() - HISTORY_CAP;
            self.actions.drain(..drop);
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
        let actions: Vec<_> = (0..(HISTORY_CAP as u32 + 10))
            .map(dummy_action)
            .collect();
        let log = ChangeLog::from_actions(actions);
        assert_eq!(log.len(), HISTORY_CAP);
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
        log.push(fd("MiscItem.db", 42, "name", Value::String("Hel".into()), Value::String("Helm".into())));
        log.push(fd("MiscItem.db", 42, "name", Value::String("Helm".into()), Value::String("Helmet".into())));

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
        log.push(fd("MiscItem.db", 1, "name", Value::String("a".into()), Value::String("b".into())));
        log.push(fd("MiscItem.db", 1, "name", Value::String("b".into()), Value::String("a".into())));

        let removed = log.flatten_field_deltas();
        assert_eq!(removed, 2);
        assert!(log.is_empty());
    }

    #[test]
    fn flatten_preserves_independent_keys_and_position() {
        let mut log = ChangeLog::new();
        log.push(fd("MiscItem.db", 1, "name", Value::String("a".into()), Value::String("b".into())));
        log.push(fd("MiscItem.db", 2, "name", Value::String("x".into()), Value::String("y".into())));
        log.push(fd("MiscItem.db", 1, "name", Value::String("b".into()), Value::String("c".into())));

        log.flatten_field_deltas();
        assert_eq!(log.len(), 2);
        // Order: rec 2 then the collapsed rec 1 (because the rec-1 entry's
        // latest position is index 2, after the rec-2 entry at index 1).
        let ChangeOp::FieldDelta { record_id: r0, .. } = &log.actions()[0].op else { unreachable!() };
        let ChangeOp::FieldDelta { record_id: r1, new: n1, .. } = &log.actions()[1].op else { unreachable!() };
        assert_eq!(*r0, 2);
        assert_eq!(*r1, 1);
        assert_eq!(*n1, Value::String("c".into()));
    }

    #[test]
    fn flatten_leaves_non_field_actions_alone() {
        let mut log = ChangeLog::new();
        log.push(ChangeAction::new("Sprite/x.spr", ChangeOp::BinaryDelta { patch_bytes: vec![1, 2] }));
        log.push(fd("MiscItem.db", 1, "name", Value::String("a".into()), Value::String("b".into())));
        log.push(fd("MiscItem.db", 1, "name", Value::String("b".into()), Value::String("c".into())));
        log.push(ChangeAction::new("Sprite/x.spr", ChangeOp::BinaryDelta { patch_bytes: vec![3, 4] }));

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
}
