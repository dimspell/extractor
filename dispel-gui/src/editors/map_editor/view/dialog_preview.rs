use std::collections::HashSet;

use crate::components::utils::{horizontal_rule, horizontal_space};
use crate::message::{Message, MessageExt};
use crate::style;
use dispel_core::references::dialogue_paragraph::DialogueParagraph;
use dispel_core::references::dialogue_script::DialogueScript;
use dispel_core::references::enums::{DialogOwner, DialogType};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::Element;

use super::super::message::MapEditorMessage;
use super::super::state::{DialogPreviewState, MapEditorState};
use gui_widgets::lucide::{icon_char, LUCIDE_FONT};
use lucide_icons::Icon;

// ── Dialog Preview ─────────────────────────────────────────────────────────────

/// A single line in the rendered dialog tree.
struct DialogTreeLine {
    depth: usize,
    option_label: Option<String>,
    speaker: String,
    text: String,
    script_id: i32,
    next_labels: Vec<String>,
    has_trigger: bool,
    trigger_id: i32,
    has_prereq: bool,
    prereq_id: i32,
    is_end: bool,
}

/// Recursively walk the dialog tree, appending lines in display order.
#[allow(clippy::too_many_arguments)]
fn build_tree_lines(
    start_id: i32,
    scripts: &[DialogueScript],
    paragraphs: &[DialogueParagraph],
    depth: usize,
    option_label: Option<String>,
    visited: &mut HashSet<i32>,
    lines: &mut Vec<DialogTreeLine>,
) {
    const MAX_DEPTH: usize = 30;
    if depth > MAX_DEPTH || start_id == 0 {
        return;
    }
    if !visited.insert(start_id) {
        lines.push(DialogTreeLine {
            depth,
            option_label,
            speaker: String::new(),
            text: format!("(see dialog node #{})", start_id),
            script_id: start_id,
            next_labels: vec![],
            has_trigger: false,
            trigger_id: 0,
            has_prereq: false,
            prereq_id: 0,
            is_end: true,
        });
        return;
    }

    let Some(script) = scripts.iter().find(|s| s.id == start_id) else {
        return;
    };

    let text = script
        .dialog_id
        .and_then(|did| paragraphs.iter().find(|p| p.id == did))
        .map(|p| p.text.as_str())
        .unwrap_or("[text not found]");

    let speaker = match script.dialog_owner {
        Some(DialogOwner::Npc) => "NPC",
        Some(DialogOwner::Player) => "Player",
        None => "?",
    };

    let (next_ids, is_end): (Vec<(String, i32)>, bool) = match script.dialog_type {
        Some(DialogType::Choice) => {
            let mut v = Vec::new();
            if let Some(id) = script.next_dialog_id1.filter(|&id| id != 0) {
                v.push(("A".to_string(), id));
            }
            if let Some(id) = script.next_dialog_id2.filter(|&id| id != 0) {
                v.push(("B".to_string(), id));
            }
            if let Some(id) = script.next_dialog_id3.filter(|&id| id != 0) {
                v.push(("C".to_string(), id));
            }
            (v, false)
        }
        _ => {
            if let Some(id) = script
                .next_dialog_to_check
                .or(script.next_dialog_id1)
                .filter(|&id| id != 0)
            {
                (vec![("→".to_string(), id)], false)
            } else {
                (vec![], true)
            }
        }
    };

    let next_labels: Vec<String> = next_ids
        .iter()
        .map(|(l, id)| format!("{l}[{id}]"))
        .collect();

    lines.push(DialogTreeLine {
        depth,
        option_label,
        speaker: speaker.to_string(),
        text: text.to_string(),
        script_id: script.id,
        next_labels,
        has_trigger: script.triggered_event_id.is_some_and(|id| id != 0),
        trigger_id: script.triggered_event_id.unwrap_or(0),
        has_prereq: script.required_event_id.is_some_and(|id| id != 0),
        prereq_id: script.required_event_id.unwrap_or(0),
        is_end,
    });

    for (label, next_id) in &next_ids {
        build_tree_lines(
            *next_id,
            scripts,
            paragraphs,
            depth + 1,
            Some(label.clone()),
            visited,
            lines,
        );
    }
}

/// Render the dialog preview modal.
pub fn view_dialog_preview<'a>(
    state: &'a MapEditorState,
    tab_id: usize,
    preview: &DialogPreviewState,
) -> Element<'a, Message> {
    let entry_id = state
        .data
        .npcs
        .get(preview.npc_index)
        .map(|n| n.dialog_id)
        .unwrap_or(0);

    let mut lines = Vec::new();
    build_tree_lines(
        entry_id,
        &preview.dialog_scripts,
        &preview.dialog_paragraphs,
        0,
        None,
        &mut HashSet::new(),
        &mut lines,
    );

    let npc_label = format!("Dialog Preview — NPC #{}", preview.npc_index);

    if lines.is_empty() {
        return container(
            column![
                text(npc_label).size(14).style(style::primary_text),
                horizontal_rule(1),
                text("No dialog found for this NPC.")
                    .size(11)
                    .style(style::subtle_text),
            ]
            .spacing(12)
            .padding(20)
            .width(iced::Length::Fixed(480.0)),
        )
        .style(style::toolbar_container)
        .into();
    }

    let header_row: Element<'_, Message> = row![
        text(npc_label).size(14).style(style::primary_text),
        horizontal_space(),
        button(text(icon_char(Icon::X)).font(LUCIDE_FONT).size(14))
            .on_press(Message::map_editor(MapEditorMessage::HideDialogPreview(
                tab_id,
            )))
            .padding([2, 8])
            .style(style::browse_button),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into();

    let entries: Element<'_, Message> = column(lines.into_iter().map(|line| {
        let left_pad = line.depth as f32 * 24.0;

        // Build label prefix (option letter or empty)
        let prefix = line
            .option_label
            .as_deref()
            .map(|l| format!("{l} "))
            .unwrap_or_default();

        // Speaker label
        let speaker_colored = match line.speaker.as_str() {
            "NPC" => "NPC",
            "Player" => "Player",
            _ => "?",
        };

        // Truncate long text
        let display_text = if line.text.len() > 100 {
            format!("{}…", &line.text[..97])
        } else {
            line.text.clone()
        };

        // Build suffix (events, end marker, next links)
        let mut suffix = String::new();
        if line.has_prereq {
            suffix.push_str(&format!(" [req#{}]", line.prereq_id));
        }
        if line.has_trigger {
            suffix.push_str(&format!(" [ev#{}]", line.trigger_id));
        }
        if !line.next_labels.is_empty() {
            suffix.push_str(&format!(" ─ {}", line.next_labels.join(" ")));
        } else if line.is_end {
            suffix.push_str(" ─ [END]");
        }

        let full_text = format!(
            "{prefix}[{id}] {speaker}: \"{text}\"{suffix}",
            id = line.script_id,
            speaker = speaker_colored,
            text = display_text,
            suffix = suffix,
        );

        container(text(full_text).size(11))
            .padding([2.0f32, left_pad + 8.0])
            .into()
    }))
    .spacing(1)
    .into();

    container(
        column![
            header_row,
            horizontal_rule(1),
            scrollable(entries).height(iced::Length::Fill),
        ]
        .spacing(8)
        .padding(16)
        .width(iced::Length::Fixed(600.0))
        .height(iced::Length::Shrink),
    )
    .style(style::toolbar_container)
    .height(550.0)
    .into()
}
