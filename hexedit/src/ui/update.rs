use iced::{clipboard, Task};

use crate::config::HexEditorConfig;
use crate::domain::export_config::ExportConfig;
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
        ((addr_width + hex_col_width + 2 + bpr + 1) * bytes.len().div_ceil(bpr)).min(usize::MAX),
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
