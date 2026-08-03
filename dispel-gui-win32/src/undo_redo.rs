//! Generic, editor-agnostic undo/redo engine.
//!
//! Used by spreadsheet editors, the EventScript text editor, the hex editor,
//! and the map editor. The module is pure Rust — it makes no Win32 API calls.
//!
//! # Design
//!
//! We use a **trait-based** design: an edit is a boxed [`UndoableEdit`] whose
//! `undo`/`redo` methods apply against the editor state through captured
//! closures. Concrete helper constructors ([`field_edit`], [`bytes_edit`])
//! cover the two most common edit shapes (a single field of a record, and a
//! byte range in a byte-array editor). Editors that need custom behaviour can
//! implement [`UndoableEdit`] directly and still be stored in an [`UndoStack`].
//!
//! The [`UndoStack`] enforces the cross-cutting spec limits:
//! - at most [`UndoStack::MAX_UNDO`] (100) undo entries — the oldest entry is
//!   discarded when the limit is exceeded;
//! - pushing a new edit clears the redo stack.
//!
//! An [`UndoManager`] transaction wrapper was deliberately **not** added: the
//! specs only require undo, redo, and history (max 100, redo cleared on new
//! edit). Editors that need grouped edits can push a single composite edit that
//! implements [`UndoableEdit`] and replays several sub-edits in `undo`/`redo`.

/// A single reversible edit applied against editor state.
///
/// Implementations capture whatever state access they need (e.g. an
/// `Rc<RefCell<...>>` handle to the editor model) and mutate it in `undo` /
/// `redo`. `describe` provides a human-readable label for the undo history
/// panel (Ctrl+Alt+H).
pub trait UndoableEdit {
    /// Revert the edit, restoring the pre-edit state.
    fn undo(&self);
    /// Re-apply the edit, restoring the post-edit state.
    fn redo(&self);
    /// Human-readable label, e.g. "Edit weapon #12 name".
    fn describe(&self) -> String;
}

/// A field-level edit storing the old and new values of a single field.
///
/// The `apply` closure is invoked with either the old value (on undo) or the
/// new value (on redo); it sets the field on the editor's model.
pub struct FieldEdit<T> {
    record_desc: String,
    field_name: String,
    old_value: T,
    new_value: T,
    apply: Box<dyn Fn(T)>,
}

impl<T> FieldEdit<T> {
    fn new(
        record_desc: &str,
        field_name: &str,
        old_value: T,
        new_value: T,
        apply: Box<dyn Fn(T)>,
    ) -> Self {
        FieldEdit {
            record_desc: record_desc.to_string(),
            field_name: field_name.to_string(),
            old_value,
            new_value,
            apply,
        }
    }
}

impl<T: Clone> UndoableEdit for FieldEdit<T> {
    fn undo(&self) {
        (self.apply)(self.old_value.clone());
    }

    fn redo(&self) {
        (self.apply)(self.new_value.clone());
    }

    fn describe(&self) -> String {
        format!("Edit {} {}", self.record_desc, self.field_name)
    }
}

/// Convenience constructor for a single-field edit.
///
/// `apply` sets the field to the value it is given. Undo applies `old_value`,
/// redo applies `new_value`.
pub fn field_edit<T: Clone + PartialEq + 'static>(
    record_desc: &str,
    field_name: &str,
    old_value: T,
    new_value: T,
    apply: impl Fn(T) + 'static,
) -> Box<dyn UndoableEdit> {
    Box::new(FieldEdit::new(
        record_desc,
        field_name,
        old_value,
        new_value,
        Box::new(apply),
    ))
}

/// A byte-range edit for byte-array editors (hex editor, binary buffers).
///
/// The `apply` closure receives the offset and a byte slice to write at that
/// offset. Undo writes `old_bytes`, redo writes `new_bytes`.
pub struct BytesEdit {
    offset: usize,
    old_bytes: Vec<u8>,
    new_bytes: Vec<u8>,
    apply: Box<dyn Fn(usize, &[u8])>,
}

impl UndoableEdit for BytesEdit {
    fn undo(&self) {
        (self.apply)(self.offset, &self.old_bytes);
    }

    fn redo(&self) {
        (self.apply)(self.offset, &self.new_bytes);
    }

    fn describe(&self) -> String {
        format!("Edit {} byte(s) at 0x{:X}", self.new_bytes.len(), self.offset)
    }
}

/// Convenience constructor for a byte-range edit.
///
/// `apply` writes `bytes` at `offset` into the editor's byte buffer.
pub fn bytes_edit(
    offset: usize,
    old_bytes: Vec<u8>,
    new_bytes: Vec<u8>,
    apply: impl Fn(usize, &[u8]) + 'static,
) -> Box<dyn UndoableEdit> {
    Box::new(BytesEdit {
        offset,
        old_bytes,
        new_bytes,
        apply: Box::new(apply),
    })
}

/// The undo/redo stack shared by all editors.
///
/// The undo entries hold the most recent edit last (LIFO). Pushing a new edit
/// clears the redo stack and discards the oldest undo entry when the stack is
/// full.
pub struct UndoStack {
    undo_entries: Vec<Box<dyn UndoableEdit>>,
    redo_entries: Vec<Box<dyn UndoableEdit>>,
}

impl UndoStack {
    /// Maximum number of undo entries retained (oldest discarded beyond this).
    pub const MAX_UNDO: usize = 100;

    /// Creates an empty undo stack.
    pub fn new() -> Self {
        UndoStack {
            undo_entries: Vec::new(),
            redo_entries: Vec::new(),
        }
    }

    /// Pushes a new edit.
    ///
    /// The redo stack is cleared, and if the undo stack exceeds
    /// [`MAX_UNDO`](Self::MAX_UNDO) entries the oldest entry is discarded.
    pub fn push(&mut self, edit: Box<dyn UndoableEdit>) {
        self.redo_entries.clear();
        self.undo_entries.push(edit);
        if self.undo_entries.len() > Self::MAX_UNDO {
            self.undo_entries.remove(0);
        }
    }

    /// Undoes the most recent edit.
    ///
    /// Returns `false` if there is nothing to undo. On success the edit's
    /// `undo()` is invoked and the edit is moved to the redo stack.
    pub fn undo(&mut self) -> bool {
        match self.undo_entries.pop() {
            Some(edit) => {
                edit.undo();
                self.redo_entries.push(edit);
                true
            }
            None => false,
        }
    }

    /// Redoes the most recently undone edit.
    ///
    /// Returns `false` if there is nothing to redo. On success the edit's
    /// `redo()` is invoked and the edit is moved back to the undo stack.
    pub fn redo(&mut self) -> bool {
        match self.redo_entries.pop() {
            Some(edit) => {
                edit.redo();
                self.undo_entries.push(edit);
                true
            }
            None => false,
        }
    }

    /// Returns `true` if there is at least one edit to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_entries.is_empty()
    }

    /// Returns `true` if there is at least one edit to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_entries.is_empty()
    }

    /// Resets both the undo and redo stacks.
    pub fn clear(&mut self) {
        self.undo_entries.clear();
        self.redo_entries.clear();
    }

    /// Number of entries currently in the undo stack.
    pub fn undo_count(&self) -> usize {
        self.undo_entries.len()
    }

    /// Number of entries currently in the redo stack.
    pub fn redo_count(&self) -> usize {
        self.redo_entries.len()
    }

    /// Descriptions of all undoable actions, oldest first, most recent last.
    ///
    /// Used by the undo history panel (Ctrl+Alt+H).
    pub fn history(&self) -> Vec<String> {
        self.undo_entries.iter().map(|edit| edit.describe()).collect()
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        UndoStack::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Builds a `field_edit` against an `Rc<RefCell<i32>>` test state.
    fn test_field_edit(
        state: Rc<RefCell<i32>>,
        old: i32,
        new: i32,
        record_desc: &str,
    ) -> Box<dyn UndoableEdit> {
        let apply_state = state.clone();
        field_edit(record_desc, "value", old, new, move |v| {
            *apply_state.borrow_mut() = v;
        })
    }

    #[test]
    fn test_undo_redo_basic_field_edit() {
        let state = Rc::new(RefCell::new(1));
        let mut stack = UndoStack::new();

        stack.push(test_field_edit(state.clone(), 1, 2, "score"));
        assert_eq!(*state.borrow(), 1);
        assert!(stack.can_undo());
        assert!(!stack.can_redo());

        assert!(stack.undo());
        assert_eq!(*state.borrow(), 1);
        assert!(!stack.can_undo());
        assert!(stack.can_redo());

        assert!(stack.redo());
        assert_eq!(*state.borrow(), 2);
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_undo_redo_clears_redo_stack_on_new_edit() {
        let state = Rc::new(RefCell::new(0));
        let mut stack = UndoStack::new();

        stack.push(test_field_edit(state.clone(), 0, 1, "item"));
        assert!(stack.undo());
        assert!(stack.can_redo());

        // A brand-new edit must wipe the redo stack.
        stack.push(test_field_edit(state.clone(), 1, 2, "item"));
        assert!(!stack.can_redo());
        assert_eq!(stack.redo_count(), 0);
        assert!(stack.can_undo());
    }

    #[test]
    fn test_undo_history_limit_100() {
        let state = Rc::new(RefCell::new(0));
        let mut stack = UndoStack::new();

        for i in 1..=101 {
            stack.push(test_field_edit(state.clone(), i - 1, i, &format!("item {i}")));
        }

        assert_eq!(stack.undo_count(), UndoStack::MAX_UNDO);
        let history = stack.history();
        assert_eq!(history.len(), UndoStack::MAX_UNDO);
        // The oldest entry ("item 1") was discarded, so the oldest remaining
        // is "item 2"; the most recent is "item 101".
        assert!(history[0].contains("item 2"), "oldest retained: {}", history[0]);
        assert!(
            history.last().unwrap().contains("item 101"),
            "most recent: {}",
            history.last().unwrap()
        );
    }

    #[test]
    fn test_undo_empty_stack_returns_false() {
        let mut stack = UndoStack::new();
        assert!(!stack.undo());
        assert!(!stack.redo());
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
        assert_eq!(stack.undo_count(), 0);
        assert_eq!(stack.redo_count(), 0);
    }

    #[test]
    fn test_undo_history_descriptions() {
        let state = Rc::new(RefCell::new(0));
        let mut stack = UndoStack::new();

        stack.push(test_field_edit(state.clone(), 0, 1, "weapon #12"));
        stack.push(test_field_edit(state.clone(), 1, 2, "weapon #13"));

        let history = stack.history();
        assert_eq!(
            history,
            vec!["Edit weapon #12 value", "Edit weapon #13 value"]
        );
    }

    #[test]
    fn test_undo_redo_bytes_edit() {
        let data = Rc::new(RefCell::new(vec![0u8; 4]));
        let mut stack = UndoStack::new();
        let apply_data = data.clone();

        stack.push(bytes_edit(1, vec![0x00u8], vec![0xFFu8], move |offset, bytes| {
            let mut buf = apply_data.borrow_mut();
            buf[offset..offset + bytes.len()].copy_from_slice(bytes);
        }));

        assert_eq!(data.borrow()[1], 0x00);
        assert!(stack.undo());
        assert_eq!(data.borrow()[1], 0x00);
        assert!(stack.redo());
        assert_eq!(data.borrow()[1], 0xFF);
        assert_eq!(stack.history(), vec!["Edit 1 byte(s) at 0x1"]);
    }

    #[test]
    fn test_undo_redo_clear() {
        let state = Rc::new(RefCell::new(0));
        let mut stack = UndoStack::new();

        stack.push(test_field_edit(state.clone(), 0, 1, "item"));
        assert!(stack.undo());
        assert!(stack.can_undo() || stack.can_redo());

        stack.clear();
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
        assert_eq!(stack.undo_count(), 0);
        assert_eq!(stack.redo_count(), 0);
        assert!(stack.history().is_empty());
    }
}
