//! State for the Save.ifo editor.
//!
//! Slot rows are display-only (timestamps are written by the game itself on
//! save); the editable surface is the global tail block plus slot swapping.
//! Tail edits go through [`EditHistory`] so Ctrl+Z/Ctrl+Y work; swaps are
//! confirmed, immediately persisted, and intentionally NOT undoable.

use crate::components::edit_history::{EditAction, EditHistory};
use crate::components::generic_editor::UndoRedo;
use crate::components::loading_state::LoadingState;
use dispel_core::{SaveIfo, SlotSummary};

/// Loaded editor payload.
#[derive(Debug, Clone)]
pub struct EditorData {
    pub summaries: Vec<SlotSummary>,
    pub ifo: SaveIfo,
    pub dirty: bool,
}

/// String buffers for the tail text inputs (source of truth while typing).
#[derive(Debug, Clone, Default)]
pub struct TailBuffers {
    pub game_version: String,
    pub game_tmp_key: String,
    pub map_id: String,
    pub reserved: String,
    pub payload_counts: [String; 4],
}

#[derive(Debug, Clone)]
pub struct SaveIfoEditorState {
    pub data: Option<EditorData>,
    pub loading_state: LoadingState<()>,
    pub status_msg: String,
    pub tail_buffers: TailBuffers,
    pub edit_history: EditHistory,
    /// Slots awaiting confirmation: `(a, b)`.
    pub pending_swap: Option<(usize, usize)>,
}

impl Default for SaveIfoEditorState {
    fn default() -> Self {
        Self {
            data: None,
            loading_state: LoadingState::Idle,
            status_msg: String::new(),
            tail_buffers: TailBuffers::default(),
            edit_history: EditHistory::default(),
            pending_swap: None,
        }
    }
}

impl SaveIfoEditorState {
    /// Install freshly loaded data and reset transient editing state.
    pub fn load_completed(&mut self, summaries: Vec<SlotSummary>, ifo: SaveIfo) {
        self.data = Some(EditorData {
            summaries,
            ifo,
            dirty: false,
        });
        self.sync_tail_buffers();
        self.edit_history = EditHistory::default();
        self.pending_swap = None;
    }

    /// Splice in post-swap data: slot records and summaries are replaced from
    /// the fresh on-disk state, but unsaved tail edits (and their buffers)
    /// survive — a swap never touches the global tail.
    pub fn apply_swapped(&mut self, summaries: Vec<SlotSummary>, fresh: SaveIfo) {
        let Some(data) = &mut self.data else {
            self.load_completed(summaries, fresh);
            return;
        };
        data.summaries = summaries;
        data.ifo.slots = fresh.slots;
        // Tail fields and buffers intentionally left as-is (dirty stays true).
    }

    /// Refresh tail buffers from the loaded record.
    pub fn sync_tail_buffers(&mut self) {
        let Some(data) = &self.data else { return };
        let ifo = &data.ifo;
        self.tail_buffers = TailBuffers {
            game_version: ifo.game_version.to_string(),
            game_tmp_key: ifo.game_tmp_key.to_string(),
            map_id: ifo.map_id.to_string(),
            reserved: ifo.reserved.to_string(),
            payload_counts: ifo.payload_counts.map(|v| v.to_string()),
        };
    }

    /// Read a tail field as its string representation. `None` for unknown paths.
    pub fn field_value(ifo: &SaveIfo, path: &str) -> Option<String> {
        match path {
            "tail.game_version" => Some(ifo.game_version.to_string()),
            "tail.game_tmp_key" => Some(ifo.game_tmp_key.to_string()),
            "tail.map_id" => Some(ifo.map_id.to_string()),
            "tail.reserved" => Some(ifo.reserved.to_string()),
            other => {
                let index = other
                    .strip_prefix("tail.payload_counts.")?
                    .parse::<usize>()
                    .ok()?;
                ifo.payload_counts.get(index).map(|v| v.to_string())
            }
        }
    }

    /// Parse and apply a tail field. Returns `false` when the path is unknown
    /// or the value does not parse (the model is left untouched then).
    fn apply_field(ifo: &mut SaveIfo, path: &str, value: &str) -> bool {
        match path {
            "tail.game_version" => match value.parse::<f32>() {
                Ok(v) => {
                    ifo.game_version = v;
                    true
                }
                Err(_) => false,
            },
            "tail.game_tmp_key" => Self::apply_u32(&mut ifo.game_tmp_key, value),
            "tail.map_id" => Self::apply_u32(&mut ifo.map_id, value),
            "tail.reserved" => Self::apply_u32(&mut ifo.reserved, value),
            other => match other.strip_prefix("tail.payload_counts.") {
                Some(index) => match index.parse::<usize>() {
                    Ok(i) if i < ifo.payload_counts.len() => {
                        Self::apply_u32(&mut ifo.payload_counts[i], value)
                    }
                    _ => false,
                },
                None => false,
            },
        }
    }

    fn apply_u32(target: &mut u32, value: &str) -> bool {
        match value.parse::<u32>() {
            Ok(v) => {
                *target = v;
                true
            }
            Err(_) => false,
        }
    }

    /// Commit a tail edit: validates, applies to the model, marks dirty, and
    /// records history. Invalid input only updates the status message.
    pub fn update_field(&mut self, path: String, value: String) {
        let Some(data) = &mut self.data else {
            return;
        };
        let Some(old_value) = Self::field_value(&data.ifo, &path) else {
            return;
        };
        if old_value == value {
            return;
        }
        if !Self::apply_field(&mut data.ifo, &path, &value) {
            self.status_msg = format!("Invalid value for {path}: '{value}'");
            return;
        }
        data.dirty = true;
        self.set_tail_buffer(&path, value.clone());
        self.edit_history.push(EditAction::FieldChange {
            record_idx: 0,
            field: path,
            old_value,
            new_value: value,
        });
    }

    fn set_tail_buffer(&mut self, path: &str, value: String) {
        match path {
            "tail.game_version" => self.tail_buffers.game_version = value,
            "tail.game_tmp_key" => self.tail_buffers.game_tmp_key = value,
            "tail.map_id" => self.tail_buffers.map_id = value,
            "tail.reserved" => self.tail_buffers.reserved = value,
            other => {
                if let Some(index) = other.strip_prefix("tail.payload_counts.")
                    && let Ok(i) = index.parse::<usize>()
                    && i < self.tail_buffers.payload_counts.len()
                {
                    self.tail_buffers.payload_counts[i] = value;
                }
            }
        }
    }
}

impl UndoRedo for SaveIfoEditorState {
    fn undo(&mut self) -> Option<String> {
        let action = self.edit_history.undo()?;
        if let EditAction::FieldChange {
            ref field,
            ref old_value,
            ..
        } = action
        {
            if let Some(data) = &mut self.data {
                Self::apply_field(&mut data.ifo, field, old_value);
            }
            self.set_tail_buffer(field, old_value.clone());
        }
        Some(format!("Undo: {}", action.display_text()))
    }

    fn redo(&mut self) -> Option<String> {
        let action = self.edit_history.redo()?;
        // The redo action is the inverted undo action: old_value holds the
        // value to re-apply (same convention as the store editor).
        if let EditAction::FieldChange {
            ref field,
            ref old_value,
            ..
        } = action
        {
            if let Some(data) = &mut self.data {
                Self::apply_field(&mut data.ifo, field, old_value);
            }
            self.set_tail_buffer(field, old_value.clone());
        }
        Some(format!("Redo: {}", action.display_text()))
    }

    fn can_undo(&self) -> bool {
        self.edit_history.can_undo()
    }

    fn can_redo(&self) -> bool {
        self.edit_history.can_redo()
    }

    fn edit_history(&self) -> &EditHistory {
        &self.edit_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor_with_data() -> SaveIfoEditorState {
        let mut editor = SaveIfoEditorState::default();
        editor.load_completed(
            Vec::new(),
            SaveIfo {
                game_tmp_key: 6,
                map_id: 12,
                ..SaveIfo::default()
            },
        );
        editor
    }

    #[test]
    fn test_field_change_updates_model_and_marks_dirty() {
        let mut editor = editor_with_data();
        editor.update_field("tail.map_id".into(), "13".into());
        let data = editor.data.as_ref().unwrap();
        assert_eq!(data.ifo.map_id, 13);
        assert!(data.dirty);
        assert!(editor.edit_history.can_undo());
    }

    #[test]
    fn test_field_change_invalid_value_is_rejected() {
        let mut editor = editor_with_data();
        editor.update_field("tail.map_id".into(), "abc".into());
        assert_eq!(editor.data.as_ref().unwrap().ifo.map_id, 12);
        assert!(!editor.data.as_ref().unwrap().dirty);
        assert!(!editor.edit_history.can_undo());
        assert!(editor.status_msg.contains("Invalid value"));
    }

    #[test]
    fn test_undo_reverts_tail_field_and_buffer() {
        let mut editor = editor_with_data();
        editor.update_field("tail.game_tmp_key".into(), "99".into());
        let msg = editor.undo();
        assert!(msg.is_some());
        assert_eq!(editor.data.as_ref().unwrap().ifo.game_tmp_key, 6);
        assert_eq!(editor.tail_buffers.game_tmp_key, "6");
    }

    #[test]
    fn test_redo_reapplies_tail_field() {
        let mut editor = editor_with_data();
        editor.update_field("tail.payload_counts.0".into(), "350".into());
        editor.undo();
        editor.redo();
        assert_eq!(editor.data.as_ref().unwrap().ifo.payload_counts[0], 350);
        assert_eq!(editor.tail_buffers.payload_counts[0], "350");
    }

    #[test]
    fn test_no_change_skips_history() {
        let mut editor = editor_with_data();
        editor.update_field("tail.map_id".into(), "12".into());
        assert!(!editor.edit_history.can_undo());
    }

    #[test]
    fn test_load_completed_resets_history_and_dirty() {
        let mut editor = editor_with_data();
        editor.update_field("tail.map_id".into(), "20".into());
        editor.load_completed(Vec::new(), SaveIfo::default());
        assert!(!editor.data.as_ref().unwrap().dirty);
        assert!(!editor.edit_history.can_undo());
        assert_eq!(editor.tail_buffers.map_id, "0");
    }

    #[test]
    fn swapdone_preserves_unsaved_tail_edits() {
        let mut editor = editor_with_data();
        editor.update_field("tail.map_id".into(), "99".into());

        // Fresh on-disk state after a swap: different slots, different tail.
        let mut fresh = SaveIfo {
            game_tmp_key: 7,
            map_id: 55,
            ..SaveIfo::default()
        };
        fresh.slots[0].flags = [1, 0, 0, 0];
        fresh.slots[0].month = 9;
        let summaries = vec![SlotSummary {
            index: 0,
            occupied: true,
            sav_present: true,
            month: 9,
            day: 1,
            hour: 2,
            minute: 3,
            game_tmp_key: Some(7),
            map_id: Some(55),
        }];

        editor.apply_swapped(summaries, fresh);

        let data = editor.data.as_ref().unwrap();
        // Slots + summaries replaced from fresh…
        assert_eq!(data.ifo.slots[0].month, 9);
        assert_eq!(data.summaries.len(), 1);
        assert_eq!(data.summaries[0].month, 9);
        // …but unsaved tail edits survive.
        assert_eq!(data.ifo.map_id, 99);
        assert!(data.dirty);
        assert_eq!(editor.tail_buffers.map_id, "99");
    }
}
