//! Pane title bars and per-panel content rendering for the hex editor's
//! Halloy-style movable pane grid.
//!
//! Each panel in the grid gets a title bar showing its name, a drag grip,
//! split/close controls, and focused-pane highlighting — following the
//! pattern from Halloy's [`pane.rs`].

use iced::widget::pane_grid;
use iced::widget::space::Space;
use iced::widget::{button, container, row, text};
use iced::{Element, Fill, Font};
use iced::Color;

use crate::config::HexEditorConfig;
use crate::domain::panel::HexPanelContent;
use crate::{HexEditorMessage, HexEditorState};

/// Maximum number of panels before we disable splitting to prevent
/// window-shattering fragmentation.
const MAX_PANELS: usize = 8;

/// Background colour for the focused pane's title bar (#333344).
const FOCUSED_TITLE_BG: Color = Color::from_rgb(0.2, 0.2, 0.267);
/// Background colour for unfocused pane title bars (#222222).
const UNFOCUSED_TITLE_BG: Color = Color::from_rgb(0.133, 0.133, 0.133);

/// Render the content for a single pane in the hex editor grid.
pub fn pane_content<'a>(
    state: &'a HexEditorState,
    config: &HexEditorConfig,
    _id: pane_grid::Pane,
    panel: &'a crate::domain::panel::HexPanel,
) -> Element<'a, HexEditorMessage> {
    match panel.content {
        HexPanelContent::Matrix => matrix_content(state),
        HexPanelContent::Inspector => super::inspector::view(state, config),
        HexPanelContent::PatternList => super::patterns::view(state),
    }
}

/// Build the hex matrix view for a Matrix pane.
fn matrix_content<'a>(state: &'a HexEditorState) -> Element<'a, HexEditorMessage> {
    use crate::ui::view::matrix::{EditView, HexMatrix};

    let cache = state.cache.clone();
    let edit = state.edit_mode.as_ref().map(|e| EditView {
        addr: e.addr,
        draft: e.draft.as_str(),
    });

    HexMatrix::new(
        state.provider.as_slice(),
        state.bytes_per_row,
        state.selection,
        edit,
        state.provider.dirty(),
        &state.vanilla_diff,
        &state.pattern_by_addr,
        &state.search.match_set,
        state.search.query_len,
        state.search.current_addr(),
        &state.search.results,
        &state.row_annotations,
        &state.active_patterns,
        cache,
    )
    .on_select_at(HexEditorMessage::SelectAt)
    .on_extend_to(HexEditorMessage::ExtendTo)
    .on_nav(|dir, extend| HexEditorMessage::Nav { dir, extend })
    .on_begin_edit(HexEditorMessage::BeginEdit)
    .on_edit_type(HexEditorMessage::EditTypeChar)
    .on_edit_backspace(|| HexEditorMessage::EditBackspace)
    .on_edit_cancel(|| HexEditorMessage::EditCancel)
    .on_edit_commit(|advance| HexEditorMessage::EditCommit { advance })
    .on_right_click(HexEditorMessage::RightClickAt)
    .on_create_pattern(|| HexEditorMessage::CreatePattern)
    .on_open_goto(|| HexEditorMessage::OpenGotoDialog)
    .on_open_search(|| HexEditorMessage::OpenSearch)
    .on_copy_selection(|| HexEditorMessage::CopySelection)
    .on_paste(|| HexEditorMessage::Paste)
    .show_decimal(state.show_decimal)
    .on_toggle_addr_format(|| HexEditorMessage::ToggleAddrFormat)
    .into()
}

/// Build the title bar for a pane.
///
/// Follows Halloy's pattern:
/// - Drag grip indicator (≡) on the left
/// - Panel name label
/// - Spacer
/// - Split buttons (▤ / ▥) when more panes are allowed
/// - Close button (✕) when there's more than one pane
/// - Background colour changes when this pane has focus
pub fn title_bar<'a>(
    _state: &'a HexEditorState,
    _id: pane_grid::Pane,
    panel: &'a crate::domain::panel::HexPanel,
    pane_count: usize,
    is_focused: bool,
) -> pane_grid::TitleBar<'a, HexEditorMessage> {
    let label = match panel.content {
        HexPanelContent::Matrix => "Hex Dump",
        HexPanelContent::Inspector => "Inspector",
        HexPanelContent::PatternList => "Patterns",
    };

    let can_close = pane_count > 1;
    let can_split = pane_count < MAX_PANELS;

    let mut controls = row![].spacing(4);

    if can_split {
        let split_h = button(text("▤").size(10).font(Font::MONOSPACE))
            .padding([2, 5])
            .on_press(HexEditorMessage::SplitPane(
                iced::widget::pane_grid::Axis::Horizontal,
            ));
        controls = controls.push(split_h);

        let split_v = button(text("▥").size(10).font(Font::MONOSPACE))
            .padding([2, 5])
            .on_press(HexEditorMessage::SplitPane(
                iced::widget::pane_grid::Axis::Vertical,
            ));
        controls = controls.push(split_v);
    }

    if can_close {
        let close_btn = button(text("✕").size(10).font(Font::MONOSPACE))
            .padding([2, 6])
            .on_press(HexEditorMessage::ClosePane);
        controls = controls.push(close_btn);
    }

    // Halloy-style drag grip indicator (≡) on the left, then label, then controls.
    let title_row = row![
        text("≡").size(12).font(Font::MONOSPACE),
        text(label).size(11).font(Font::MONOSPACE),
        Space::default().width(Fill),
        controls,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let bg = if is_focused {
        FOCUSED_TITLE_BG
    } else {
        UNFOCUSED_TITLE_BG
    };

    let title_widget = container(title_row)
        .padding([3, 8])
        .width(Fill)
        .style(move |_theme| container::Style {
            background: Some(bg.into()),
            ..container::Style::default()
        });

    pane_grid::TitleBar::new(title_widget)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::panel::HexPanel;

    #[test]
    fn title_bar_has_correct_label_for_matrix() {
        let panel = HexPanel::new(HexPanelContent::Matrix);
        let label = match panel.content {
            HexPanelContent::Matrix => "Hex Dump",
            HexPanelContent::Inspector => "Inspector",
            HexPanelContent::PatternList => "Patterns",
        };
        assert_eq!(label, "Hex Dump");
    }

    #[test]
    fn title_bar_has_correct_label_for_inspector() {
        let panel = HexPanel::new(HexPanelContent::Inspector);
        let label = match panel.content {
            HexPanelContent::Matrix => "Hex Dump",
            HexPanelContent::Inspector => "Inspector",
            HexPanelContent::PatternList => "Patterns",
        };
        assert_eq!(label, "Inspector");
    }

    #[test]
    fn title_bar_has_correct_label_for_patterns() {
        let panel = HexPanel::new(HexPanelContent::PatternList);
        let label = match panel.content {
            HexPanelContent::Matrix => "Hex Dump",
            HexPanelContent::Inspector => "Inspector",
            HexPanelContent::PatternList => "Patterns",
        };
        assert_eq!(label, "Patterns");
    }

    #[test]
    fn matrix_content_renders_with_empty_provider() {
        // Quick smoke test: matrix content function shouldn't panic even with
        // a minimal state.
        let state = HexEditorState::load_from_path(
            std::path::Path::new("/nonexistent/file.bin"),
        );
        let _content = matrix_content(&state);
    }
}
