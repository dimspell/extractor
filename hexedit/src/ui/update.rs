use iced::widget::pane_grid;
use iced::{clipboard, Task};

use crate::config::HexEditorConfig;
use crate::domain::export_config::ExportConfig;
use crate::domain::panel::HexPanel;

use crate::domain::pattern::{RepeatPatternDialog, RepeatedPatternGroup};
use crate::editing::{EditState, InspectorEditState};
use crate::goto::GotoState;
use crate::inspector::ENTRIES;
use crate::message::HexEditorMessage;
use crate::pattern::Pattern;
use crate::search::parse_hex_query;
use crate::selection::nav_target;
use crate::HexProvider;

/// Page nav heuristic — the matrix doesn't propagate live viewport height
/// up here, so PageUp/PageDown approximate a screenful.
const PAGE_ROWS: u64 = 24;

/// How many bytes to read from the cursor position when decoding an inspector
/// value for copy-to-clipboard. 64 bytes covers every built-in inspector
/// entry (largest is u128 + string at 18 bytes) with plenty of headroom.
const INSPECTOR_READ_LIMIT: u64 = 64;

pub fn update(
    state: &mut crate::HexEditorState,
    config: &HexEditorConfig,
    message: HexEditorMessage,
) -> Task<HexEditorMessage> {
    let max_addr = state.max_addr();
    match message {
        // ── Pane grid layout (Halloy-style) ─────────────────────────────
        HexEditorMessage::PaneClicked(pane) => {
            state.pane_focus = pane;
        }
        HexEditorMessage::PaneResized(event) => {
            state.panes.resize(event.split, event.ratio);
        }
        HexEditorMessage::PaneDragged(event) => {
            // Halloy pattern: only the Dropped variant mutates state.
            // Picked / Canceled are no-ops (visual feedback is handled
            // internally by the PaneGrid widget).
            if let pane_grid::DragEvent::Dropped { pane, target } = event {
                state.panes.drop(pane, target);
            }
        }
        HexEditorMessage::SplitPane(axis) => {
            let focus = state.pane_focus;
            let can_split =
                state.panes.len() < 8;
            if can_split {
                let new_panel = HexPanel::new(
                    crate::domain::panel::HexPanelContent::Matrix,
                );
                let _ = state.panes.split(axis, focus, new_panel);
            }
        }
        HexEditorMessage::ClosePane => {
            if state.panes.len() > 1 {
                let focus = state.pane_focus;
                if let Some((_, sibling)) = state.panes.close(focus) {
                    state.pane_focus = sibling;
                }
            }
        }

        HexEditorMessage::SetBytesPerRow(n) => {
            if matches!(n, 8 | 16 | 32) {
                state.bytes_per_row = n;
            }
        }
        HexEditorMessage::SelectAt(addr) => {
            state.selection.select(addr, max_addr);
            state.edit_mode = None;
            state.refresh_active_patterns();
        }
        HexEditorMessage::ExtendTo(addr) => {
            state.selection.extend(addr, max_addr);
            state.refresh_active_patterns();
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
            state.refresh_active_patterns();
        }

        HexEditorMessage::BeginEdit(addr) => {
            if state.provider.is_empty() || !state.provider.is_writable() {
                return Task::none();
            }
            let addr = addr.min(max_addr);
            state.selection.select(addr, max_addr);
            state.edit_mode = Some(EditState::new(addr));
            state.refresh_active_patterns();
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
                    return clipboard::write(decoded);
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
            let Some(ref encode) = entry.encode else {
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
        HexEditorMessage::RemovePatternAtContextMenu => {
            if let Some(addr) = state.context_menu_addr {
                if let Some(id) = state.pattern_id_at(addr) {
                    state.remove_pattern(id);
                }
            }
            state.context_menu_addr = None;
        }
        HexEditorMessage::ClearAllPatterns => {
            state.clear_patterns();
            state.context_menu_addr = None;
            state.status_msg = "All patterns cleared".to_string();
        }
        HexEditorMessage::RightClickAt(addr) => {
            state.context_menu_addr = Some(addr);
        }

        // ── Repeat pattern dialog ────────────────────────────────────────
        HexEditorMessage::BeginRepeatedPattern => {
            if state.selection.is_single() {
                state.status_msg = "Select a range of bytes to repeat".to_string();
            } else {
                let (start, end) = (state.selection.start(), state.selection.end());
                let block_size = end - start + 1;
                state.repeat_pattern =
                    Some(RepeatPatternDialog::new(start, block_size));
                return iced::widget::operation::focus(RepeatPatternDialog::input_id());
            }
        }
        HexEditorMessage::SetRepeatedPatternDraft(s) => {
            if let Some(ref mut dlg) = state.repeat_pattern {
                dlg.draft = s;
                dlg.error = None;
            }
        }
        HexEditorMessage::SetRepeatedPatternLabel(s) => {
            if let Some(ref mut dlg) = state.repeat_pattern {
                dlg.label_draft = s;
            }
        }
        HexEditorMessage::CommitRepeatedPattern => {
            let result = state
                .repeat_pattern
                .as_ref()
                .map(|dlg| dlg.parse_repeat_count());
            match result {
                Some(Ok(count)) => {
                    let dlg = state.repeat_pattern.take().unwrap();
                    let label = if dlg.label_draft.trim().is_empty() {
                        "Unnamed group".to_string()
                    } else {
                        dlg.label_draft.trim().to_string()
                    };

                    // Create the group entry.
                    let group_id = state.next_group_id;
                    state.next_group_id += 1;
                    let group_color = (state.groups.len() % 16) as u8;
                    state.groups.push(RepeatedPatternGroup::new(
                        group_id,
                        label.clone(),
                        group_color,
                    ));

                    let max_addr = state.max_addr();
                    let mut created = 0u64;
                    for i in 0..count {
                        let block_start = dlg.block_start + i * dlg.block_size;
                        if block_start > max_addr {
                            break;
                        }
                        let block_end =
                            (block_start + dlg.block_size - 1).min(max_addr);
                        let id = state.next_pattern_id;
                        state.next_pattern_id += 1;
                        state.patterns.push(Pattern::grouped(
                            id,
                            block_start,
                            block_end,
                            group_color,
                            group_id,
                        ));
                        // Prefill annotation with group label + index so the
                        // hex matrix annotation column identifies each entry.
                        if let Some(pat) = state.patterns.last_mut() {
                            pat.annotation = Some(format!("{}[{}]", label, i));
                        }
                        created += 1;
                    }
                    state.rebuild_pattern_lookup();
                    state.recompute_row_annotations();
                    state.status_msg =
                        format!("Created group \"{label}\" with {created} repetition(s)");
                }
                Some(Err(msg)) => {
                    if let Some(ref mut dlg) = state.repeat_pattern {
                        dlg.error = Some(msg);
                    }
                }
                None => {}
            }
        }
        HexEditorMessage::CloseRepeatedPattern => {
            state.repeat_pattern = None;
        }

        // ── Pattern list panel ──────────────────────────────────────────
        HexEditorMessage::TogglePatternList => {
            // With the pane grid, toggling the pattern list adds or removes
            // a PatternList pane (Halloy-style) rather than showing/hiding a
            // pinned section. Keep the legacy boolean in sync for the toolbar.
            let existing: Option<pane_grid::Pane> = state
                .panes
                .iter()
                .find_map(|(id, panel)| {
                    if panel.content == crate::domain::panel::HexPanelContent::PatternList {
                        Some(id)
                    } else {
                        None
                    }
                })
                .copied();

            if let Some(pane_id) = existing {
                if state.panes.len() > 1 {
                    if let Some((_, sibling)) = state.panes.close(pane_id) {
                        state.pane_focus = sibling;
                    }
                }
                state.show_pattern_list = false;
            } else {
                let focus = state.pane_focus;
                let can_split = state.panes.len() < 8;
                if can_split {
                    let _ = state.panes.split(
                        iced::widget::pane_grid::Axis::Vertical,
                        focus,
                        HexPanel::new(
                            crate::domain::panel::HexPanelContent::PatternList,
                        ),
                    );
                }
                state.show_pattern_list = true;
            }
        }
        HexEditorMessage::NavigateToPattern(id) => {
            if let Some(pat) = state.pattern_by_id(id) {
                state.selection.select(pat.start, max_addr);
            }
            state.refresh_active_patterns();
        }
        HexEditorMessage::RemovePattern(id) => {
            state.remove_pattern(id);
            state.context_menu_addr = None;
        }
        HexEditorMessage::TogglePatternGroup(id) => {
            if !state.collapsed_groups.remove(&id) {
                state.collapsed_groups.insert(id);
            }
        }

        // ── Group operations ────────────────────────────────────────────
        HexEditorMessage::RemovePatternGroup(gid) => {
            state.patterns.retain(|p| p.group_id != Some(gid));
            state.groups.retain(|g| g.id != gid);
            state.collapsed_groups.remove(&gid);
            state.rebuild_pattern_lookup();
            state.recompute_row_annotations();
            state.status_msg = format!("Removed group and {} pattern(s)", state.patterns.len());
        }
        HexEditorMessage::BeginRenameGroup(gid) => {
            state.renaming_group = Some(gid);
            if let Some(grp) = state.groups.iter().find(|g| g.id == gid) {
                state.renaming_group_draft = grp.label.clone();
            }
            return iced::widget::operation::focus(
                iced::widget::Id::new("hex-rename-group-input"),
            );
        }
        HexEditorMessage::SetRenameGroupDraft(s) => {
            state.renaming_group_draft = s;
        }
        HexEditorMessage::CommitRenameGroup => {
            if let Some(gid) = state.renaming_group.take() {
                let label = if state.renaming_group_draft.trim().is_empty() {
                    "Unnamed group"
                } else {
                    state.renaming_group_draft.trim()
                };
                if let Some(grp) = state.groups.iter_mut().find(|g| g.id == gid) {
                    grp.label = label.to_string();
                }
                state.status_msg = format!("Group renamed to \"{label}\"");
            }
            state.renaming_group_draft.clear();
        }
        HexEditorMessage::CancelRenameGroup => {
            state.renaming_group = None;
            state.renaming_group_draft.clear();
        }
        HexEditorMessage::CycleGroupColor(gid) => {
            if let Some(grp) = state.groups.iter_mut().find(|g| g.id == gid) {
                grp.color_idx = (grp.color_idx + 1) % 16;
                for pat in &mut state.patterns {
                    if pat.group_id == Some(gid) {
                        pat.color_idx = grp.color_idx;
                    }
                }
            }
        }
        HexEditorMessage::CyclePatternColor(pid) => {
            if let Some(pat) = state.patterns.iter_mut().find(|p| p.id == pid) {
                pat.color_idx = (pat.color_idx + 1) % 16;
            }
        }

        // ── Pattern annotations ─────────────────────────────────────────
        HexEditorMessage::SetPatternAnnotation(pid, text) => {
            if let Some(pat) = state.patterns.iter_mut().find(|p| p.id == pid) {
                let text = text.trim().to_string();
                pat.annotation = if text.is_empty() { None } else { Some(text) };
                state.recompute_row_annotations();
            }
        }
        HexEditorMessage::ClearPatternAnnotation(pid) => {
            if let Some(pat) = state.patterns.iter_mut().find(|p| p.id == pid) {
                pat.annotation = None;
                state.recompute_row_annotations();
            }
        }

        // ── Pattern import / export ─────────────────────────────────────
        HexEditorMessage::ExportPatterns => {
            let payload = export_patterns_to_text(state);
            if let Some(text) = payload {
                return Task::perform(
                    async move {
                        let path = rfd::AsyncFileDialog::new()
                            .set_title("Export Patterns")
                            .set_file_name("patterns.txt")
                            .add_filter("Pattern files", &["txt", "pat"])
                            .save_file()
                            .await;
                        let Some(path) = path else {
                            return HexEditorMessage::PatternsExported(
                                Err("cancelled".to_string()),
                            );
                        };
                        match tokio::fs::write(path.path(), text).await {
                            Ok(()) => HexEditorMessage::PatternsExported(Ok(())),
                            Err(e) => {
                                HexEditorMessage::PatternsExported(Err(e.to_string()))
                            }
                        }
                    },
                    std::convert::identity,
                );
            }
        }
        HexEditorMessage::ImportPatterns => {
            return Task::perform(
                async move {
                    let path = rfd::AsyncFileDialog::new()
                        .set_title("Import Patterns")
                        .add_filter("Pattern files", &["txt", "pat"])
                        .pick_file()
                        .await;
                    let Some(path) = path else {
                        return HexEditorMessage::PatternsImported(
                            Err("cancelled".to_string()),
                        );
                    };
                    match tokio::fs::read_to_string(path.path()).await {
                        Ok(text) => {
                            HexEditorMessage::PatternsImported(Ok(text))
                        }
                        Err(e) => {
                            HexEditorMessage::PatternsImported(Err(e.to_string()))
                        }
                    }
                },
                std::convert::identity,
            );
        }
        HexEditorMessage::PatternsExported(result) => match result {
            Ok(()) => {
                state.status_msg = "Patterns exported".to_string();
            }
            Err(e) => {
                if e != "cancelled" {
                    state.status_msg = format!("Export failed: {e}");
                }
            }
        },
        HexEditorMessage::PatternsImported(result) => match result {
            Ok(text) => {
                match import_patterns_into(&text, state) {
                    Ok(msg) => {
                        state.rebuild_pattern_lookup();
                        state.recompute_row_annotations();
                        state.status_msg = msg;
                    }
                    Err(e) => {
                        state.status_msg = format!("Import failed: {e}");
                    }
                }
            }
            Err(e) => {
                if e != "cancelled" {
                    state.status_msg = format!("Import failed: {e}");
                }
            }
        },

        // ── Address format ──────────────────────────────────────────────
        HexEditorMessage::ToggleAddrFormat => {
            state.show_decimal = !state.show_decimal;
        }

        // ── Settings modal ──────────────────────────────────────────────
        HexEditorMessage::OpenSettings => {
            state.settings_open = true;
        }
        HexEditorMessage::CloseSettings => {
            state.settings_open = false;
        }
        HexEditorMessage::SetColorScheme(scheme) => {
            state.color_scheme = scheme;
        }
        HexEditorMessage::SetDimNulls(v) => {
            state.dim_nulls = v;
        }

        // ── Copy / Paste ─────────────────────────────────────────────────
        HexEditorMessage::CopySelection => {
            if state.provider.is_empty() {
                return Task::none();
            }
            let start = state.selection.start();
            let end = state.selection.end();
            let bytes = state.provider.read(start..end.saturating_add(1));
            if bytes.is_empty() {
                return Task::none();
            }
            let hex_str = bytes
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            let n = bytes.len();
            state.status_msg = format!("Copied {} byte(s) to clipboard", n);
            return clipboard::write(hex_str);
        }

        HexEditorMessage::Paste => {
            if state.provider.is_empty() {
                return Task::none();
            }
            return clipboard::read().map(|contents| {
                HexEditorMessage::PasteContent(contents.unwrap_or_default())
            });
        }

        HexEditorMessage::PasteContent(contents) => {
            if state.provider.is_empty() {
                return Task::none();
            }
            let bytes = if contents.is_empty() {
                state.status_msg = "Clipboard is empty".to_string();
                return Task::none();
            } else {
                match parse_hex_query(&contents) {
                    Some(b) if !b.is_empty() => b,
                    _ => {
                        state.status_msg =
                            "Clipboard doesn't contain valid hex bytes".to_string();
                        return Task::none();
                    }
                }
            };
            let addr = state.selection.cursor;
            if addr >= state.provider.len() {
                state.status_msg =
                    "Cannot paste: cursor is past end of file".to_string();
                return Task::none();
            }
            state.provider.write(addr, &bytes);
            state.recompute_vanilla_diff();
            state.status_msg = format!("Pasted {} byte(s)", bytes.len());
        }

        // ── Export as text ──────────────────────────────────────────────
        HexEditorMessage::OpenExportConfig => {
            state.export_config = Some(ExportConfig {
                show_address: true,
                address_decimal: state.show_decimal,
                show_ascii: true,
            });
        }

        HexEditorMessage::CloseExportConfig => {
            state.export_config = None;
        }

        HexEditorMessage::SetExportShowAddress(v) => {
            if let Some(ref mut c) = state.export_config {
                c.show_address = v;
            }
        }
        HexEditorMessage::SetExportAddressDecimal(v) => {
            if let Some(ref mut c) = state.export_config {
                c.address_decimal = v;
            }
        }
        HexEditorMessage::SetExportShowAscii(v) => {
            if let Some(ref mut c) = state.export_config {
                c.show_ascii = v;
            }
        }

        HexEditorMessage::CommitExport => {
            let bytes = state.provider.as_slice().to_vec();
            if bytes.is_empty() {
                state.status_msg = "Nothing to export — file is empty".to_string();
                state.export_config = None;
                return Task::none();
            }
            let bpr = state.bytes_per_row;
            let config = state.export_config.clone().unwrap_or_default();
            state.export_config = None; // close modal

            return Task::perform(
                async move {
                    let path = rfd::AsyncFileDialog::new()
                        .set_title("Export as Text")
                        .set_file_name("hex_dump.txt")
                        .add_filter("Text Files", &["txt"])
                        .save_file()
                        .await;
                    let Some(path) = path else {
                        return HexEditorMessage::TextExportCompleted(Err("cancelled".to_string()));
                    };
                    let text = format_hex_dump(&bytes, bpr, &config);
                    match tokio::fs::write(path.path(), text).await {
                        Ok(()) => HexEditorMessage::TextExportCompleted(Ok(())),
                        Err(e) => HexEditorMessage::TextExportCompleted(Err(e.to_string())),
                    }
                },
                std::convert::identity,
            );
        }

        HexEditorMessage::TextExportCompleted(result) => match result {
            Ok(()) => {
                state.status_msg = "Exported as text file".to_string();
            }
            Err(e) => {
                if e != "cancelled" {
                    state.status_msg = format!("Export failed: {e}");
                }
            }
        },
    }
    Task::none()
}

/// Format raw bytes as a human-readable hex dump matching the matrix layout.
///
/// Each line shows: address (hex) · hex bytes grouped 8+8 · ASCII repr.
/// Non-printable bytes (outside 0x20..0x7F) are displayed as `·` (middle dot).
/// The [`ExportConfig`] controls which columns are included and the address
/// format.
///
/// # Example (with default config, `bytes_per_row = 16`)
///
/// ```text
/// 00000000  48 65 6C 6C 6F 20 57 6F  72 6C 64 00 00 00 00 00  Hello World·····
/// 00000010  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F  ················
/// ```
pub fn format_hex_dump(bytes: &[u8], bytes_per_row: u8, config: &ExportConfig) -> String {
    let bpr = bytes_per_row.max(1) as usize;

    // Compute the fixed width of the hex column for a full row.
    // Each byte contributes: `XX` (2 chars) + a trailing separator, except:
    //   - the first byte has no leading separator,
    //   - the last byte has no trailing separator,
    //   - after every 8th byte an extra space is added (group gap).
    // Formula: bpr * 3 - 1 + group_gaps
    // where group_gaps = groups.saturating_sub(1)
    let groups = bpr.div_ceil(8);
    let hex_col_width = bpr * 3 - 1 + groups.saturating_sub(1);

    // Maximum width (in chars) of the address column (including "  " separator).
    let addr_width = if config.show_address {
        if config.address_decimal {
            let max_addr = bytes.len().saturating_sub(1);
            format!("{}", max_addr).len().max(1) + 2
        } else {
            8 + 2 // 8 hex chars + "  "
        }
    } else {
        0
    };

    let mut output = String::with_capacity(
        (addr_width + hex_col_width + 2 + bpr + 1) * bytes.len().div_ceil(bpr),
    );

    for (chunk_idx, chunk) in bytes.chunks(bpr).enumerate() {
        let addr = chunk_idx * bpr;

        // ── Address gutter ────────────────────────────────────────────
        if config.show_address {
            if config.address_decimal {
                output.push_str(&format!("{:>width$}  ", addr, width = addr_width - 2));
            } else {
                output.push_str(&format!("{:08X}  ", addr));
            }
        }

        // ── Hex column ────────────────────────────────────────────────
        let mut hex_part = String::with_capacity(hex_col_width);
        for (col, &b) in chunk.iter().enumerate() {
            if col > 0 {
                if col % 8 == 0 {
                    hex_part.push_str("  "); // double space between groups
                } else {
                    hex_part.push(' ');
                }
            }
            hex_part.push_str(&format!("{:02X}", b));
        }
        output.push_str(&hex_part);

        // Right-pad hex column to fixed width
        for _ in hex_part.len()..hex_col_width {
            output.push(' ');
        }

        // ── ASCII column ──────────────────────────────────────────────
        if config.show_ascii {
            output.push_str("  ");
            for &b in chunk {
                if (0x20..0x7F).contains(&b) {
                    output.push(b as char);
                } else {
                    output.push('·');
                }
            }
        }

        output.push('\n');
    }

    output
}

// ── Pattern import / export format ─────────────────────────────────────
//
// Text format (one record per line):
//   P|<start_hex>|<end_hex>|<color_idx>|<group_id>|<annotation>
//   G|<group_id>|<label>|<color_idx>
// Lines starting with '#' are comments. Start addresses in hex.

/// Serialise current patterns + groups to the custom text format.
pub(crate) fn export_patterns_to_text(state: &crate::HexEditorState) -> Option<String> {
    if state.patterns.is_empty() && state.groups.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(state.patterns.len() * 80);
    out.push_str("# HexEdit Pattern Export v1\n");
    for g in &state.groups {
        out.push_str(&format!(
            "G|{}|{}|{}\n",
            g.id, g.label, g.color_idx
        ));
    }
    for p in &state.patterns {
        let gid = p
            .group_id
            .map(|id| id.to_string())
            .unwrap_or_default();
        let ann = p
            .annotation
            .as_deref()
            .unwrap_or("");
        out.push_str(&format!(
            "P|{:X}|{:X}|{}|{}|{}\n",
            p.start, p.end, p.color_idx, gid, ann
        ));
    }
    Some(out)
}

/// Parse the custom text format and apply patterns/groups to `state`.
/// Returns a status message or an error.
pub(crate) fn import_patterns_into(
    text: &str,
    state: &mut crate::HexEditorState,
) -> Result<String, String> {
    use std::collections::HashMap;

    let mut group_map: HashMap<usize, usize> = HashMap::new();
    let mut pattern_count = 0usize;

    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();

        match parts.first() {
            Some(&"G") if parts.len() >= 3 => {
                let gid: usize = parts[1]
                    .parse()
                    .map_err(|_| format!("line {}: invalid group id", lineno + 1))?;
                let label = parts[2].to_string();
                let color_idx: u8 = parts
                    .get(3)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let new_id = state.next_group_id;
                state.next_group_id += 1;
                group_map.insert(gid, new_id);
                state
                    .groups
                    .push(RepeatedPatternGroup::new(new_id, label, color_idx));
            }
            Some(&"P") if parts.len() >= 4 => {
                let start = u64::from_str_radix(parts[1], 16)
                    .map_err(|_| format!("line {}: invalid start address", lineno + 1))?;
                let end = u64::from_str_radix(parts[2], 16)
                    .map_err(|_| format!("line {}: invalid end address", lineno + 1))?;
                let color_idx: u8 = parts[3]
                    .parse()
                    .map_err(|_| format!("line {}: invalid color index", lineno + 1))?;
                let mapped_gid = parts
                    .get(4)
                    .and_then(|s| s.parse::<usize>().ok())
                    .and_then(|old| group_map.get(&old).copied());
                let annotation = parts
                    .get(5)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());
                let id = state.next_pattern_id;
                state.next_pattern_id += 1;
                let mut pat =
                    Pattern::new(id, start.min(end), start.max(end), color_idx % 16);
                pat.group_id = mapped_gid;
                pat.annotation = annotation;
                state.patterns.push(pat);
                pattern_count += 1;
            }
            Some(t) if *t == "G" || *t == "P" => {
                return Err(format!("line {}: too few fields", lineno + 1));
            }
            _ => {
                return Err(format!("line {}: unknown type '{}'", lineno + 1, parts[0]));
            }
        }
    }

    if pattern_count == 0 {
        return Err("No patterns found in import file".to_string());
    }
    Ok(format!("Imported {pattern_count} pattern(s)"))
}

#[cfg(test)]
mod tests {
    use super::format_hex_dump;
    use crate::domain::export_config::ExportConfig;

    fn default_config() -> ExportConfig {
        ExportConfig::default()
    }

    /// Config with addresses hidden.
    fn no_addr_config() -> ExportConfig {
        ExportConfig {
            show_address: false,
            address_decimal: false,
            show_ascii: true,
        }
    }

    /// Config with no ASCII column.
    fn no_ascii_config() -> ExportConfig {
        ExportConfig {
            show_address: true,
            address_decimal: false,
            show_ascii: false,
        }
    }

    /// Config with decimal addresses.
    fn decimal_addr_config() -> ExportConfig {
        ExportConfig {
            show_address: true,
            address_decimal: true,
            show_ascii: true,
        }
    }

    #[test]
    fn format_empty_bytes() {
        assert_eq!(format_hex_dump(&[], 16, &default_config()), "");
    }

    #[test]
    fn format_single_row() {
        let bytes = b"Hello World";
        let result = format_hex_dump(bytes, 16, &default_config());
        assert!(result.starts_with("00000000  48 65 6C 6C 6F 20 57 6F  72 6C 64"));
        assert!(result.contains("Hello World"));
        assert!(result.ends_with("\n"));
    }

    #[test]
    fn format_two_rows() {
        let bytes: Vec<u8> = (0..32).collect();
        let result = format_hex_dump(&bytes, 16, &default_config());
        let expected = "\
00000000  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F  ················
00000010  10 11 12 13 14 15 16 17  18 19 1A 1B 1C 1D 1E 1F  ················
";
        assert_eq!(result, expected);
    }

    #[test]
    fn format_partial_last_row() {
        let bytes: Vec<u8> = (0..20).collect();
        let result = format_hex_dump(&bytes, 16, &default_config());
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("00000000"));
        assert!(lines[1].starts_with("00000010"));
        assert!(lines[1].contains("10 11 12 13"));
    }

    #[test]
    fn format_bpr_8() {
        let bytes: Vec<u8> = (0..8).collect();
        let result = format_hex_dump(&bytes, 8, &default_config());
        assert!(result.contains("00 01 02 03 04 05 06 07"));
    }

    #[test]
    fn format_bpr_32() {
        let bytes: Vec<u8> = (0..64).collect();
        let result = format_hex_dump(&bytes, 32, &default_config());
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("00000000"));
        assert!(lines[1].starts_with("00000020"));
    }

    #[test]
    fn format_non_printable_shows_dot() {
        let bytes = [0x00, 0x1F, 0x7F, 0x80, 0xFF];
        let result = format_hex_dump(&bytes, 16, &default_config());
        let ascii_part = result.split("  ").last().unwrap_or("");
        assert_eq!(ascii_part.trim_end_matches('\n'), "·····");
    }

    #[test]
    fn format_printable_preserved() {
        let bytes = b"ABC123!@#";
        let result = format_hex_dump(bytes, 16, &default_config());
        let ascii_part = result.split("  ").last().unwrap_or("");
        assert!(ascii_part.contains("ABC123!@#"));
    }

    #[test]
    fn format_hex_colums_align() {
        let bytes: Vec<u8> = (0..40).collect();
        let result = format_hex_dump(&bytes, 16, &default_config());
        for (i, line) in result.lines().enumerate() {
            assert!(
                line.len() >= 60,
                "row {i}: line too short (len={})",
                line.len()
            );
            assert_eq!(&line[58..60], "  ",
                "row {i}: hex/ASCII separator not at expected position 58");
        }
    }

    // ── Config-variant tests ───────────────────────────────────────────

    #[test]
    fn format_no_address_gutter() {
        let bytes = b"Hello World";
        let result = format_hex_dump(bytes, 16, &no_addr_config());
        // No address prefix — should start directly with hex
        assert!(!result.starts_with("00000000"));
        assert!(result.starts_with("48 65 6C"));
        assert!(result.contains("Hello World"));
    }

    #[test]
    fn format_no_ascii() {
        let bytes: Vec<u8> = (0..16).collect();
        let result = format_hex_dump(&bytes, 16, &no_ascii_config());
        // Should have the address and hex but no ASCII column
        assert!(result.starts_with("00000000"));
        assert!(result.contains("00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F"));
        // No dots (middle-dot characters) because ASCII column is gone
        assert!(!result.contains('·'));
        // Line should end at hex column + newline (no ASCII)
        assert!(result.ends_with("\n"));
    }

    #[test]
    fn format_decimal_addresses() {
        let bytes: Vec<u8> = (0..32).collect();
        let result = format_hex_dump(&bytes, 16, &decimal_addr_config());
        // First address is 0 — right-aligned in width, then "  "
        assert!(result.starts_with(" 0  "));
        // Second row starts at address 16 (decimal)
        assert!(result.contains("16  10 11 12 13 14 15 16 17"));
        // Should still have hex and ASCII columns
        assert!(result.contains("00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F"));
        assert!(result.contains('·'));
    }

    #[test]
    fn format_only_hex_column() {
        let bytes = b"\x00\x01\x02\x03";
        let result = format_hex_dump(bytes, 16, &ExportConfig {
            show_address: false,
            address_decimal: false,
            show_ascii: false,
        });
        // Just the hex values, nothing else
        assert_eq!(result, "00 01 02 03                                     \n");
    }
}
