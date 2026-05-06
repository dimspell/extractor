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
