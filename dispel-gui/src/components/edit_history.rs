use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const MAX_HISTORY: usize = 100;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EditAction {
    FieldChange {
        record_idx: usize,
        field: String,
        old_value: String,
        new_value: String,
    },
    RecordAdd {
        record_idx: usize,
        data: String,
    },
    RecordRemove {
        record_idx: usize,
        data: String,
    },
}

impl EditAction {
    pub fn display_text(&self) -> String {
        match self {
            EditAction::FieldChange {
                field, new_value, ..
            } => format!("Changed {} to \"{}\"", field, new_value),
            EditAction::RecordAdd { record_idx, .. } => {
                format!("Added record #{}", record_idx)
            }
            EditAction::RecordRemove { record_idx, .. } => {
                format!("Removed record #{}", record_idx)
            }
        }
    }

    pub fn record_idx(&self) -> usize {
        match self {
            EditAction::FieldChange { record_idx, .. }
            | EditAction::RecordAdd { record_idx, .. }
            | EditAction::RecordRemove { record_idx, .. } => *record_idx,
        }
    }

    pub fn set_record_idx(&mut self, new_idx: usize) {
        match self {
            EditAction::FieldChange { record_idx, .. }
            | EditAction::RecordAdd { record_idx, .. }
            | EditAction::RecordRemove { record_idx, .. } => *record_idx = new_idx,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EditHistory {
    undo_stack: VecDeque<EditAction>,
    redo_stack: VecDeque<EditAction>,
}

impl EditHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, action: EditAction) {
        self.undo_stack.push_front(action);
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.pop_back();
        }
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_stack(&self) -> &VecDeque<EditAction> {
        &self.undo_stack
    }

    pub fn redo_stack(&self) -> &VecDeque<EditAction> {
        &self.redo_stack
    }

    pub fn undo(&mut self) -> Option<EditAction> {
        if let Some(action) = self.undo_stack.pop_front() {
            let inverted = action.clone().invert();
            self.redo_stack.push_front(inverted);
            Some(action)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<EditAction> {
        if let Some(action) = self.redo_stack.pop_front() {
            let inverted = action.clone().invert();
            self.undo_stack.push_front(inverted);
            Some(action)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Adjust all `record_idx` values after removing record at `removed_idx`.
    /// Actions referencing the removed record are dropped; actions referencing
    /// records after it have their index decremented.
    pub fn adjust_for_removal(&mut self, removed_idx: usize) {
        self.undo_stack.retain(|a| a.record_idx() != removed_idx);
        self.redo_stack.retain(|a| a.record_idx() != removed_idx);
        for action in self.undo_stack.iter_mut() {
            let idx = action.record_idx();
            if idx > removed_idx {
                action.set_record_idx(idx - 1);
            }
        }
        for action in self.redo_stack.iter_mut() {
            let idx = action.record_idx();
            if idx > removed_idx {
                action.set_record_idx(idx - 1);
            }
        }
    }

    /// Adjust all `record_idx` values after adding a record at `added_idx`.
    /// Actions referencing records at or after the added index are incremented.
    pub fn adjust_for_addition(&mut self, added_idx: usize) {
        for action in self.undo_stack.iter_mut() {
            let idx = action.record_idx();
            if idx >= added_idx {
                action.set_record_idx(idx + 1);
            }
        }
        for action in self.redo_stack.iter_mut() {
            let idx = action.record_idx();
            if idx >= added_idx {
                action.set_record_idx(idx + 1);
            }
        }
    }
}

impl EditAction {
    pub fn invert(self) -> EditAction {
        match self {
            EditAction::FieldChange {
                record_idx,
                field,
                old_value,
                new_value,
            } => EditAction::FieldChange {
                record_idx,
                field,
                old_value: new_value,
                new_value: old_value,
            },
            EditAction::RecordAdd { record_idx, data } => {
                EditAction::RecordRemove { record_idx, data }
            }
            EditAction::RecordRemove { record_idx, data } => {
                EditAction::RecordAdd { record_idx, data }
            }
        }
    }
}
