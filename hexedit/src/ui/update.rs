use iced::{clipboard, Task};

use crate::config::HexEditorConfig;
use crate::editing::{EditState, InspectorEditState};
use crate::goto::GotoState;
use crate::inspector::ENTRIES;
use crate::message::HexEditorMessage;
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
            state.context_menu_addr = None;
        }

        // ── Address format ──────────────────────────────────────────────
        HexEditorMessage::ToggleAddrFormat => {
            state.show_decimal = !state.show_decimal;
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
        HexEditorMessage::ExportAsText => {
            let bytes = state.provider.as_slice().to_vec();
            if bytes.is_empty() {
                state.status_msg = "Nothing to export — file is empty".to_string();
                return Task::none();
            }
            let bpr = state.bytes_per_row;
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
                    let text = format_hex_dump(&bytes, bpr);
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
///
/// # Example (with `bytes_per_row = 16`)
///
/// ```text
/// 00000000  48 65 6C 6C 6F 20 57 6F  72 6C 64 00 00 00 00 00  Hello World·····
/// 00000010  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F  ················
/// ```
pub fn format_hex_dump(bytes: &[u8], bytes_per_row: u8) -> String {
    let bpr = bytes_per_row.max(1) as usize;

    // Compute the fixed width of the hex column for a full row.
    // Each byte contributes: `XX` (2 chars) + a trailing separator, except:
    //   - the first byte has no leading separator,
    //   - the last byte has no trailing separator,
    //   - after every 8th byte an extra space is added (group gap).
    // Formula: bpr * 3 - 1 + group_gaps
    // where group_gaps = (bpr / 8).saturating_sub(1)  (n groups → n-1 gaps)
    let groups = bpr.div_ceil(8);
    let hex_col_width = bpr * 3 - 1 + groups.saturating_sub(1);

    let mut output = String::with_capacity(
        // Estimate: 8 addr + 2 sep + hex_col_width + 2 sep + bpr ascii + 1 newline, per row
        ((8 + 2 + hex_col_width + 2 + bpr + 1) * bytes.len().div_ceil(bpr)).min(usize::MAX),
    );

    for (chunk_idx, chunk) in bytes.chunks(bpr).enumerate() {
        let mut hex_part = String::with_capacity(hex_col_width);

        for (col, &b) in chunk.iter().enumerate() {
            if col > 0 {
                if col % 8 == 0 {
                    hex_part.push_str("  "); // double space between 8-byte groups
                } else {
                    hex_part.push(' ');
                }
            }
            hex_part.push_str(&format!("{:02X}", b));
        }

        // Build ASCII column
        let mut ascii_part = String::with_capacity(bpr);
        for &b in chunk {
            if (0x20..0x7F).contains(&b) {
                ascii_part.push(b as char);
            } else {
                ascii_part.push('·'); // middle dot, matching matrix rendering
            }
        }

        // Left-align hex column, pad to full width so ASCII aligns
        output.push_str(&format!(
            "{0:08X}  {1:<2$}  {3}\n",
            chunk_idx * bpr,
            hex_part,
            hex_col_width,
            ascii_part,
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::format_hex_dump;

    #[test]
    fn format_empty_bytes() {
        assert_eq!(format_hex_dump(&[], 16), "");
    }

    #[test]
    fn format_single_row() {
        let bytes = b"Hello World";
        let result = format_hex_dump(bytes, 16);
        // 11 bytes in a 16-byte row — hex column is right-padded to fixed width
        assert!(result.starts_with("00000000  48 65 6C 6C 6F 20 57 6F  72 6C 64"));
        assert!(result.contains("Hello World"));
        assert!(result.ends_with("\n"));
    }

    #[test]
    fn format_two_rows() {
        let bytes: Vec<u8> = (0..32).collect();
        let result = format_hex_dump(&bytes, 16);
        let expected = "\
00000000  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F  ················
00000010  10 11 12 13 14 15 16 17  18 19 1A 1B 1C 1D 1E 1F  ················
";
        assert_eq!(result, expected);
    }

    #[test]
    fn format_partial_last_row() {
        let bytes: Vec<u8> = (0..20).collect();
        let result = format_hex_dump(&bytes, 16);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        // Full first row
        assert!(lines[0].starts_with("00000000"));
        // Second row has only 4 bytes (address 0x10)
        assert!(lines[1].starts_with("00000010"));
        assert!(lines[1].contains("10 11 12 13")); // bytes 16-19
    }

    #[test]
    fn format_bpr_8() {
        let bytes: Vec<u8> = (0..8).collect();
        let result = format_hex_dump(&bytes, 8);
        // With 8 BPR there should be 8 bytes, no group gap needed (only 1 group)
        assert!(result.contains("00 01 02 03 04 05 06 07"));
    }

    #[test]
    fn format_bpr_32() {
        let bytes: Vec<u8> = (0..64).collect();
        let result = format_hex_dump(&bytes, 32);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        // First line should have address 0x00000000, second 0x00000020 (32 byte rows)
        assert!(lines[0].starts_with("00000000"));
        assert!(lines[1].starts_with("00000020"));
    }

    #[test]
    fn format_non_printable_shows_dot() {
        let bytes = [0x00, 0x1F, 0x7F, 0x80, 0xFF];
        let result = format_hex_dump(&bytes, 16);
        // These bytes should all show as '·' in the ASCII column
        let ascii_part = result.split("  ").last().unwrap_or("");
        assert_eq!(ascii_part.trim_end_matches('\n'), "·····");
    }

    #[test]
    fn format_printable_preserved() {
        let bytes = b"ABC123!@#";
        let result = format_hex_dump(bytes, 16);
        let ascii_part = result.split("  ").last().unwrap_or("");
        assert!(ascii_part.contains("ABC123!@#"));
    }

    #[test]
    fn format_hex_colums_align() {
        // Hex column is padded to a fixed width so ASCII always starts at
        // the same column position regardless of row length.
        // For BPR=16: hex_col_width = 48, address + spaces = 10,
        // so the hex/ASCII separator "  " should always be at position 58.
        let bytes: Vec<u8> = (0..40).collect();
        let result = format_hex_dump(&bytes, 16);
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
}
