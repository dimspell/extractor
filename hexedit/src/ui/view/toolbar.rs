use iced::widget::{button, container, row, text};
use iced::{Element, Fill, Font};

use crate::config::HexEditorConfig;
use crate::{HexEditorMessage, HexEditorState};

pub fn build_toolbar<'a>(
    editor: &'a HexEditorState,
    config: &HexEditorConfig,
) -> Element<'a, HexEditorMessage> {
    let can_save = config.can_save_now(editor);

    let save_label = config.save_label().to_string();
    let mut save_btn = button(text(save_label).size(11).font(Font::MONOSPACE)).padding([3, 10]);
    if can_save {
        save_btn = save_btn.on_press(HexEditorMessage::SaveIntoRecording);
    }

    let hint = config.save_hint.clone();

    // Check if an Inspector pane exists in the grid (Halloy-style).
    let has_inspector_pane = editor
        .panes
        .iter()
        .any(|(_, p)| matches!(p.content, crate::domain::panel::HexPanelContent::Inspector));
    let inspector_label = if has_inspector_pane {
        "Hide Inspector"
    } else {
        "Inspector"
    };
    let inspector_btn = button(text(inspector_label).size(11).font(Font::MONOSPACE))
        .padding([3, 10])
        .on_press(HexEditorMessage::ToggleInspector);

    // Check if a PatternList pane exists in the grid (Halloy-style).
    let has_patterns_pane = editor.panes.iter().any(|(_, p)| {
        matches!(
            p.content,
            crate::domain::panel::HexPanelContent::PatternList
        )
    });
    let patterns_label = if has_patterns_pane {
        "Hide Patterns"
    } else {
        "Patterns"
    };
    let patterns_btn = button(text(patterns_label).size(11).font(Font::MONOSPACE))
        .padding([3, 10])
        .on_press(HexEditorMessage::TogglePatternList);

    // Byte statistics toggle.
    let has_stats_pane = editor
        .panes
        .iter()
        .any(|(_, p)| matches!(p.content, crate::domain::panel::HexPanelContent::Statistics));
    let stats_label = if has_stats_pane {
        "Hide Stats"
    } else {
        "Stats"
    };
    let stats_btn = button(text(stats_label).size(11).font(Font::MONOSPACE))
        .padding([3, 10])
        .on_press(HexEditorMessage::ToggleStats);

    // Diff / comparison button.
    let has_comparison = editor.comparison_file.is_some();
    let diff_btn = if has_comparison {
        button(text("Close Diff").size(11).font(Font::MONOSPACE))
            .padding([3, 10])
            .on_press(HexEditorMessage::CloseComparison)
    } else {
        button(text("Diff Against…").size(11).font(Font::MONOSPACE))
            .padding([3, 10])
            .on_press(HexEditorMessage::LoadComparisonFile)
    };
    // Diff chunk navigation buttons (◀ ▶), visible when comparison loaded.
    let nav_prev_btn = if has_comparison {
        button(text("◀").size(11).font(Font::MONOSPACE))
            .padding([3, 6])
            .on_press(HexEditorMessage::DiffNavPrev)
    } else {
        button(text("◀").size(11).font(Font::MONOSPACE))
            .padding([3, 6])
            .style(button::secondary)
    };
    let nav_next_btn = if has_comparison {
        button(text("▶").size(11).font(Font::MONOSPACE))
            .padding([3, 6])
            .on_press(HexEditorMessage::DiffNavNext)
    } else {
        button(text("▶").size(11).font(Font::MONOSPACE))
            .padding([3, 6])
            .style(button::secondary)
    };

    let diff_review_btn = if has_comparison {
        let label = if editor.diff_review {
            "Full View"
        } else {
            "Show Diffs Only"
        };
        button(text(label).size(11).font(Font::MONOSPACE))
            .padding([3, 10])
            .on_press(HexEditorMessage::ToggleDiffReview)
    } else {
        button(text("Show Diffs Only").size(11).font(Font::MONOSPACE))
            .padding([3, 10])
            .style(button::secondary)
    };

    let export_btn = button(text("Export TXT").size(11).font(Font::MONOSPACE))
        .padding([3, 10])
        .on_press(HexEditorMessage::OpenExportConfig);

    let settings_btn = button(text("Settings").size(11).font(Font::MONOSPACE))
        .padding([3, 10])
        .on_press(HexEditorMessage::OpenSettings);

    // Bytes-per-row toggle group.
    let goto_btn = button(text("Go to...").size(11).font(Font::MONOSPACE))
        .padding([3, 10])
        .on_press(HexEditorMessage::OpenGotoDialog);

    let bpr = editor.bytes_per_row;
    let bpr_btn = |n: u8| {
        let label = format!("{:02}", n);
        let active = bpr == n;
        let mut btn = button(text(label).size(11).font(Font::MONOSPACE)).padding([3, 6]);
        if !active {
            btn = btn.style(button::text);
        }
        btn.on_press(HexEditorMessage::SetBytesPerRow(n))
    };

    let status: Element<'a, HexEditorMessage> = if editor.status_msg.is_empty() {
        text("").size(11).into()
    } else {
        text(editor.status_msg.clone())
            .size(11)
            .font(Font::MONOSPACE)
            .into()
    };

    container(
        row![
            save_btn,
            goto_btn,
            inspector_btn,
            patterns_btn,
            stats_btn,
            diff_btn,
            nav_prev_btn,
            nav_next_btn,
            diff_review_btn,
            export_btn,
            settings_btn,
            row![
                text("BPR").size(10).font(Font::MONOSPACE),
                bpr_btn(8),
                bpr_btn(16),
                bpr_btn(32),
            ]
            .spacing(2)
            .align_y(iced::Alignment::Center),
            text(hint).size(11).font(Font::MONOSPACE),
            container(status).width(Fill),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
    )
    .padding([4, 12])
    .width(Fill)
    .into()
}
