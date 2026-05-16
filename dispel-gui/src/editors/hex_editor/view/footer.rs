use iced::widget::{container, text};
use iced::{Element, Fill, Font};

use crate::editors::hex_editor::selection::Selection;
use crate::editors::hex_editor::HexEditorState;
use crate::editors::hex_editor::HexProvider;
use crate::message::Message;

pub fn view(editor: &HexEditorState) -> Element<'_, Message> {
    container(text(format_footer(editor)).size(11).font(Font::MONOSPACE))
        .padding([4, 12])
        .width(Fill)
        .into()
}

/// Pure formatter — easy to assert in unit tests.
pub fn format_footer(editor: &HexEditorState) -> String {
    let total = editor.provider.len();
    let dirty = editor.provider.dirty_count();
    let cursor = editor.selection.cursor;
    let total_str = humanize_size(total);
    if editor.provider.is_empty() {
        return format!("(empty)  ·  total: 0 (0 B)  ·  dirty: {dirty}");
    }
    let sel = editor.selection;
    let sel_str = format_selection(sel, editor.show_decimal);
    let total_fmt = if editor.show_decimal {
        format!("{total}")
    } else {
        format!("0x{total:X}")
    };
    let cursor_fmt = if editor.show_decimal {
        format!("{cursor}")
    } else {
        format!("0x{cursor:X}")
    };
    format!(
        "{sel}  ·  total: {total_fmt} ({total_str})  ·  dirty: {dirty}  ·  cursor: {cursor_fmt}",
        sel = sel_str,
    )
}

pub fn format_selection(sel: Selection, show_decimal: bool) -> String {
    let lo = sel.start();
    let hi = sel.end();
    let len = sel.len();
    if show_decimal {
        if sel.is_single() {
            format!("{lo}")
        } else {
            format!("{lo} - {hi} ({len} B)")
        }
    } else if sel.is_single() {
        format!("0x{lo:X}")
    } else {
        format!("0x{lo:X} - 0x{hi:X} (0x{len:X} / {len} B)")
    }
}

fn humanize_size(n: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    if n >= GB {
        format!("{:.2} GB", n as f64 / GB as f64)
    } else if n >= MB {
        format!("{:.2} MB", n as f64 / MB as f64)
    } else if n >= KB {
        format!("{:.2} KB", n as f64 / KB as f64)
    } else {
        format!("{n} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humanize_handles_each_unit() {
        assert_eq!(humanize_size(512), "512 B");
        assert_eq!(humanize_size(2048), "2.00 KB");
        assert_eq!(humanize_size(2 * 1024 * 1024), "2.00 MB");
    }

    #[test]
    fn format_selection_single_byte() {
        assert_eq!(format_selection(Selection::single(0x10), false), "0x10");
    }

    #[test]
    fn format_selection_range_shows_size() {
        let sel = Selection {
            anchor: 0x10,
            cursor: 0x1F,
        };
        let s = format_selection(sel, false);
        assert!(s.contains("0x10 - 0x1F"));
        assert!(s.contains("(0x10 / 16 B)"));
    }

    #[test]
    fn format_selection_decimal() {
        assert_eq!(format_selection(Selection::single(0x10), true), "16");
        let sel = Selection {
            anchor: 0x10,
            cursor: 0x1F,
        };
        let s = format_selection(sel, true);
        assert!(s.contains("16 - 31"));
        assert!(s.contains("(16 B)"));
    }
}
