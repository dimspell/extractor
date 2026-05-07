use crate::app::App;
use crate::editors::hex_editor::editing::{EditState, InspectorEditState};
use crate::editors::hex_editor::inspector::ENTRIES;
use crate::editors::hex_editor::selection::nav_target;
use crate::editors::hex_editor::HexEditorMessage;
use crate::editors::hex_editor::HexProvider;
use iced::Task;

/// Page nav heuristic — the matrix doesn't propagate live viewport height
/// up here, so PageUp/PageDown approximate a screenful.
const PAGE_ROWS: u64 = 24;

pub fn handle(message: HexEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);
    let Some(editor) = app.state.hex_editors.get_mut(&tab_id) else {
        return Task::none();
    };

    let max_addr = editor.max_addr();
    match message {
        HexEditorMessage::SetBytesPerRow(n) => {
            if matches!(n, 8 | 16 | 32) {
                editor.bytes_per_row = n;
            }
        }
        HexEditorMessage::SelectAt(addr) => {
            editor.selection.select(addr, max_addr);
            // Any non-edit click cancels in-flight typing.
            editor.edit_mode = None;
        }
        HexEditorMessage::ExtendTo(addr) => {
            editor.selection.extend(addr, max_addr);
        }
        HexEditorMessage::Nav { dir, extend } => {
            if editor.provider.is_empty() {
                return Task::none();
            }
            let bpr = editor.bytes_per_row as u64;
            let target = nav_target(editor.selection.cursor, dir, bpr, PAGE_ROWS, max_addr);
            if extend {
                editor.selection.extend(target, max_addr);
            } else {
                editor.selection.select(target, max_addr);
            }
            // Any nav cancels an in-flight edit (committing would be a
            // surprise; users hitting an arrow want to move, not write).
            editor.edit_mode = None;
        }

        HexEditorMessage::BeginEdit(addr) => {
            if editor.provider.is_empty() || !editor.provider.is_writable() {
                return Task::none();
            }
            let addr = addr.min(max_addr);
            editor.selection.select(addr, max_addr);
            editor.edit_mode = Some(EditState::new(addr));
        }
        HexEditorMessage::EditTypeChar(c) => {
            if editor.provider.is_empty() {
                return Task::none();
            }
            let edit_addr = match editor.edit_mode {
                Some(ref e) => e.addr,
                None => editor.selection.cursor,
            };
            let edit = editor
                .edit_mode
                .get_or_insert_with(|| EditState::new(edit_addr));
            if !edit.push_char(c) {
                return Task::none();
            }
            if edit.is_complete() {
                if let Some(byte) = edit.staged_byte() {
                    editor.provider.write(edit.addr, &[byte]);
                }
                let next = (edit.addr + 1).min(max_addr);
                if next == edit.addr {
                    // At EOF — commit, exit edit mode.
                    editor.edit_mode = None;
                } else {
                    editor.selection.select(next, max_addr);
                    editor.edit_mode = Some(EditState::new(next));
                }
            }
        }
        HexEditorMessage::EditBackspace => {
            if let Some(ref mut e) = editor.edit_mode {
                e.pop_char();
            }
        }
        HexEditorMessage::EditCancel => {
            editor.edit_mode = None;
        }
        HexEditorMessage::EditCommit { advance } => {
            if let Some(edit) = editor.edit_mode.take() {
                if let Some(byte) = edit.staged_byte() {
                    editor.provider.write(edit.addr, &[byte]);
                }
                if advance {
                    let next = (edit.addr + 1).min(max_addr);
                    editor.selection.select(next, max_addr);
                    if next > edit.addr {
                        editor.edit_mode = Some(EditState::new(next));
                    }
                } else {
                    editor.selection.select(edit.addr, max_addr);
                }
            }
        }

        HexEditorMessage::WriteBytes { addr, bytes } => {
            if !editor.provider.is_empty() {
                editor.provider.write(addr, &bytes);
            }
        }

        HexEditorMessage::BeginInspectorEdit(idx) => {
            if editor.provider.is_empty() {
                return Task::none();
            }
            let Some(entry) = ENTRIES.get(idx) else {
                return Task::none();
            };
            if entry.encode.is_none() {
                return Task::none();
            }
            let cursor = editor.selection.cursor;
            let len = editor.provider.len();
            if cursor + entry.min_size as u64 > len {
                return Task::none();
            }
            // Pre-fill the modal with the current decoded value.
            let bytes = editor.provider.read(cursor..cursor + entry.min_size as u64);
            let initial = (entry.decode)(bytes);
            // Strip any "(0xN)" suffix so the user types just the number.
            let initial = initial
                .split_once(' ')
                .map(|(lhs, _)| lhs.to_string())
                .unwrap_or(initial);
            editor.inspector_edit = Some(InspectorEditState::new(idx, cursor, initial));
        }
        HexEditorMessage::SetInspectorDraft(s) => {
            if let Some(ref mut ie) = editor.inspector_edit {
                ie.draft = s;
                ie.error = None;
            }
        }
        HexEditorMessage::CloseInspectorEdit => {
            editor.inspector_edit = None;
        }
        HexEditorMessage::CommitInspectorEdit => {
            let Some(ref ie) = editor.inspector_edit else {
                return Task::none();
            };
            let Some(entry) = ENTRIES.get(ie.entry_idx) else {
                editor.inspector_edit = None;
                return Task::none();
            };
            let Some(encode) = entry.encode else {
                editor.inspector_edit = None;
                return Task::none();
            };
            match encode(&ie.draft) {
                Ok(bytes) => {
                    let addr = ie.addr;
                    editor.provider.write(addr, &bytes);
                    editor.inspector_edit = None;
                }
                Err(msg) => {
                    if let Some(ref mut ie) = editor.inspector_edit {
                        ie.error = Some(msg);
                    }
                }
            }
        }
    }
    Task::none()
}
