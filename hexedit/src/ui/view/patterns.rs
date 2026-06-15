//! Pattern list panel with Git-log-style vertical timeline.
//!
//! All patterns are sorted by their start address (with creation-order tiebreak).
//! Contiguous runs of patterns in the same group get branch connectors in the
//! left gutter (`├`, `│`, `└`) — exactly like Git branches.
//!
//! Ungrouped patterns show as `●`.
//!
//! ```text
//! Monster ─────────────────────────────────────────►
//! ├─ 0x10‑0x2F  32 B  [header]  [annotation]  [✕]
//! ├─ 0x30‑0x4F  32 B  [body]    [annotation]  [✕]
//! └─ 0x50‑0x6F  32 B  [footer]  [annotation]  [✕]
//!
//! ●─ 0x80‑0x8F  16 B  [checksum]  [annotation]  [✕]
//! ```

use iced::widget::space::Space;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{color, Element, Fill, Font, Length};

use crate::domain::pattern_layout::{compute_pattern_rows, GutterGlyph, PatternRow};
use crate::pattern::{pattern_bg, pattern_fg};
use crate::state::HexEditorState;
use crate::HexEditorMessage;

// ── Public entry point ──────────────────────────────────────────────────────

pub fn view(editor: &HexEditorState) -> Element<'_, HexEditorMessage> {
    let count = editor.patterns.len();
    let header = row![
        text(format!("Patterns ({})", count))
            .size(11)
            .font(Font::MONOSPACE),
        Space::default().width(Fill),
        // Export / Import buttons
        if !editor.patterns.is_empty() {
            text("↥").size(12).font(Font::MONOSPACE)
        } else {
            text("").size(12)
        },
        if !editor.patterns.is_empty() {
            text("↧").size(12).font(Font::MONOSPACE)
        } else {
            text("").size(12)
        },
        button(text("✕").size(10).font(Font::MONOSPACE))
            .padding([2, 6])
            .on_press(HexEditorMessage::TogglePatternList),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let header = container(header).padding([4, 12]).width(Fill);

    if count == 0 {
        return container(column![
            header,
            text("No patterns defined").size(11).font(Font::MONOSPACE),
        ])
        .width(Fill)
        .into();
    }

    // ── Compute sorted visible rows ─────────────────────────────────────
    let rows = compute_pattern_rows(&editor.patterns, &editor.groups, &editor.collapsed_groups);

    let mut col = column![].spacing(2).padding([2, 12]);

    // ── Toolbar row ─────────────────────────────────────────────────────
    let import_export_row = row![
        Space::default().width(Fill),
        button(text("Export").size(9).font(Font::MONOSPACE))
            .padding([2, 8])
            .on_press(HexEditorMessage::ExportPatterns),
        button(text("Import").size(9).font(Font::MONOSPACE))
            .padding([2, 8])
            .on_press(HexEditorMessage::ImportPatterns),
    ]
    .spacing(6);
    col = col.push(import_export_row);

    // ── Render rows ─────────────────────────────────────────────────────
    for row_info in &rows {
        // Insert group header when we encounter a new group.
        // Group headers repeat on every run of the same group (interleaving).
        if row_info.group_label.is_some() {
            col = col.push(group_header_row(
                row_info.group_id.expect("group_label ⇒ Some(group_id)"),
                editor,
            ));
        }

        // For collapsed groups only the header is rendered — no pattern row.
        if !row_info.collapsed {
            col = col.push(pattern_row(row_info, editor));
        }
    }

    let body: Element<'_, HexEditorMessage> = scrollable(col).height(Length::Shrink).into();

    container(column![header, body])
        .width(Fill)
        .style(|_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(color!(0x1e1e1e))),
            border: iced::Border {
                color: color!(0x3d3d3d),
                width: 1.0,
                radius: 0.into(),
            },
            ..Default::default()
        })
        .into()
}

// ── Group header row ────────────────────────────────────────────────────────

/// Renders a group section header: toggle + swatch + label + count + remove.
/// The `label` parameter is unused (the group's actual label is fetched from
/// `editor.groups`) — it only serves as a signal that a group starts here.
fn group_header_row<'a>(gid: usize, editor: &'a HexEditorState) -> Element<'a, HexEditorMessage> {
    let group = editor.groups.iter().find(|g| g.id == gid);
    let grp = match group {
        Some(g) => g,
        None => return text("").into(),
    };

    let (bg, fg) = (pattern_bg(grp.color_idx), pattern_fg(grp.color_idx));

    let swatch = container(text("  ").size(8))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(12.0))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                color: fg,
                width: 1.0,
                radius: 2.into(),
            },
            ..Default::default()
        });
    let swatch_btn = button(swatch)
        .padding(0)
        .on_press(HexEditorMessage::CycleGroupColor(grp.id))
        .style(button::text);

    let collapsed = editor.collapsed_groups.contains(&grp.id);
    let toggle_char = if collapsed { "▶" } else { "▼" };
    let toggle = text(toggle_char).size(9).font(Font::MONOSPACE);

    // ── Rename UI ────────────────────────────────────────────────────────
    let label_elem: Element<'a, HexEditorMessage> = if editor.renaming_group == Some(grp.id) {
        text_input("Group name", &editor.renaming_group_draft)
            .id(iced::widget::Id::from(format!(
                "hex-rename-group-input-{}",
                grp.id
            )))
            .on_input(HexEditorMessage::SetRenameGroupDraft)
            .on_submit(HexEditorMessage::CommitRenameGroup)
            .size(10)
            .padding([1, 4])
            .width(Length::Fixed(140.0))
            .into()
    } else {
        button(text(&grp.label).size(10).font(Font::MONOSPACE))
            .on_press(HexEditorMessage::BeginRenameGroup(grp.id))
            .padding(0)
            .style(button::text)
            .into()
    };

    // Count patterns belonging to this group (including collapsed)
    let pattern_count = editor
        .patterns
        .iter()
        .filter(|p| p.group_id == Some(gid))
        .count();
    let count_text = text(format!("({})", pattern_count))
        .size(10)
        .color(color!(0x8a7a6a))
        .font(Font::MONOSPACE);

    let remove_btn = button(text("✕").size(9).font(Font::MONOSPACE))
        .padding([1, 4])
        .on_press(HexEditorMessage::RemovePatternGroup(grp.id));

    // ── Assemble ─────────────────────────────────────────────────────────
    let inner = row![
        toggle,
        swatch_btn,
        label_elem,
        count_text,
        Space::default().width(Fill),
        remove_btn,
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    // Clicking the header toggles collapse/expand
    button(inner)
        .on_press(HexEditorMessage::TogglePatternGroup(grp.id))
        .padding([4, 6])
        .width(Fill)
        .style(button::text)
        .into()
}

// ── Pattern row ─────────────────────────────────────────────────────────────

/// Renders a single pattern row with gutter glyph, swatch, addresses, size,
/// annotation text input, and remove button.
///
/// The lifetime is deliberately decoupled from `row_info` — the returned
/// `Element` only borrows from `editor` and owned data, so the caller can
/// construct it from a local `PatternRow` without borrow conflicts.
fn pattern_row<'a>(
    row_info: &PatternRow,
    editor: &'a HexEditorState,
) -> Element<'a, HexEditorMessage> {
    let pattern = match editor.pattern_by_id(row_info.pattern_id) {
        Some(p) => p,
        None => return text("").into(),
    };

    let (bg, fg) = (pattern_bg(pattern.color_idx), pattern_fg(pattern.color_idx));

    // ── Gutter glyph ─────────────────────────────────────────────────────
    let glyph_char = match row_info.glyph {
        GutterGlyph::GroupFirst => "├",
        GutterGlyph::GroupMiddle => "│",
        GutterGlyph::GroupLast => "└",
        GutterGlyph::Solo => "●",
    };
    let glyph = container(text(glyph_char).size(10).font(Font::MONOSPACE))
        .width(Length::Fixed(18.0))
        .align_x(iced::Alignment::Center);

    // ── Color swatch (click to cycle) ────────────────────────────────────
    let swatch = container(text("  ").size(8))
        .width(Length::Fixed(14.0))
        .height(Length::Fixed(12.0))
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                color: fg,
                width: 1.0,
                radius: 2.into(),
            },
            ..Default::default()
        });
    let swatch_btn = button(swatch)
        .padding(0)
        .on_press(HexEditorMessage::CyclePatternColor(pattern.id))
        .style(button::text);

    // ── Address range ────────────────────────────────────────────────────
    let start = text(format!("0x{:08X}", pattern.start))
        .size(10)
        .font(Font::MONOSPACE);
    let end = text(format!("0x{:08X}", pattern.end))
        .size(10)
        .font(Font::MONOSPACE);
    let range = row![
        container(start).width(Length::Fixed(80.0)),
        text("─").size(10).font(Font::MONOSPACE),
        container(end).width(Length::Fixed(80.0)),
    ]
    .spacing(2)
    .align_y(iced::Alignment::Center);

    // ── Size ─────────────────────────────────────────────────────────────
    let size_str = format_size(pattern.len());
    let size = container(text(size_str).size(10).font(Font::MONOSPACE)).width(Length::Fixed(50.0));

    // ── Remove button ────────────────────────────────────────────────────
    let remove_btn = button(text("✕").size(9).font(Font::MONOSPACE))
        .padding([1, 4])
        .on_press(HexEditorMessage::RemovePattern(pattern.id));

    // ── Annotation text input ────────────────────────────────────────────
    let current = pattern.annotation.as_deref().unwrap_or("");
    let ann_input = text_input("Annotation…", current)
        .on_input(move |s| HexEditorMessage::SetPatternAnnotation(pattern.id, s))
        .size(9)
        .padding([1, 4])
        .width(Fill);

    // ── Navigation button (range + size, click to navigate) ──────────────
    let nav_btn = button(
        row![range, size]
            .spacing(6)
            .align_y(iced::Alignment::Center),
    )
    .on_press(HexEditorMessage::NavigateToPattern(pattern.id))
    .padding([3, 6])
    .style(button::text);

    // ── Assemble metadata row (sibling buttons, no nesting) ──────────────
    let metadata = row![swatch_btn, nav_btn, remove_btn,]
        .spacing(6)
        .align_y(iced::Alignment::Center);

    // ── Assemble row ─────────────────────────────────────────────────────
    let row = row![glyph, metadata, ann_input]
        .spacing(6)
        .align_y(iced::Alignment::Center);

    let is_active = editor.active_patterns.contains(&pattern.id);
    container(row)
        .padding(iced::Padding {
            top: 0.0,
            right: 6.0,
            bottom: 0.0,
            left: 6.0,
        })
        .width(Fill)
        .style(move |_: &iced::Theme| {
            if is_active {
                container::Style {
                    background: Some(iced::Background::Color(color!(0x3b2a18))),
                    ..Default::default()
                }
            } else {
                container::Style::default()
            }
        })
        .into()
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Format a byte count into a human-readable short string.
fn format_size(len: u64) -> String {
    if len >= 1024 * 1024 {
        format!("{:.1} MB", len as f64 / (1024.0 * 1024.0))
    } else if len >= 1024 {
        format!("{} KB", len / 1024)
    } else {
        format!("{} B", len)
    }
}
