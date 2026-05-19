use iced::Task;

use super::config::HexEditorConfig;
use super::editing::{EditState, InspectorEditState};
use super::goto::GotoState;
use super::inspector::ENTRIES;
use super::message::HexEditorMessage;
use super::selection::nav_target;
use super::HexProvider;

/// Page nav heuristic — the matrix doesn't propagate live viewport height
/// up here, so PageUp/PageDown approximate a screenful.
const PAGE_ROWS: u64 = 24;

/// How many bytes to read from the cursor position when decoding an inspector
/// value for copy-to-clipboard. 64 bytes covers every built-in inspector
/// entry (largest is u128 + string at 18 bytes) with plenty of headroom.
const INSPECTOR_READ_LIMIT: u64 = 64;

pub fn update(
    state: &mut super::HexEditorState,
    config: &HexEditorConfig,
    message: HexEditorMessage,
) -> Task<HexEditorMessage> {
    let max_addr = state.max_addr();
    match message {
        HexEditorMessage::SetBytesPerRow(n) => {
            if matches!(n, 8 | 16 | 32) {
                state.bytes_per_row = n;
            }
        }
        HexEditorMessage::SelectAt(addr) => {
            state.selection.select(addr, max_addr);
            state.edit_mode = None;
        }
        HexEditorMessage::ExtendTo(addr) => {
            state.selection.extend(addr, max_addr);
        }
        HexEditorMessage::Nav { dir, extend } => {
            if state.provider.is_empty() {
                return Task::none();
            }
            let bpr = state.bytes_per_row as u64;
            let target = nav_target(state.selection.cursor, dir, bpr, PAGE_ROWS, max_addr);
            if extend {
                state.selection.extend(target, max_addr);
            } else {
                state.selection.select(target, max_addr);
            }
            state.edit_mode = None;
        }

        HexEditorMessage::BeginEdit(addr) => {
            if state.provider.is_empty() || !state.provider.is_writable() {
                return Task::none();
            }
            let addr = addr.min(max_addr);
            state.selection.select(addr, max_addr);
            state.edit_mode = Some(EditState::new(addr));
        }
        HexEditorMessage::EditTypeChar(c) => {
            if state.provider.is_empty() {
                return Task::none();
            }
            let edit_addr = match state.edit_mode {
                Some(ref e) => e.addr,
                None => state.selection.cursor,
            };
            let edit = state
                .edit_mode
                .get_or_insert_with(|| EditState::new(edit_addr));
            if !edit.push_char(c) {
                return Task::none();
            }
            let staged = edit.is_complete().then(|| (edit.addr, edit.staged_byte()));
            if let Some((addr, byte)) = staged {
                if let Some(byte) = byte {
                    state.provider.write(addr, &[byte]);
                    state.recompute_vanilla_diff();
                }
                let next = (addr + 1).min(max_addr);
                if next == addr {
                    state.edit_mode = None;
                } else {
                    state.selection.select(next, max_addr);
                    state.edit_mode = Some(EditState::new(next));
                }
            }
        }
        HexEditorMessage::EditBackspace => {
            if let Some(ref mut e) = state.edit_mode {
                e.pop_char();
            }
        }
        HexEditorMessage::EditCancel => {
            state.edit_mode = None;
        }
        HexEditorMessage::EditCommit { advance } => {
            if let Some(edit) = state.edit_mode.take() {
                if let Some(byte) = edit.staged_byte() {
                    state.provider.write(edit.addr, &[byte]);
                    state.recompute_vanilla_diff();
                }
                if advance {
                    let next = (edit.addr + 1).min(max_addr);
                    state.selection.select(next, max_addr);
                    if next > edit.addr {
                        state.edit_mode = Some(EditState::new(next));
                    }
                } else {
                    state.selection.select(edit.addr, max_addr);
                }
            }
        }

        HexEditorMessage::WriteBytes { addr, bytes } => {
            if !state.provider.is_empty() {
                state.provider.write(addr, &bytes);
                state.recompute_vanilla_diff();
            }
        }

        // ── Inspector ───────────────────────────────────────────────────
        HexEditorMessage::CopyInspectorValue(idx) => {
            let cursor = state.selection.cursor;
            let len = state.provider.len();
            let read_end = (cursor + INSPECTOR_READ_LIMIT).min(len);
            let bytes = state.provider.read(cursor..read_end);
            let entry = if idx < ENTRIES.len() {
                ENTRIES.get(idx)
            } else {
                config.extra_entries.get(idx - ENTRIES.len())
            };
            if let Some(entry) = entry {
                if len - cursor >= entry.min_size as u64 {
                    let decoded = (entry.decode)(bytes);
                    state.status_msg = format!("Copied: {decoded}");
                }
            }
        }

        HexEditorMessage::BeginInspectorEdit(idx) => {
            if state.provider.is_empty() {
                return Task::none();
            }
            let entry = if idx < ENTRIES.len() {
                ENTRIES.get(idx)
            } else {
                config.extra_entries.get(idx - ENTRIES.len())
            };
            let Some(entry) = entry else {
                return Task::none();
            };
            if entry.encode.is_none() {
                return Task::none();
            }
            let cursor = state.selection.cursor;
            let len = state.provider.len();
            if cursor + entry.min_size as u64 > len {
                return Task::none();
            }
            let bytes = state.provider.read(cursor..cursor + entry.min_size as u64);
            let initial = (entry.decode)(bytes);
            let initial = initial
                .split_once(' ')
                .map(|(lhs, _)| lhs.to_string())
                .unwrap_or(initial);
            state.inspector_edit = Some(InspectorEditState::new(idx, cursor, initial));
        }
        HexEditorMessage::SetInspectorDraft(s) => {
            if let Some(ref mut ie) = state.inspector_edit {
                ie.draft = s;
                ie.error = None;
            }
        }
        HexEditorMessage::CloseInspectorEdit => {
            state.inspector_edit = None;
        }
        HexEditorMessage::CommitInspectorEdit => {
            let Some(ref ie) = state.inspector_edit else {
                return Task::none();
            };
            let entry = if ie.entry_idx < ENTRIES.len() {
                ENTRIES.get(ie.entry_idx)
            } else {
                config.extra_entries.get(ie.entry_idx - ENTRIES.len())
            };
            let Some(entry) = entry else {
                state.inspector_edit = None;
                return Task::none();
            };
            let Some(encode) = entry.encode else {
                state.inspector_edit = None;
                return Task::none();
            };
            match encode(&ie.draft) {
                Ok(bytes) => {
                    let addr = ie.addr;
                    state.provider.write(addr, &bytes);
                    state.recompute_vanilla_diff();
                    state.inspector_edit = None;
                }
                Err(msg) => {
                    if let Some(ref mut ie) = state.inspector_edit {
                        ie.error = Some(msg);
                    }
                }
            }
        }

        // ── Save ────────────────────────────────────────────────────────
        HexEditorMessage::SaveIntoRecording => {
            if let Some(ref on_save) = config.on_save {
                return on_save(state);
            }
            state.status_msg = "Save not available.".to_string();
        }
        HexEditorMessage::SavedIntoRecording(result) => match result {
            Ok(msg) => {
                state.provider.clear_dirty();
                state.status_msg = msg;
            }
            Err(e) => {
                state.status_msg = format!("Save failed: {e}");
            }
        },
        HexEditorMessage::ClearStatus => {
            state.status_msg.clear();
        }

        // ── Search & Find/Replace ──────────────────────────────────────
        HexEditorMessage::OpenSearch => {
            state.search.open();
        }
        HexEditorMessage::Search(query) => {
            state.search.visible = true;
            state.search.query = query;
            state.search.execute(state.provider.as_slice());
            if let Some(addr) = state.search.current_addr() {
                state.selection.select(addr.min(max_addr), max_addr);
            }
        }
        HexEditorMessage::ToggleSearchMode => {
            state.search.mode = state.search.mode.toggle();
            if !state.search.query.is_empty() {
                state.search.execute(state.provider.as_slice());
                if let Some(addr) = state.search.current_addr() {
                    state.selection.select(addr.min(max_addr), max_addr);
                }
            }
        }
        HexEditorMessage::SearchNext => {
            state.search.next_match();
            if let Some(addr) = state.search.current_addr() {
                state.selection.select(addr.min(max_addr), max_addr);
            }
        }
        HexEditorMessage::SearchPrev => {
            state.search.prev_match();
            if let Some(addr) = state.search.current_addr() {
                state.selection.select(addr.min(max_addr), max_addr);
            }
        }
        HexEditorMessage::CloseSearch => {
            state.search.clear();
        }

        // ── Goto address ───────────────────────────────────────────────
        HexEditorMessage::OpenGotoDialog => {
            state.goto = Some(GotoState::new());
            return iced::widget::operation::focus(GotoState::input_id());
        }
        HexEditorMessage::SetGotoDraft(s) => {
            if let Some(ref mut g) = state.goto {
                g.draft = s;
                g.error = None;
            }
        }
        HexEditorMessage::CommitGoto => {
            let parse_result = state
                .goto
                .as_ref()
                .map(|g| g.parse(state.selection.cursor, max_addr));
            match parse_result {
                Some(Ok(addr)) => {
                    state.selection.select(addr, max_addr);
                    state.goto = None;
                }
                Some(Err(msg)) => {
                    if let Some(ref mut g) = state.goto {
                        g.error = Some(msg);
                    }
                }
                None => {}
            }
        }
        HexEditorMessage::CloseGotoDialog => {
            state.goto = None;
        }

        // ── Pattern highlighting ────────────────────────────────────────
        HexEditorMessage::CreatePattern => {
            if state.selection.is_single() {
                state.status_msg = "Select a range of bytes to create a pattern".to_string();
            } else {
                let (start, end) = (state.selection.start(), state.selection.end());
                state.add_pattern(start, end);
                state.status_msg = format!("Pattern created: 0x{:08X}..0x{:08X}", start, end);
            }
        }
        HexEditorMessage::RemovePatternAt(addr) => {
            if let Some(id) = state.pattern_id_at(addr) {
                state.remove_pattern(id);
            }
        }
        HexEditorMessage::ClearAllPatterns => {
            state.clear_patterns();
            state.status_msg = "All patterns cleared".to_string();
        }
        HexEditorMessage::RightClickAt(addr) => {
            state.context_menu_addr = Some(addr);
        }

        // ── Pattern list panel ──────────────────────────────────────────
        HexEditorMessage::TogglePatternList => {
            state.show_pattern_list = !state.show_pattern_list;
        }
        HexEditorMessage::NavigateToPattern(id) => {
            if let Some(pat) = state.pattern_by_id(id) {
                state.selection.select(pat.start, max_addr);
            }
        }
        HexEditorMessage::RemovePattern(id) => {
            state.remove_pattern(id);
        }

        // ── Address format ──────────────────────────────────────────────
        HexEditorMessage::ToggleAddrFormat => {
            state.show_decimal = !state.show_decimal;
        }
    }
    Task::none()
}
