use std::path::PathBuf;

use dispel_core::modding::{make_delta, ChangeAction, ChangeOp, Workspace};
use iced::Task;

use crate::app::App;
use crate::editors::hex_editor::editing::{EditState, InspectorEditState};
use crate::editors::hex_editor::goto::GotoState;
use crate::editors::hex_editor::inspector::ENTRIES;
use crate::editors::hex_editor::selection::nav_target;
use crate::editors::hex_editor::HexEditorMessage;
use crate::editors::hex_editor::HexProvider;
use crate::message::{Message, MessageExt};

/// Page nav heuristic — the matrix doesn't propagate live viewport height
/// up here, so PageUp/PageDown approximate a screenful.
const PAGE_ROWS: u64 = 24;

/// If the bsdiff delta is at least this fraction of the full file size, we
/// emit `FileReplace` instead of `BinaryDelta` — the patch is no longer a
/// space win and a full replace is simpler to inspect.
const DELTA_KEEP_THRESHOLD: f64 = 0.7;

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
            let staged = edit.is_complete().then(|| (edit.addr, edit.staged_byte()));
            if let Some((addr, byte)) = staged {
                if let Some(byte) = byte {
                    editor.provider.write(addr, &[byte]);
                    editor.recompute_vanilla_diff();
                }
                let next = (addr + 1).min(max_addr);
                if next == addr {
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
                    editor.recompute_vanilla_diff();
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
                editor.recompute_vanilla_diff();
            }
        }

        // ── Inspector ───────────────────────────────────────────────────
        HexEditorMessage::CopyInspectorValue(idx) => {
            let cursor = editor.selection.cursor;
            let len = editor.provider.len();
            let avail = (len - cursor) as usize;
            let read_end = (cursor + 64).min(len);
            let bytes = editor.provider.read(cursor..read_end);
            if let Some(entry) = ENTRIES.get(idx) {
                if avail >= entry.min_size {
                    let decoded = (entry.decode)(bytes);
                    editor.status_msg = format!("Copied: {decoded}");
                }
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
            let bytes = editor.provider.read(cursor..cursor + entry.min_size as u64);
            let initial = (entry.decode)(bytes);
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
                    editor.recompute_vanilla_diff();
                    editor.inspector_edit = None;
                }
                Err(msg) => {
                    if let Some(ref mut ie) = editor.inspector_edit {
                        ie.error = Some(msg);
                    }
                }
            }
        }

        HexEditorMessage::SaveIntoRecording => {
            return start_save_into_recording(app);
        }
        HexEditorMessage::SavedIntoRecording(result) => {
            // Re-resolve the editor — the active tab may have changed
            // between dispatch and completion.
            if let Some(editor) = app.state.hex_editors.get_mut(&tab_id) {
                match result {
                    Ok(msg) => {
                        editor.provider.clear_dirty();
                        editor.status_msg = msg;
                    }
                    Err(e) => {
                        editor.status_msg = format!("Save failed: {e}");
                    }
                }
            }
        }
        HexEditorMessage::ClearStatus => {
            editor.status_msg.clear();
        }

        // ── Search & Find/Replace ──────────────────────────────────────
        HexEditorMessage::OpenSearch => {
            editor.search.open();
        }
        HexEditorMessage::Search(query) => {
            editor.search.visible = true;
            editor.search.query = query;
            editor.search.execute(editor.provider.as_slice());
            if let Some(addr) = editor.search.current_addr() {
                editor.selection.select(addr.min(max_addr), max_addr);
            }
        }
        HexEditorMessage::ToggleSearchMode => {
            editor.search.mode = editor.search.mode.toggle();
            if !editor.search.query.is_empty() {
                editor.search.execute(editor.provider.as_slice());
                if let Some(addr) = editor.search.current_addr() {
                    editor.selection.select(addr.min(max_addr), max_addr);
                }
            }
        }
        HexEditorMessage::SearchNext => {
            editor.search.next_match();
            if let Some(addr) = editor.search.current_addr() {
                editor.selection.select(addr.min(max_addr), max_addr);
            }
        }
        HexEditorMessage::SearchPrev => {
            editor.search.prev_match();
            if let Some(addr) = editor.search.current_addr() {
                editor.selection.select(addr.min(max_addr), max_addr);
            }
        }
        HexEditorMessage::CloseSearch => {
            editor.search.clear();
        }

        // ── Goto address ───────────────────────────────────────────────
        HexEditorMessage::OpenGotoDialog => {
            editor.goto = Some(GotoState::new());
        }
        HexEditorMessage::SetGotoDraft(s) => {
            if let Some(ref mut g) = editor.goto {
                g.draft = s;
                g.error = None;
            }
        }
        HexEditorMessage::CommitGoto => {
            let parse_result = editor
                .goto
                .as_ref()
                .map(|g| g.parse(editor.selection.cursor, max_addr));
            match parse_result {
                Some(Ok(addr)) => {
                    editor.selection.select(addr, max_addr);
                    editor.goto = None;
                }
                Some(Err(msg)) => {
                    if let Some(ref mut g) = editor.goto {
                        g.error = Some(msg);
                    }
                }
                None => {}
            }
        }
        HexEditorMessage::CloseGotoDialog => {
            editor.goto = None;
        }

        // ── Pattern highlighting ────────────────────────────────────────
        HexEditorMessage::CreatePattern => {
            if editor.selection.is_single() {
                editor.status_msg = "Select a range of bytes to create a pattern".to_string();
            } else {
                let (start, end) = (editor.selection.start(), editor.selection.end());
                editor.add_pattern(start, end);
                editor.status_msg = format!("Pattern created: 0x{:08X}..0x{:08X}", start, end);
            }
        }
        HexEditorMessage::RemovePatternAt(addr) => {
            if let Some(id) = editor.pattern_id_at(addr) {
                editor.remove_pattern(id);
            }
        }
        HexEditorMessage::ClearAllPatterns => {
            editor.clear_patterns();
            editor.status_msg = "All patterns cleared".to_string();
        }
        HexEditorMessage::RightClickAt(addr) => {
            editor.context_menu_addr = Some(addr);
        }

        // ── Pattern list panel ──────────────────────────────────────────
        HexEditorMessage::TogglePatternList => {
            editor.show_pattern_list = !editor.show_pattern_list;
        }
        HexEditorMessage::NavigateToPattern(id) => {
            if let Some(pat) = editor.pattern_by_id(id) {
                editor.selection.select(pat.start, max_addr);
            }
        }
        HexEditorMessage::RemovePattern(id) => {
            editor.remove_pattern(id);
        }

        // ── Address format ──────────────────────────────────────────────
        HexEditorMessage::ToggleAddrFormat => {
            editor.show_decimal = !editor.show_decimal;
        }
    }
    Task::none()
}

/// Build the async save-into-recording task, or short-circuit with a
/// status message when prerequisites aren't met.
fn start_save_into_recording(app: &mut App) -> Task<crate::message::Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);
    let Some(editor) = app.state.hex_editors.get_mut(&tab_id) else {
        return Task::none();
    };

    let Some(session) = app.state.recording.as_ref() else {
        editor.status_msg = "No active recording — start one in the Mod Manager.".to_string();
        return Task::none();
    };
    let game_path = match app.state.workspace.game_path.clone() {
        Some(p) => p,
        None => {
            editor.status_msg =
                "Game path not set — pick one before saving into a mod.".to_string();
            return Task::none();
        }
    };
    let Ok(relative) = editor.path.strip_prefix(&game_path) else {
        editor.status_msg = format!(
            "File `{}` is outside the active game directory.",
            editor.path.display()
        );
        return Task::none();
    };
    let relative_str = relative.to_string_lossy().replace('\\', "/");

    let workspace_root = session.workspace_root.clone();
    let mod_slug = session.mod_slug.clone();
    let current_bytes = editor.provider.as_slice().to_vec();
    let game_path_for_async: PathBuf = game_path;

    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || -> Result<String, String> {
                build_and_append_action(
                    &workspace_root,
                    &game_path_for_async,
                    &mod_slug,
                    &relative_str,
                    current_bytes,
                )
            })
            .await
            .unwrap_or_else(|e| Err(e.to_string()))
        },
        |result| Message::hex_editor(HexEditorMessage::SavedIntoRecording(result)),
    )
}

/// Pure(-ish) helper: open the workspace, ensure a vanilla snapshot exists,
/// compute a binary delta, and append the resulting [`ChangeAction`].
/// Returns a human-readable summary on success.
fn build_and_append_action(
    workspace_root: &std::path::Path,
    game_dir: &std::path::Path,
    mod_slug: &str,
    relative: &str,
    current_bytes: Vec<u8>,
) -> Result<String, String> {
    let ws = Workspace::open(workspace_root.to_path_buf()).map_err(|e| e.to_string())?;
    let vanilla_bytes = ws
        .vanilla()
        .ensure_snapshot(game_dir, relative)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Vanilla file not present on disk; cannot diff.".to_string())?;

    let op = decide_op(&vanilla_bytes, &current_bytes)?;
    let summary = match &op {
        ChangeOp::BinaryDelta { patch_bytes } => {
            format!(
                "Saved into `{mod_slug}` as BinaryDelta — {} byte patch.",
                patch_bytes.len()
            )
        }
        ChangeOp::FileReplace { content } => {
            format!(
                "Saved into `{mod_slug}` as FileReplace — {} bytes.",
                content.len()
            )
        }
        _ => format!("Saved into `{mod_slug}`."),
    };
    let action = ChangeAction::new(relative, op);
    ws.append_action(mod_slug, action)
        .map_err(|e| e.to_string())?;
    Ok(summary)
}

/// Decide between [`ChangeOp::BinaryDelta`] and [`ChangeOp::FileReplace`]
/// based on the relative size of the qbsdiff patch.
pub fn decide_op(vanilla: &[u8], current: &[u8]) -> Result<ChangeOp, String> {
    let delta = make_delta(vanilla, current).map_err(|e| e.to_string())?;
    let keep_delta =
        !current.is_empty() && (delta.len() as f64) < (current.len() as f64) * DELTA_KEEP_THRESHOLD;
    if keep_delta {
        Ok(ChangeOp::BinaryDelta { patch_bytes: delta })
    } else {
        Ok(ChangeOp::FileReplace {
            content: current.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decide_op_picks_binary_delta_for_small_patches() {
        // Long mostly-identical buffers → small delta.
        let vanilla = vec![0xABu8; 4096];
        let mut current = vanilla.clone();
        current[0] = 0xCD;
        current[100] = 0xEF;
        let op = decide_op(&vanilla, &current).unwrap();
        assert!(matches!(op, ChangeOp::BinaryDelta { .. }));
    }

    #[test]
    fn decide_op_picks_file_replace_when_files_diverge_heavily() {
        // Wholly different buffers → delta is comparable to current size.
        let vanilla: Vec<u8> = (0u8..64).collect();
        let current: Vec<u8> = (0u8..64).map(|b| 255 - b).collect();
        let op = decide_op(&vanilla, &current).unwrap();
        assert!(matches!(op, ChangeOp::FileReplace { .. }));
    }
}
