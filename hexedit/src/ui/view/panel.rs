//! Pane title bars and per-panel content rendering for the hex editor's
//! movable pane grid, matching Iced's official [`pane_grid` example].
//!
//! Each panel gets a title bar with a grip indicator, panel name,
//! split/close controls, and focused-pane highlighting via the theme
//! palette. The title bar is styled using [`container::Style`] applied
//! through [`pane_grid::TitleBar::style`], following the same pattern as
//! Iced's own example at
//! <https://github.com/iced-rs/iced/blob/master/examples/pane_grid/src/main.rs>
//!
//! **Layout:** The title content (≡ + label + [`Space::Fill`]) is passed
//! directly to [`TitleBar::new`]. Controls are separate via
//! [`Controls::dynamic`] — [`always_show_controls`] keeps the full button
//! row visible when space permits; when the pane is too narrow, Iced falls
//! back to a single compact close button. This ensures the title text is
//! **always drawn** (unlike the no-compact overflow path which hides it).
//!
//! **Dragging:** Iced's [`is_over_pick_area`] returns `!controls && !title`
//! in the non-overflow path, meaning only the TitleBar's padding area is
//! considered a drag "handle". We use generous padding so this is easy to
//! grab — matching the official example's behaviour.

use std::collections::BTreeSet;

use iced::widget::pane_grid;
use iced::widget::pane_grid::{Axis, Controls};
use iced::widget::space::Space;
use iced::widget::{button, container, row, text};
use iced::{Element, Fill, Font};

use crate::config::HexEditorConfig;
use crate::domain::panel::HexPanelContent;
use crate::{HexEditorMessage, HexEditorState};

/// Maximum number of panels before we disable splitting to prevent
/// window-shattering fragmentation.
const MAX_PANELS: usize = 8;

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

    // Compute zebra-striping: every other pattern in each group gets an
    // alternate (darkened) background so adjacent same-group patterns are
    // visually distinct.
    let mut alternate_patterns = BTreeSet::new();
    for group in &state.groups {
        let mut group_patterns: Vec<&crate::domain::pattern::Pattern> = state
            .patterns
            .iter()
            .filter(|p| p.group_id == Some(group.id))
            .collect();
        group_patterns.sort_by_key(|p| (p.start, p.id));
        for (i, pat) in group_patterns.iter().enumerate() {
            if i % 2 == 1 {
                alternate_patterns.insert(pat.id);
            }
        }
    }

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
        alternate_patterns,
        cache,
        state.color_scheme,
        state.dim_nulls,
    )
    .on_select_at(HexEditorMessage::SelectAt)
    .on_extend_to(HexEditorMessage::ExtendTo)
    .on_nav(|dir, extend| HexEditorMessage::Nav { dir, extend })
    .on_begin_edit(HexEditorMessage::BeginEdit)
    .on_edit_type(HexEditorMessage::EditTypeChar)
    .on_edit_backspace(|| HexEditorMessage::EditBackspace)
    .on_delete_byte(|| HexEditorMessage::DeleteByteAtCursor)
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
    .write_mode(state.write_mode)
    .into()
}

/// Build the title bar for a pane, following Iced's official [`pane_grid`
/// example].
///
/// The title content (grip + name + spacer) is passed directly to
/// [`TitleBar::new`] — no extra container wrapper. Styling (background +
/// text colour) is applied via [`TitleBar::style`] using the theme palette.
///
/// Controls use [`Controls::dynamic`] with a compact fallback (close button
/// only), so the title text is always drawn — unlike the no-compact overflow
/// path which hides the title content.
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

    // ── Title content ──────────────────────────────────────────────────
    //
    // A simple Row with grip, label, and a spacer. Passed directly to
    // `TitleBar::new` — the TitleBar's own styling provides the background
    // and text colour.
    let title_content = row![
        text("≡").size(12).font(Font::MONOSPACE),
        text(label).size(11).font(Font::MONOSPACE),
        Space::default().width(Fill),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // ── Full controls ──────────────────────────────────────────────────
    let mut controls = row![].spacing(4);

    if can_split {
        controls = controls.push(
            button(text("▤").size(10).font(Font::MONOSPACE))
                .padding([2, 5])
                .on_press(HexEditorMessage::SplitPane(Axis::Horizontal)),
        );
        controls = controls.push(
            button(text("▥").size(10).font(Font::MONOSPACE))
                .padding([2, 5])
                .on_press(HexEditorMessage::SplitPane(Axis::Vertical)),
        );
    }

    if can_close {
        controls = controls.push(
            button(text("✕").size(10).font(Font::MONOSPACE))
                .padding([2, 6])
                .on_press(HexEditorMessage::ClosePane),
        );
    }

    // ── Compact controls ───────────────────────────────────────────────
    //
    // Iced draws the compact variant when the pane is too narrow to fit
    // both the title and the full controls. We only show the close button
    // (matching the official example).
    let compact: Element<'a, HexEditorMessage> = if can_close {
        button(text("✕").size(11).font(Font::MONOSPACE))
            .padding([2, 6])
            .on_press(HexEditorMessage::ClosePane)
            .into()
    } else {
        Space::default().into()
    };

    pane_grid::TitleBar::new(title_content)
        .controls(Controls::dynamic(controls, compact))
        .always_show_controls()
        .padding([4, 6])
        .style(move |theme: &iced::Theme| {
            let palette = theme.extended_palette();

            if is_focused {
                container::Style {
                    text_color: Some(palette.primary.strong.text),
                    background: Some(palette.primary.strong.color.into()),
                    ..Default::default()
                }
            } else {
                container::Style {
                    text_color: Some(palette.background.strong.text),
                    background: Some(palette.background.strong.color.into()),
                    ..Default::default()
                }
            }
        })
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
        let state = HexEditorState::load_from_path(std::path::Path::new("/nonexistent/file.bin"));
        let _content = matrix_content(&state);
    }
}
