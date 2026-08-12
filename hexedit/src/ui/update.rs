use iced::widget::pane_grid;
use iced::{Task, clipboard};

use crate::config::HexEditorConfig;
use crate::domain::byte_stats::{compute_row_entropies, compute_statistics};
use crate::domain::export_config::ExportConfig;
use crate::domain::panel::HexPanel;
use crate::domain::write_mode::{encode_text, is_text_mode, remap_write_mode};
use crate::state::{ComparisonFile, InspectorSource};
use crate::ui::coloring::ColorScheme;
use crate::ui::theme::ThemeVariant;

use crate::HexProvider;
use crate::domain::pattern::{RepeatPatternDialog, RepeatedPatternGroup};
use crate::editing::{EditState, InspectorEditState};
use crate::goto::GotoState;
use crate::inspector::ENTRIES;
use crate::message::HexEditorMessage;
use crate::pattern::Pattern;
use crate::search::{SearchMode, parse_hex_query};
use crate::selection::nav_target;

/// Page nav heuristic — the matrix doesn't propagate live viewport height
/// up here, so PageUp/PageDown approximate a screenful.
const PAGE_ROWS: u64 = 24;

/// How many bytes to read from the cursor position when decoding an inspector
/// value for copy-to-clipboard. 64 bytes covers every built-in inspector
/// entry (largest is u128 + string at 18 bytes) with plenty of headroom.
const INSPECTOR_READ_LIMIT: u64 = 64;

/// Returns `(length, bytes)` of the buffer the inspector currently decodes.
fn inspector_source_bytes(state: &crate::HexEditorState) -> (u64, &[u8]) {
    match state.inspector_source {
        InspectorSource::Baseline => (state.provider.len(), state.provider.as_slice()),
        InspectorSource::Comparison => {
            let data = state
                .comparison_file
                .as_ref()
                .map(|cf| cf.data.as_slice())
                .unwrap_or(&[]);
            (data.len() as u64, data)
        }
    }
}

/// Parse a bytes-per-row draft from the settings modal's custom text input.
///
/// Returns the value only if it trims to a `u8` within
/// `MIN_BYTES_PER_ROW..=MAX_BYTES_PER_ROW` (1–64); `None` for non-numeric,
/// empty, or out-of-range drafts.
pub fn parse_bpr(draft: &str) -> Option<u8> {
    let n = draft.trim().parse::<u8>().ok()?;
    (crate::state::MIN_BYTES_PER_ROW..=crate::state::MAX_BYTES_PER_ROW)
        .contains(&n)
        .then_some(n)
}

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
            let can_split = state.panes.len() < 8;
            if can_split {
                let new_panel = HexPanel::new(crate::domain::panel::HexPanelContent::Matrix);
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
            if (crate::state::MIN_BYTES_PER_ROW..=crate::state::MAX_BYTES_PER_ROW).contains(&n) {
                state.bytes_per_row = n;
                // Keep the settings-modal draft in sync with the active value
                // so the text input mirrors preset-button clicks too.
                state.bpr_input = n.to_string();
                state.invalidate_stats();
                // Row boundaries shift with the row width, so pattern
                // annotations keyed by `row * bpr` must be re-laid-out.
                state.recompute_row_annotations();
                // Recompute row entropies immediately so the gutter band stays
                // visible after a row-width change.
                if !state.provider.is_empty() {
                    state.row_entropies = Some(compute_row_entropies(state.provider.as_slice(), n));
                }
            }
        }
        HexEditorMessage::BytesPerRowInputChanged(s) => {
            state.bpr_input = s;
        }
        HexEditorMessage::BytesPerRowInputInvalid => {
            state.notify(format!(
                "Bytes per row must be {}–{}",
                crate::state::MIN_BYTES_PER_ROW,
                crate::state::MAX_BYTES_PER_ROW
            ));
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

            if is_text_mode(state.write_mode) {
                // ── Text mode: encode & write immediately ──────────────
                let text: String = c.into();
                let encoded = encode_text(&text, state.write_mode, &state.custom_encodings);
                if encoded.is_empty() {
                    state.notify(format!(
                        "Cannot encode '{c}' in {} mode",
                        state.write_mode.label(),
                    ));
                    return Task::none();
                }
                let addr = state.selection.cursor;
                state.provider.write(addr, &encoded);
                state.recompute_vanilla_diff();
                let next = addr.saturating_add(encoded.len() as u64).min(max_addr);
                state.selection.select(next, max_addr);
            } else {
                // ── Hex mode: draft-based (existing behaviour) ─────────
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
        }
        HexEditorMessage::EditBackspace => {
            if let Some(ref mut e) = state.edit_mode {
                e.pop_char();
            } else if is_text_mode(state.write_mode) {
                // Text mode has no draft — move cursor left.
                let prev = state.selection.cursor.saturating_sub(1);
                state.selection.select(prev, state.max_addr());
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

        HexEditorMessage::DeleteByteAtCursor => {
            if !state.provider.is_empty() && is_text_mode(state.write_mode) {
                let addr = state.selection.cursor;
                state.provider.write(addr, &[0x00]);
                state.recompute_vanilla_diff();
                let next = (addr + 1).min(max_addr);
                state.selection.select(next, max_addr);
            }
        }

        HexEditorMessage::WriteBytes { addr, bytes } => {
            if !state.provider.is_empty() {
                state.provider.write(addr, &bytes);
                state.recompute_vanilla_diff();
            }
        }

        // ── Inspector ───────────────────────────────────────────────────
        HexEditorMessage::SetInspectorSource(source) => {
            if state.inspector_source != source {
                state.inspector_source = source;
                // The edit modal (if open) holds a value decoded from the
                // previous source — close it to avoid writing stale data.
                state.inspector_edit = None;
            }
        }

        HexEditorMessage::CopyInspectorValue(idx) => {
            let cursor = state.selection.cursor;
            let (len, src) = inspector_source_bytes(state);
            let read_end = (cursor + INSPECTOR_READ_LIMIT).min(len);
            let start = (cursor as usize).min(src.len());
            let end = (read_end as usize).min(src.len()).max(start);
            let bytes = &src[start..end];
            let entry = if idx < ENTRIES.len() {
                ENTRIES.get(idx)
            } else {
                config.extra_entries.get(idx - ENTRIES.len())
            };
            if let Some(entry) = entry
                && len - cursor >= entry.min_size as u64
            {
                let decoded = (entry.decode)(bytes);
                state.notify(format!("Copied: {decoded}"));
                return clipboard::write(decoded).map(|_| HexEditorMessage::ClipboardWriteResult);
            }
        }

        HexEditorMessage::BeginInspectorEdit(idx) => {
            if state.provider.is_empty() || state.inspector_source != InspectorSource::Baseline {
                // The comparison file is read-only — edits only apply to the
                // main buffer, so the edit modal is unavailable there.
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

        // ── Byte statistics / entropy panel ───────────────────────────────
        HexEditorMessage::ToggleStats => {
            // Check if a Statistics pane exists in the grid.
            let existing: Option<pane_grid::Pane> = state
                .panes
                .iter()
                .find_map(|(id, panel)| {
                    if panel.content == crate::domain::panel::HexPanelContent::Statistics {
                        Some(id)
                    } else {
                        None
                    }
                })
                .copied();

            if let Some(pane_id) = existing {
                if state.panes.len() > 1
                    && let Some((_, sibling)) = state.panes.close(pane_id)
                {
                    state.pane_focus = sibling;
                }
                state.show_stats = false;
            } else {
                let focus = state.pane_focus;
                let can_split = state.panes.len() < 8;
                if can_split {
                    let _ = state.panes.split(
                        iced::widget::pane_grid::Axis::Vertical,
                        focus,
                        HexPanel::new(crate::domain::panel::HexPanelContent::Statistics),
                    );
                }
                state.show_stats = true;
                // If stats haven't been computed yet, trigger analysis.
                if state.file_stats.is_none() && !state.provider.is_empty() {
                    let bytes = state.provider.as_slice().to_vec();
                    let bpr = state.bytes_per_row;
                    return Task::perform(
                        async move {
                            let stats = compute_statistics(&bytes);
                            let entropies = compute_row_entropies(&bytes, bpr);
                            HexEditorMessage::FileAndRowEntropiesComputed(
                                Box::new(stats),
                                Box::new(entropies),
                            )
                        },
                        std::convert::identity,
                    );
                }
            }
        }
        HexEditorMessage::AnalyzeFile => {
            if !state.provider.is_empty() {
                let bytes = state.provider.as_slice().to_vec();
                let bpr = state.bytes_per_row;
                return Task::perform(
                    async move {
                        let stats = compute_statistics(&bytes);
                        let entropies = compute_row_entropies(&bytes, bpr);
                        HexEditorMessage::FileAndRowEntropiesComputed(
                            Box::new(stats),
                            Box::new(entropies),
                        )
                    },
                    std::convert::identity,
                );
            }
        }
        HexEditorMessage::AnalyzeSelection => {
            if !state.provider.is_empty() && !state.selection.is_single() {
                let start = state.selection.start();
                let end = state.selection.end();
                let bytes = state.provider.read(start..end.saturating_add(1)).to_vec();
                return Task::perform(
                    async move {
                        let stats = compute_statistics(&bytes);
                        HexEditorMessage::SelectionAnalyzed(Box::new(stats))
                    },
                    std::convert::identity,
                );
            }
        }
        HexEditorMessage::FileAndRowEntropiesComputed(stats, entropies) => {
            state.file_stats = Some(*stats);
            state.row_entropies = Some(*entropies);
        }
        HexEditorMessage::SelectionAnalyzed(stats) => {
            state.selection_stats = Some(*stats);
        }

        // ── Save ────────────────────────────────────────────────────────
        HexEditorMessage::SaveIntoRecording => {
            if let Some(ref on_save) = config.on_save {
                return on_save(state);
            }
            state.notify("Save not available.");
        }
        HexEditorMessage::SavedIntoRecording(result) => match result {
            Ok(msg) => {
                state.provider.clear_dirty();
                state.notify(msg);
            }
            Err(e) => {
                state.notify(format!("Save failed: {e}"));
            }
        },
        HexEditorMessage::ClearStatus => {
            state.clear_notifications();
        }
        HexEditorMessage::DismissNotification(index) => {
            state.dismiss_notification(index);
        }
        HexEditorMessage::ClipboardWriteResult => {}

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
                state.pending_center_on.set(Some(addr.min(max_addr)));
            }
        }
        HexEditorMessage::ToggleSearchMode => {
            state.search.mode = state.search.mode.toggle();
            if !state.search.query.is_empty() {
                state.search.execute(state.provider.as_slice());
                if let Some(addr) = state.search.current_addr() {
                    state.selection.select(addr.min(max_addr), max_addr);
                    state.pending_center_on.set(Some(addr.min(max_addr)));
                }
            }
        }
        HexEditorMessage::SearchNext => {
            state.search.next_match();
            if let Some(addr) = state.search.current_addr() {
                state.selection.select(addr.min(max_addr), max_addr);
                state.pending_center_on.set(Some(addr.min(max_addr)));
            }
        }
        HexEditorMessage::SearchPrev => {
            state.search.prev_match();
            if let Some(addr) = state.search.current_addr() {
                state.selection.select(addr.min(max_addr), max_addr);
                state.pending_center_on.set(Some(addr.min(max_addr)));
            }
        }
        HexEditorMessage::CloseSearch => {
            state.search.clear();
        }
        HexEditorMessage::SetSearchWidth(width) => {
            if !matches!(width, 1 | 2 | 4 | 8) {
                return Task::none();
            }
            state.search.width = width;
            if state.search.mode == SearchMode::Decimal && !state.search.query.is_empty() {
                state.search.execute(state.provider.as_slice());
                if let Some(addr) = state.search.current_addr() {
                    state.selection.select(addr.min(max_addr), max_addr);
                    state.pending_center_on.set(Some(addr.min(max_addr)));
                }
            }
        }
        HexEditorMessage::ToggleSearchEndian => {
            state.search.little_endian = !state.search.little_endian;
            if state.search.mode == SearchMode::Decimal && !state.search.query.is_empty() {
                state.search.execute(state.provider.as_slice());
                if let Some(addr) = state.search.current_addr() {
                    state.selection.select(addr.min(max_addr), max_addr);
                    state.pending_center_on.set(Some(addr.min(max_addr)));
                }
            }
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
                    state.pending_center_on.set(Some(addr));
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
                state.notify("Select a range of bytes to create a pattern");
            } else {
                let (start, end) = (state.selection.start(), state.selection.end());
                state.add_pattern(start, end);
                state.notify(format!("Pattern created: 0x{:08X}..0x{:08X}", start, end));
            }
        }
        HexEditorMessage::RemovePatternAt(addr) => {
            if let Some(id) = state.pattern_id_at(addr) {
                state.remove_pattern(id);
            }
            state.context_menu_addr = None;
        }
        HexEditorMessage::RemovePatternAtContextMenu => {
            if let Some(addr) = state.context_menu_addr
                && let Some(id) = state.pattern_id_at(addr)
            {
                state.remove_pattern(id);
            }
            state.context_menu_addr = None;
        }
        HexEditorMessage::ClearAllPatterns => {
            state.clear_patterns();
            state.context_menu_addr = None;
            state.notify("All patterns cleared");
        }
        HexEditorMessage::RightClickAt(addr) => {
            state.context_menu_addr = Some(addr);
        }

        // ── Repeat pattern dialog ────────────────────────────────────────
        HexEditorMessage::BeginRepeatedPattern => {
            if state.selection.is_single() {
                state.notify("Select a range of bytes to repeat");
            } else {
                let (start, end) = (state.selection.start(), state.selection.end());
                let block_size = end - start + 1;
                state.repeat_pattern = Some(RepeatPatternDialog::new(start, block_size));
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
                        let block_end = (block_start + dlg.block_size - 1).min(max_addr);
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
                    state.notify(format!(
                        "Created group \"{label}\" with {created} repetition(s)"
                    ));
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
        HexEditorMessage::ToggleInspector => {
            let existing: Option<pane_grid::Pane> = state
                .panes
                .iter()
                .find_map(|(id, panel)| {
                    if panel.content == crate::domain::panel::HexPanelContent::Inspector {
                        Some(id)
                    } else {
                        None
                    }
                })
                .copied();

            if let Some(pane_id) = existing {
                if state.panes.len() > 1
                    && let Some((_, sibling)) = state.panes.close(pane_id)
                {
                    state.pane_focus = sibling;
                }
            } else {
                let focus = state.pane_focus;
                let can_split = state.panes.len() < 8;
                if can_split
                    && let Some((_, split)) = state.panes.split(
                        iced::widget::pane_grid::Axis::Vertical,
                        focus,
                        HexPanel::new(crate::domain::panel::HexPanelContent::Inspector),
                    )
                {
                    state.panes.resize(split, 0.75);
                }
            }
        }
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
                if state.panes.len() > 1
                    && let Some((_, sibling)) = state.panes.close(pane_id)
                {
                    state.pane_focus = sibling;
                }
                state.show_pattern_list = false;
            } else {
                let focus = state.pane_focus;
                let can_split = state.panes.len() < 8;
                if can_split {
                    let _ = state.panes.split(
                        iced::widget::pane_grid::Axis::Vertical,
                        focus,
                        HexPanel::new(crate::domain::panel::HexPanelContent::PatternList),
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
            let before = state.patterns.len();
            state.patterns.retain(|p| p.group_id != Some(gid));
            let removed = before - state.patterns.len();
            state.groups.retain(|g| g.id != gid);
            state.collapsed_groups.remove(&gid);
            state.rebuild_pattern_lookup();
            state.recompute_row_annotations();
            state.context_menu_addr = None;
            state.notify(format!("Removed group and {removed} pattern(s)"));
        }
        HexEditorMessage::BeginRenameGroup(gid) => {
            state.renaming_group = Some(gid);
            if let Some(grp) = state.groups.iter().find(|g| g.id == gid) {
                state.renaming_group_draft = grp.label.clone();
            }
            return iced::widget::operation::focus(iced::widget::Id::from(format!(
                "hex-rename-group-input-{gid}"
            )));
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
                    let old_label = std::mem::take(&mut grp.label);
                    grp.label = label.to_string();

                    // Update child pattern annotations whose auto-generated
                    // prefix matches the old label, e.g. "Monster[0]" → "Enemy[0]".
                    // Only update annotations matching `{old_label}[{digit}+]`
                    // to avoid overwriting manual edits.
                    if !old_label.is_empty() {
                        let old_prefix = format!("{}[", old_label);
                        let new_prefix = format!("{}[", grp.label);
                        for pat in &mut state.patterns {
                            if pat.group_id == Some(gid)
                                && let Some(ann) = &mut pat.annotation
                                && ann.starts_with(&old_prefix)
                            {
                                let after_bracket = &ann[old_prefix.len()..];
                                if let Some(bracket_end) = after_bracket.find(']') {
                                    let digits = &after_bracket[..bracket_end];
                                    if !digits.is_empty()
                                        && digits.chars().all(|c| c.is_ascii_digit())
                                    {
                                        *ann = ann.replacen(&old_prefix, &new_prefix, 1);
                                    }
                                }
                            }
                        }
                    }
                }
                state.notify(format!("Group renamed to \"{label}\""));
                state.recompute_row_annotations();
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
                state.rebuild_pattern_lookup();
            }
        }
        HexEditorMessage::CyclePatternColor(pid) => {
            if let Some(pat) = state.patterns.iter_mut().find(|p| p.id == pid) {
                pat.color_idx = (pat.color_idx + 1) % 16;
                state.rebuild_pattern_lookup();
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
            let payload = export_patterns_to_json(state);
            if let Some(json) = payload {
                return Task::perform(
                    async move {
                        let path = rfd::AsyncFileDialog::new()
                            .set_title("Export Patterns")
                            .set_file_name("patterns.json")
                            .add_filter("Pattern files", &["json"])
                            .save_file()
                            .await;
                        let Some(path) = path else {
                            return HexEditorMessage::PatternsExported(
                                Err("cancelled".to_string()),
                            );
                        };
                        match tokio::fs::write(path.path(), json).await {
                            Ok(()) => HexEditorMessage::PatternsExported(Ok(())),
                            Err(e) => HexEditorMessage::PatternsExported(Err(e.to_string())),
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
                        .add_filter("Pattern files", &["json"])
                        .pick_file()
                        .await;
                    let Some(path) = path else {
                        return HexEditorMessage::PatternsImported(Err("cancelled".to_string()));
                    };
                    match tokio::fs::read_to_string(path.path()).await {
                        Ok(text) => HexEditorMessage::PatternsImported(Ok(text)),
                        Err(e) => HexEditorMessage::PatternsImported(Err(e.to_string())),
                    }
                },
                std::convert::identity,
            );
        }
        HexEditorMessage::PatternsExported(result) => match result {
            Ok(()) => {
                state.notify("Patterns exported");
            }
            Err(e) => {
                if e != "cancelled" {
                    state.notify(format!("Export failed: {e}"));
                }
            }
        },
        HexEditorMessage::PatternsImported(result) => match result {
            Ok(text) => match import_patterns_from_json(&text, state) {
                Ok(msg) => {
                    state.rebuild_pattern_lookup();
                    state.recompute_row_annotations();
                    state.notify(msg);
                }
                Err(e) => {
                    state.notify(format!("Import failed: {e}"));
                }
            },
            Err(e) => {
                if e != "cancelled" {
                    state.notify(format!("Import failed: {e}"));
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
        HexEditorMessage::SetTheme(variant) => {
            state.theme_variant = variant;
            state.theme = variant.theme();
        }
        HexEditorMessage::SetColorScheme(scheme) => {
            state.color_scheme = scheme;
        }
        HexEditorMessage::SetDimNulls(v) => {
            state.dim_nulls = v;
        }
        HexEditorMessage::SetShowEntropyBand(v) => {
            state.show_entropy_band = v;
        }
        HexEditorMessage::SetShowMinimapEnabled(v) => {
            state.show_minimap = v;
        }
        HexEditorMessage::SetAddrFormat(decimal) => {
            state.show_decimal = decimal;
        }
        HexEditorMessage::ResetSettings => {
            state.color_scheme = ColorScheme::Monochrome;
            state.dim_nulls = true;
            state.show_decimal = false;
            state.bytes_per_row = crate::state::DEFAULT_BYTES_PER_ROW;
            state.bpr_input = crate::state::DEFAULT_BYTES_PER_ROW.to_string();
            state.show_entropy_band = true;
            state.theme_variant = ThemeVariant::Dark;
            state.theme = ThemeVariant::Dark.theme();
            state.notify("Settings reset to defaults");
        }

        // ── Write mode / text encoding ───────────────────────────────────
        HexEditorMessage::SetWriteMode(mode) => {
            if mode != state.write_mode {
                state.edit_mode = None; // clear any in-progress hex edit
                state.write_mode = mode;
                if let Some(ref cb) = config.on_write_mode_changed {
                    return cb(mode);
                }
            }
        }
        HexEditorMessage::OpenEncodingSettings => {
            state.encoding_settings_open = true;
            state.encoding_settings_selection = Some(0);
        }
        HexEditorMessage::CloseEncodingSettings => {
            state.encoding_settings_open = false;
        }
        HexEditorMessage::AddCustomEncoding(common_idx) => {
            if let Some((label, enc_name)) =
                crate::domain::write_mode::COMMON_ENCODINGS.get(common_idx)
            {
                // Avoid duplicates.
                if !state
                    .custom_encodings
                    .iter()
                    .any(|e| e.encoding_name == *enc_name)
                {
                    state
                        .custom_encodings
                        .push(crate::domain::write_mode::EncodingEntry {
                            label: label.to_string(),
                            encoding_name: enc_name.to_string(),
                        });
                    state.notify(format!("Added encoding: {label}"));
                } else {
                    state.notify(format!("Encoding already added: {label}"));
                }
            }
        }
        HexEditorMessage::RemoveCustomEncoding(idx) => {
            if idx < state.custom_encodings.len() {
                let removed = state.custom_encodings.remove(idx);
                remap_write_mode(&mut state.write_mode, idx);
                state.notify(format!("Removed encoding: {}", removed.label));
            }
        }
        HexEditorMessage::SetCustomEncodings(encodings) => {
            state.custom_encodings = encodings;
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
            state.notify(format!("Copied {} byte(s) to clipboard", n));
            return clipboard::write(hex_str).map(|_| HexEditorMessage::ClipboardWriteResult);
        }

        HexEditorMessage::Paste => {
            if state.provider.is_empty() {
                return Task::none();
            }
            return clipboard::read_text().map(|contents| {
                let text = contents.unwrap_or_default();
                HexEditorMessage::PasteContent(text.to_string())
            });
        }

        HexEditorMessage::PasteContent(contents) => {
            if state.provider.is_empty() {
                return Task::none();
            }
            let bytes = if contents.is_empty() {
                state.notify("Clipboard is empty");
                return Task::none();
            } else {
                match parse_hex_query(&contents) {
                    Some(b) if !b.is_empty() => b,
                    _ => {
                        state.notify("Clipboard doesn't contain valid hex bytes");
                        return Task::none();
                    }
                }
            };
            let addr = state.selection.cursor;
            if addr >= state.provider.len() {
                state.notify("Cannot paste: cursor is past end of file");
                return Task::none();
            }
            state.provider.write(addr, &bytes);
            state.recompute_vanilla_diff();
            state.notify(format!("Pasted {} byte(s)", bytes.len()));
        }

        // ── Fill Selection ───────────────────────────────────────────────
        HexEditorMessage::BeginFill => {
            if state.provider.is_empty() || state.selection.is_single() {
                state.notify("Select a range of bytes to fill");
                return Task::none();
            }
            state.fill_dialog = Some(crate::domain::fill_dialog::FillDialog::new());
            return iced::widget::operation::focus(
                crate::domain::fill_dialog::FillDialog::input_id(),
            );
        }
        HexEditorMessage::SetFillDraft(s) => {
            if let Some(ref mut dlg) = state.fill_dialog {
                dlg.draft = s;
                dlg.error = None;
            }
        }
        HexEditorMessage::CommitFill => {
            let parse_result = state.fill_dialog.as_ref().map(|dlg| dlg.parse_pattern());
            match parse_result {
                Some(Ok(pattern)) => {
                    let start = state.selection.start();
                    let end = state.selection.end();
                    let range_len = end.saturating_sub(start).saturating_add(1);
                    let pattern_len = pattern.len() as u64;

                    if pattern_len > 0 {
                        // Repeat the pattern across the selected range.
                        let mut offset = 0u64;
                        while offset < range_len {
                            let chunk_end = (offset + pattern_len).min(range_len) as usize;
                            let chunk = &pattern[..chunk_end - offset as usize];
                            state.provider.write(start + offset, chunk);
                            offset += pattern_len;
                        }
                        state.recompute_vanilla_diff();
                        let written = range_len;
                        state.notify(format!("Filled {} byte(s) with {:02X?}", written, pattern));
                    } else {
                        state.notify("No bytes to fill — empty pattern");
                    }
                    state.fill_dialog = None;
                }
                Some(Err(msg)) => {
                    if let Some(ref mut dlg) = state.fill_dialog {
                        dlg.error = Some(msg);
                    }
                }
                None => {}
            }
        }
        HexEditorMessage::CloseFill => {
            state.fill_dialog = None;
        }

        // ── Extend File ─────────────────────────────────────────────────
        HexEditorMessage::BeginExtend => {
            if state.provider.is_empty() {
                state.notify("Cannot extend an empty file");
                return Task::none();
            }
            state.extend_dialog = Some(crate::domain::extend_dialog::ExtendDialog::new());
            return iced::widget::operation::focus(
                crate::domain::extend_dialog::ExtendDialog::count_input_id(),
            );
        }
        HexEditorMessage::SetExtendCount(s) => {
            if let Some(ref mut dlg) = state.extend_dialog {
                dlg.count_draft = s;
                dlg.error = None;
            }
        }
        HexEditorMessage::SetExtendPattern(s) => {
            if let Some(ref mut dlg) = state.extend_dialog {
                dlg.pattern_draft = s;
                dlg.error = None;
            }
        }
        HexEditorMessage::CommitExtend => {
            let parse_result = state.extend_dialog.as_ref().map(|dlg| dlg.parse());
            match parse_result {
                Some(Ok((count, pattern))) => {
                    // The dialog is modal, so context_menu_addr can't change
                    // between BeginExtend and CommitExtend — the right-clicked
                    // byte is the authoritative insert point.
                    let addr = state.context_menu_addr.unwrap_or(state.selection.cursor);
                    // A right-clicked byte past the editable buffer's last
                    // address (only reachable via the diff pane) can't be a
                    // valid insert point — reject instead of clamping to
                    // max_addr.
                    if state.context_menu_addr.is_some() && addr > state.max_addr() {
                        state.notify("Cannot extend: clicked past end of file");
                        return Task::none();
                    }
                    // Selection-driven path: `addr == len` is a valid append;
                    // anything past EOF is rejected.
                    if addr > state.provider.len() {
                        state.notify("Cannot extend: cursor is past end of file");
                        return Task::none();
                    }
                    // Repeat the pattern to exactly `count` bytes (count > 0
                    // and a non-empty pattern are guaranteed by parse).
                    let mut fill: Vec<u8> = Vec::with_capacity(count as usize);
                    let mut written = 0usize;
                    while written < count as usize {
                        let take = pattern.len().min(count as usize - written);
                        fill.extend_from_slice(&pattern[..take]);
                        written += take;
                    }

                    state.provider.insert(addr, &fill);
                    state.recompute_vanilla_diff();
                    // The comparison file's diff addresses shift with the
                    // insert — recompute so the diff pane stays accurate.
                    if let Some(cf) = state.comparison_file.as_mut() {
                        cf.diff =
                            crate::vanilla_diff::compute_diff(state.provider.as_slice(), &cf.data);
                    }
                    // Extend shifts every row boundary after the insert, so the
                    // per-row entropy band and cached stats are stale until the
                    // next analysis pass.
                    state.invalidate_stats();

                    // Pattern rebase: a span starting at/after the insert point
                    // shifts forward by `count`. A span straddling the insert
                    // point (start < addr <= end) keeps its start and absorbs
                    // the inserted bytes into its tail (end += count); a span
                    // fully before the insert point is untouched.
                    for p in &mut state.patterns {
                        if p.start >= addr {
                            p.start += count;
                            p.end += count;
                        } else if p.end >= addr {
                            p.end += count;
                        }
                    }

                    // Search results point at stale addresses after the
                    // insert — drop them (keep the overlay open/visible).
                    state.search.results.clear();
                    state.search.match_set.clear();
                    state.search.current_match = None;
                    state.search.query_len = 0;

                    // Select the inserted range BEFORE refreshing pattern
                    // lookups so refresh_active_patterns() sees the final
                    // cursor, not the pre-extend position.
                    state.selection.anchor = addr;
                    state.selection.cursor = addr + count - 1;

                    state.rebuild_pattern_lookup();
                    state.recompute_row_annotations();

                    // A pending in-matrix edit would now point at a shifted
                    // byte — cancel it so the next keystroke can't overwrite
                    // inserted data.
                    state.edit_mode = None;

                    state.notify(format!(
                        "Extended file by {} byte(s) with {:02X?}",
                        count, pattern
                    ));
                    state.extend_dialog = None;
                }
                Some(Err(msg)) => {
                    if let Some(ref mut dlg) = state.extend_dialog {
                        dlg.error = Some(msg);
                    }
                }
                None => {}
            }
        }
        HexEditorMessage::CloseExtend => {
            state.extend_dialog = None;
        }

        // ── Side-by-side diff view ───────────────────────────────────────
        HexEditorMessage::LoadComparisonFile => {
            return Task::perform(
                async move {
                    let path = rfd::AsyncFileDialog::new()
                        .set_title("Select Comparison File")
                        .pick_file()
                        .await;
                    match path {
                        Some(handle) => {
                            let name = handle
                                .path()
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("comparison")
                                .to_string();
                            match tokio::fs::read(handle.path()).await {
                                Ok(data) => {
                                    HexEditorMessage::ComparisonFileLoaded(Ok((data, name)))
                                }
                                Err(e) => HexEditorMessage::ComparisonFileLoaded(Err(format!(
                                    "Failed to read comparison file: {e}"
                                ))),
                            }
                        }
                        None => HexEditorMessage::ClearStatus,
                    }
                },
                std::convert::identity,
            );
        }
        HexEditorMessage::ComparisonFileLoaded(result) => match result {
            Ok((data, name)) => {
                let diff = crate::vanilla_diff::compute_diff(state.provider.as_slice(), &data);
                state.comparison_file = Some(ComparisonFile { name, data, diff });
                state.notify("Comparison file loaded");

                // Ensure the focused pane switches to Diff view.
                let focus = state.pane_focus;
                if let Some(panel) = state.panes.get_mut(focus) {
                    panel.content = crate::domain::panel::HexPanelContent::Diff;
                }
            }
            Err(e) => {
                state.notify(e);
            }
        },
        HexEditorMessage::CloseComparison => {
            state.comparison_file = None;
            state.diff_review = false;
            // Revert ALL panes that show Diff content, not just the focused one.
            for (_, panel) in state.panes.iter_mut() {
                if panel.content == crate::domain::panel::HexPanelContent::Diff {
                    panel.content = crate::domain::panel::HexPanelContent::Matrix;
                }
            }
            state.notify("Diff closed");
        }
        HexEditorMessage::DiffAddrSelected { addr, is_baseline } => {
            let max_addr = state.max_addr();
            let clamped = addr.min(max_addr);
            state.selection.select(clamped, max_addr);
            state.pending_center_on.set(Some(clamped));
            state.edit_mode = None;
            state.refresh_active_patterns();
            // The inspector follows the side that was clicked.
            state.inspector_source = if is_baseline {
                InspectorSource::Baseline
            } else {
                InspectorSource::Comparison
            };
        }
        HexEditorMessage::DiffExtendTo { addr, is_baseline } => {
            let max_addr = state.max_addr();
            state.selection.extend(addr.min(max_addr), max_addr);
            state.refresh_active_patterns();
            // The inspector follows the side the drag ended on.
            state.inspector_source = if is_baseline {
                InspectorSource::Baseline
            } else {
                InspectorSource::Comparison
            };
        }
        HexEditorMessage::DiffNavNext => {
            // Jump to next diff chunk — find the first address in `comparison_file.diff`
            // that is strictly greater than the cursor address.
            if let Some(ref cf) = state.comparison_file {
                let cursor = state.selection.cursor;
                if let Some(&addr) = cf.diff.range(cursor + 1..).next() {
                    state.selection.select(addr, state.max_addr());
                    state.pending_center_on.set(Some(addr));
                } else if let Some(&first) = cf.diff.first() {
                    // Wrap around.
                    state.selection.select(first, state.max_addr());
                    state.pending_center_on.set(Some(first));
                }
            }
        }
        HexEditorMessage::DiffNavPrev => {
            // Jump to previous diff chunk — find the last address in `comparison_file.diff`
            // that is strictly less than the cursor address.
            if let Some(ref cf) = state.comparison_file {
                let cursor = state.selection.cursor;
                if let Some(&addr) = cf.diff.range(..cursor).next_back() {
                    state.selection.select(addr, state.max_addr());
                    state.pending_center_on.set(Some(addr));
                } else if let Some(&last) = cf.diff.last() {
                    // Wrap around.
                    state.selection.select(last, state.max_addr());
                    state.pending_center_on.set(Some(last));
                }
            }
        }
        HexEditorMessage::ToggleDiffReview => {
            state.diff_review = !state.diff_review;
            state.notify(if state.diff_review {
                "Showing only diff rows".to_string()
            } else {
                "Showing all rows".to_string()
            });
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
                state.notify("Nothing to export — file is empty");
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
                state.notify("Exported as text file");
            }
            Err(e) => {
                if e != "cancelled" {
                    state.notify(format!("Export failed: {e}"));
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

// ── Pattern import / export (JSON) ─────────────────────────────────────

/// Serialise current patterns + groups to a JSON string.
pub(crate) fn export_patterns_to_json(state: &crate::HexEditorState) -> Option<String> {
    if state.patterns.is_empty() && state.groups.is_empty() {
        return None;
    }
    let export = crate::domain::pattern::PatternExport {
        version: crate::domain::pattern::PatternExport::VERSION,
        groups: state.groups.clone(),
        patterns: state.patterns.clone(),
    };
    serde_json::to_string_pretty(&export)
        .ok()
        .map(|json| format!("{}\n", json))
}

/// Parse a JSON pattern export and apply patterns/groups to `state`.
/// Returns a status message or an error.
pub(crate) fn import_patterns_from_json(
    json: &str,
    state: &mut crate::HexEditorState,
) -> Result<String, String> {
    let export: crate::domain::pattern::PatternExport =
        serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {e}"))?;

    if export.patterns.is_empty() {
        return Err("No patterns found in import file".to_string());
    }

    // Map exported group IDs to new local IDs.
    use std::collections::HashMap;
    let mut group_map: HashMap<usize, usize> = HashMap::new();

    for g in &export.groups {
        let new_id = state.next_group_id;
        state.next_group_id += 1;
        group_map.insert(g.id, new_id);
        state.groups.push(RepeatedPatternGroup::new(
            new_id,
            g.label.clone(),
            g.color_idx,
        ));
    }

    for p in &export.patterns {
        let id = state.next_pattern_id;
        state.next_pattern_id += 1;
        let mut pat = Pattern::new(id, p.start.min(p.end), p.start.max(p.end), p.color_idx % 16);
        pat.group_id = p.group_id.and_then(|gid| group_map.get(&gid).copied());
        pat.annotation.clone_from(&p.annotation);
        state.patterns.push(pat);
    }

    let count = export.patterns.len();
    Ok(format!("Imported {count} pattern(s)"))
}

#[cfg(test)]
mod tests {
    use super::*;
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

    // ── parse_bpr ─────────────────────────────────────────────────────────

    #[test]
    fn parse_bpr_accepts_valid_values() {
        assert_eq!(parse_bpr("9"), Some(9));
        assert_eq!(parse_bpr("20"), Some(20));
    }

    #[test]
    fn parse_bpr_accepts_min_boundary() {
        assert_eq!(parse_bpr("1"), Some(1));
    }

    #[test]
    fn parse_bpr_accepts_max_boundary() {
        assert_eq!(parse_bpr("64"), Some(64));
    }

    #[test]
    fn parse_bpr_rejects_below_min() {
        assert_eq!(parse_bpr("0"), None);
    }

    #[test]
    fn parse_bpr_rejects_above_max() {
        assert_eq!(parse_bpr("65"), None);
    }

    #[test]
    fn parse_bpr_rejects_non_numeric() {
        assert_eq!(parse_bpr("abc"), None);
    }

    #[test]
    fn parse_bpr_rejects_empty() {
        assert_eq!(parse_bpr(""), None);
    }

    #[test]
    fn parse_bpr_trims_whitespace() {
        assert_eq!(parse_bpr(" 20 "), Some(20));
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
            assert_eq!(
                &line[58..60],
                "  ",
                "row {i}: hex/ASCII separator not at expected position 58"
            );
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
        let result = format_hex_dump(
            bytes,
            16,
            &ExportConfig {
                show_address: false,
                address_decimal: false,
                show_ascii: false,
            },
        );
        // Just the hex values, nothing else
        assert_eq!(result, "00 01 02 03                                     \n");
    }

    // ====================================================================
    // JSON-based pattern export / import
    // ====================================================================

    // ====================================================================
    // JSON-based pattern export / import
    // ====================================================================

    /// Pattern spec: (start, end, color, group_id, annotation).
    type PatternSpec<'a> = (u64, u64, u8, Option<usize>, Option<&'a str>);

    /// Build a minimal HexEditorState with the given patterns and groups.
    fn make_export_state(
        patterns: Vec<PatternSpec<'_>>,
        groups: Vec<(usize, &str, u8)>,
    ) -> crate::HexEditorState {
        let mut next_pid = 1usize;
        let mut state = crate::HexEditorState::load_from_path(std::path::Path::new("test.bin"));
        // Reset counters
        state.next_pattern_id = 1;
        state.next_group_id = 1;
        state.patterns.clear();
        state.groups.clear();
        for (start, end, color, gid, ann) in patterns {
            let id = next_pid;
            next_pid += 1;
            let mut pat = crate::Pattern::new(id, start, end, color);
            pat.group_id = gid;
            pat.annotation = ann.map(|s| s.to_string());
            state.patterns.push(pat);
        }
        state.next_pattern_id = next_pid;
        for (id, label, color) in groups {
            state.groups.push(crate::RepeatedPatternGroup::new(
                id,
                label.to_string(),
                color,
            ));
            if id >= state.next_group_id {
                state.next_group_id = id + 1;
            }
        }
        state.rebuild_pattern_lookup();
        state.recompute_row_annotations();
        state
    }

    #[test]
    fn export_empty_returns_none() {
        let state = make_export_state(vec![], vec![]);
        assert!(export_patterns_to_json(&state).is_none());
    }

    #[test]
    fn export_only_groups_contains_group_data() {
        let state = make_export_state(vec![], vec![(1, "GroupA", 3)]);
        let json = export_patterns_to_json(&state).unwrap();
        let parsed: crate::domain::pattern::PatternExport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.groups.len(), 1);
        assert_eq!(parsed.groups[0].label, "GroupA");
        assert_eq!(parsed.groups[0].color_idx, 3);
    }

    #[test]
    fn export_only_patterns_contains_pattern_data() {
        let state = make_export_state(vec![(0x100, 0x1FF, 5, None, None)], vec![]);
        let json = export_patterns_to_json(&state).unwrap();
        let parsed: crate::domain::pattern::PatternExport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.patterns.len(), 1);
        assert_eq!(parsed.patterns[0].start, 0x100);
        assert_eq!(parsed.patterns[0].end, 0x1FF);
        assert_eq!(parsed.patterns[0].color_idx, 5);
    }

    #[test]
    fn import_empty_json_returns_error() {
        let mut state = crate::HexEditorState::load_from_path(std::path::Path::new("test.bin"));
        let result = import_patterns_from_json("", &mut state);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn export_import_basic_roundtrip() {
        let state = make_export_state(
            vec![
                (0x000, 0x0FF, 3, None, Some("header")),
                (0x100, 0x1FF, 5, Some(1), Some("body[0]")),
                (0x200, 0x2FF, 7, Some(1), Some("body[1]")),
            ],
            vec![(1, "BodyGroup", 2)],
        );
        let json = export_patterns_to_json(&state).unwrap();

        let mut import_state =
            crate::HexEditorState::load_from_path(std::path::Path::new("test.bin"));
        let msg = import_patterns_from_json(&json, &mut import_state).unwrap();
        assert!(msg.contains("Imported 3 pattern(s)"));

        assert_eq!(import_state.patterns.len(), 3);
        assert_eq!(import_state.groups.len(), 1);

        // Patterns map to the new group
        let mapped_group = import_state.groups[0].id;
        let grouped: Vec<_> = import_state
            .patterns
            .iter()
            .filter(|p| p.group_id == Some(mapped_group))
            .collect();
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].annotation.as_deref(), Some("body[0]"));
        assert_eq!(grouped[1].annotation.as_deref(), Some("body[1]"));

        // Ungrouped pattern preserved
        let ungrouped: Vec<_> = import_state
            .patterns
            .iter()
            .filter(|p| p.group_id.is_none())
            .collect();
        assert_eq!(ungrouped.len(), 1);
        assert_eq!(ungrouped[0].annotation.as_deref(), Some("header"));
    }

    #[test]
    fn export_import_special_chars_in_annotation() {
        // JSON natively handles pipes, backslashes, newlines, etc.
        let state = make_export_state(
            vec![
                (0x000, 0x0FF, 3, None, Some("a|b|c")),
                (0x100, 0x1FF, 5, None, Some("path\\to\\file")),
                (0x200, 0x2FF, 7, None, Some("line1\nline2")),
            ],
            vec![],
        );
        let json = export_patterns_to_json(&state).unwrap();

        let mut import_state =
            crate::HexEditorState::load_from_path(std::path::Path::new("test.bin"));
        import_patterns_from_json(&json, &mut import_state).unwrap();
        assert_eq!(import_state.patterns.len(), 3);

        let ann0 = import_state
            .patterns
            .iter()
            .find(|p| p.start == 0x000)
            .unwrap()
            .annotation
            .as_deref();
        assert_eq!(ann0, Some("a|b|c"));

        let ann1 = import_state
            .patterns
            .iter()
            .find(|p| p.start == 0x100)
            .unwrap()
            .annotation
            .as_deref();
        assert_eq!(ann1, Some("path\\to\\file"));

        let ann2 = import_state
            .patterns
            .iter()
            .find(|p| p.start == 0x200)
            .unwrap()
            .annotation
            .as_deref();
        assert_eq!(ann2, Some("line1\nline2"));
    }

    #[test]
    fn export_import_special_chars_in_group_label() {
        let state = make_export_state(
            vec![(0x100, 0x1FF, 5, Some(1), Some("data"))],
            vec![(1, "Group|One\nLabel\\Test", 4)],
        );
        let json = export_patterns_to_json(&state).unwrap();

        let mut import_state =
            crate::HexEditorState::load_from_path(std::path::Path::new("test.bin"));
        import_patterns_from_json(&json, &mut import_state).unwrap();
        assert_eq!(import_state.groups.len(), 1);
        assert_eq!(import_state.groups[0].label, "Group|One\nLabel\\Test");
    }

    #[test]
    fn import_invalid_json_returns_error() {
        let mut state = crate::HexEditorState::load_from_path(std::path::Path::new("test.bin"));
        let result = import_patterns_from_json("{invalid json}", &mut state);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn import_json_no_patterns_returns_error() {
        let json = r#"{"version":1,"groups":[],"patterns":[]}"#;
        let mut state = crate::HexEditorState::load_from_path(std::path::Path::new("test.bin"));
        let result = import_patterns_from_json(json, &mut state);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No patterns found"));
    }

    #[test]
    fn export_import_large_address_space() {
        let state = make_export_state(vec![(0x0000_0000, 0x00FF_FFFF, 1, None, None)], vec![]);
        let json = export_patterns_to_json(&state).unwrap();
        let parsed: crate::domain::pattern::PatternExport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.patterns[0].start, 0);
        assert_eq!(parsed.patterns[0].end, 0x00FF_FFFF);

        let mut import_state =
            crate::HexEditorState::load_from_path(std::path::Path::new("test.bin"));
        import_patterns_from_json(&json, &mut import_state).unwrap();
        assert_eq!(import_state.patterns.len(), 1);
        assert_eq!(import_state.patterns[0].start, 0);
        assert_eq!(import_state.patterns[0].end, 0x00FF_FFFF);
    }

    #[test]
    fn export_import_empty_groups_vec() {
        // Groups array can be empty — that's fine.
        let state = make_export_state(vec![(0x000, 0x0FF, 3, None, Some("solo"))], vec![]);
        let json = export_patterns_to_json(&state).unwrap();
        let parsed: crate::domain::pattern::PatternExport = serde_json::from_str(&json).unwrap();
        assert!(parsed.groups.is_empty());
    }

    #[test]
    fn export_json_is_valid_pretty() {
        let state = make_export_state(vec![(0x000, 0x0FF, 3, None, None)], vec![]);
        let json = export_patterns_to_json(&state).unwrap();
        // Pretty-printed JSON should contain newlines
        assert!(json.contains('\n'));
        // Should end with a trailing newline
        assert!(json.ends_with('\n'));
    }
}
