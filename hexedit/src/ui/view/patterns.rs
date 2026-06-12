//! Pattern list panel with accordion grouping for repeated patterns.
//!
//! Ungrouped patterns (created via "Create Pattern") appear as flat rows.
//! Repeated-pattern groups (created via "Add Repeated Pattern") appear under
//! a collapsible accordion header labelled with the group name.

use std::collections::BTreeMap;

use iced::widget::space::Space;
use iced::widget::{button, column, container, row, scrollable, text};
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

    // ── Render ungrouped patterns (flat rows, same as before) ────────────
    for pat in &ungrouped {
        col = col.push(pattern_row(pat));
    }

    // ── Render grouped patterns under accordion headers ───────────────────
    for gid in grouped.keys() {
        let group = editor.groups.iter().find(|g| g.id == *gid);
        if let Some(grp) = group {
            let children = &grouped[gid];
            let collapsed = editor.collapsed_groups.contains(&grp.id);
            col = col.push(group_header(grp, children.len(), collapsed));
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

    let toggle_char = if collapsed { "▶" } else { "▼" };
    let toggle = text(toggle_char).size(9).font(Font::MONOSPACE);

    let label = text(&grp.label).size(10).font(Font::MONOSPACE);
    let count = text(format!("({})", child_count))
        .size(10)
        .color(color!(0x8a7a6a))
        .font(Font::MONOSPACE);

    let inner = row![
        toggle,
        swatch,
        label,
        count,
        Space::default().width(Fill),
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

/// A single pattern row — used both for ungrouped patterns and for children
/// of a group accordion.
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

    let inner = row![
        swatch,
        container(start).width(Length::Fixed(80.0)),
        container(end).width(Length::Fixed(80.0)),
        container(size).width(Length::Fixed(40.0)),
        remove_btn,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let row = button(inner)
        .on_press(HexEditorMessage::NavigateToPattern(pat.id))
        .padding([3, 6])
        .width(Fill)
        .style(button::text);

    // Indent group children with left padding.
    if pat.group_id.is_some() {
        let left_pad: f32 = 20.0;
        container(row)
            .padding(iced::Padding {
                top: 3.0,
                right: 6.0,
                bottom: 3.0,
                left: 6.0 + left_pad,
            })
            .width(Fill)
            .into()
    } else {
        row.into()
    }
}
