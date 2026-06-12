//! Pattern list panel with accordion grouping, colour cycling, inline rename,
//! annotation editing, and pattern import/export.
//!
//! Ungrouped patterns (created via "Create Pattern") appear as flat rows.
//! Repeated-pattern groups (created via "Add Repeated Pattern") appear under
//! a collapsible accordion header labelled with the group name.

use std::collections::BTreeMap;

use iced::widget::space::Space;
use iced::widget::{
    button, column, container, row, scrollable, text, text_input,
};
use iced::{color, Element, Fill, Font, Length};

use crate::pattern::{pattern_bg, pattern_fg};
use crate::state::HexEditorState;
use crate::{HexEditorMessage, Pattern, RepeatedPatternGroup};

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

    // ── Group patterns by group_id ───────────────────────────────────────
    let mut ungrouped: Vec<&Pattern> = Vec::new();
    let mut grouped: BTreeMap<usize, Vec<&Pattern>> = BTreeMap::new();
    for pat in &editor.patterns {
        match pat.group_id {
            Some(gid) => grouped.entry(gid).or_default().push(pat),
            None => ungrouped.push(pat),
        }
    }

    let mut col = column![].spacing(1).padding([2, 12]);

    // ── Toolbar row ──────────────────────────────────────────────────────
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

    // ── Render ungrouped patterns ────────────────────────────────────────
    for pat in &ungrouped {
        col = col.push(pattern_row(pat));
    }

    // ── Render grouped patterns under accordion headers ───────────────────
    for gid in grouped.keys() {
        let group = editor.groups.iter().find(|g| g.id == *gid);
        if let Some(grp) = group {
            let children = &grouped[gid];
            let collapsed = editor.collapsed_groups.contains(&grp.id);
            col = col.push(group_header(
                editor,
                grp,
                children.len(),
                collapsed,
            ));
            if !collapsed {
                for pat in children {
                    col = col.push(pattern_row(pat));
                }
            }
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

/// Accordion header row for a repeated-pattern group.
fn group_header<'a>(
    editor: &'a HexEditorState,
    grp: &'a RepeatedPatternGroup,
    child_count: usize,
    collapsed: bool,
) -> Element<'a, HexEditorMessage> {
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
    // Clicking the swatch cycles the group colour.
    let swatch_btn = button(swatch)
        .padding(0)
        .on_press(HexEditorMessage::CycleGroupColor(grp.id))
        .style(button::text);

    let toggle_char = if collapsed { "▶" } else { "▼" };
    let toggle = text(toggle_char).size(9).font(Font::MONOSPACE);

    // ── Rename UI ────────────────────────────────────────────────────────
    let label_elem: Element<'_, HexEditorMessage> =
        if editor.renaming_group == Some(grp.id) {
            text_input("Group name", &editor.renaming_group_draft)
                .id(iced::widget::Id::new("hex-rename-group-input"))
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

    let count = text(format!("({})", child_count))
        .size(10)
        .color(color!(0x8a7a6a))
        .font(Font::MONOSPACE);

    let remove_btn = button(text("✕").size(9).font(Font::MONOSPACE))
        .padding([1, 4])
        .on_press(HexEditorMessage::RemovePatternGroup(grp.id));

    let inner = row![
        toggle,
        swatch_btn,
        label_elem,
        count,
        Space::default().width(Fill),
        remove_btn,
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    button(inner)
        .on_press(HexEditorMessage::TogglePatternGroup(grp.id))
        .padding([4, 6])
        .width(Fill)
        .style(button::text)
        .into()
}

/// A single pattern row — clickable to navigate, swatch cycles colour,
/// annotation text input sits on the right.
fn pattern_row<'a>(pat: &'a Pattern) -> Element<'a, HexEditorMessage> {
    let (bg, fg) = (pattern_bg(pat.color_idx), pattern_fg(pat.color_idx));

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
    // Clicking the swatch cycles the colour.
    let swatch_btn = button(swatch)
        .padding(0)
        .on_press(HexEditorMessage::CyclePatternColor(pat.id))
        .style(button::text);

    let start = text(format!("0x{:08X}", pat.start))
        .size(10)
        .font(Font::MONOSPACE);
    let end = text(format!("0x{:08X}", pat.end))
        .size(10)
        .font(Font::MONOSPACE);
    let size = text(pat.len().to_string()).size(10).font(Font::MONOSPACE);

    let remove_btn = button(text("✕").size(9).font(Font::MONOSPACE))
        .padding([1, 4])
        .on_press(HexEditorMessage::RemovePattern(pat.id));

    // ── Annotation text input on the right ───────────────────────────────
    let current = pat.annotation.as_deref().unwrap_or("");
    let ann_input = text_input("Annotation…", current)
        .on_input(move |s| HexEditorMessage::SetPatternAnnotation(pat.id, s))
        .size(9)
        .padding([1, 4])
        .width(Fill);

    let inner = row![
        swatch_btn,
        container(start).width(Length::Fixed(80.0)),
        container(end).width(Length::Fixed(80.0)),
        container(size).width(Length::Fixed(40.0)),
        remove_btn,
        ann_input,
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    let row = button(inner)
        .on_press(HexEditorMessage::NavigateToPattern(pat.id))
        .padding(iced::Padding {
            top: 3.0,
            right: 4.0,
            bottom: 3.0,
            left: 6.0,
        })
        .width(Fill)
        .style(button::text);

    // Indent group children.
    if pat.group_id.is_some() {
        container(row)
            .padding(iced::Padding {
                top: 0.0,
                right: 6.0,
                bottom: 0.0,
                left: 26.0,
            })
            .width(Fill)
            .into()
    } else {
        row.into()
    }
}
