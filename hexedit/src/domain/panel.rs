//! Pane grid types for the hex editor's moveable panels.
//!
//! Following the Halloy IRC client pattern ([`pane_grid::State`]), the hex
//! editor's body area is a split-pane grid where users can rearrange, resize,
//! split, and close panels. Each panel contains one of the hex editor's
//! sub-views (matrix, inspector, pattern list).

use iced::widget::pane_grid;

/// Which view is shown inside a hex editor panel.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum HexPanelContent {
    /// The hex byte matrix (address column, hex dump, ASCII sidebar).
    #[default]
    Matrix,
    /// The decoded-value inspector (cursor-relative decodes).
    Inspector,
    /// The pattern list with create/edit/delete/group operations.
    PatternList,
}

/// A single panel in the hex editor's pane grid.
#[derive(Debug, Clone)]
pub struct HexPanel {
    pub content: HexPanelContent,
}

impl HexPanel {
    pub fn new(content: HexPanelContent) -> Self {
        Self { content }
    }
}

impl From<HexPanelContent> for HexPanel {
    fn from(content: HexPanelContent) -> Self {
        Self::new(content)
    }
}

/// Build the default pane layout: a vertical split with the matrix on the left
/// and the inspector on the right.
pub fn default_pane_grid() -> pane_grid::State<HexPanel> {
    let (mut state, matrix_pane) =
        pane_grid::State::new(HexPanel::new(HexPanelContent::Matrix));

    if let Some((_, split)) = state.split(
        pane_grid::Axis::Vertical,
        matrix_pane,
        HexPanel::new(HexPanelContent::Inspector),
    ) {
        // Give the inspector panel ~25% of the width (~280px on a 1100px
        // window) instead of the default 50/50 split.
        state.resize(split, 0.75);
    }

    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_has_two_panes() {
        let state = default_pane_grid();
        assert_eq!(state.len(), 2);
    }

    #[test]
    fn default_layout_contains_matrix_and_inspector() {
        let state = default_pane_grid();
        let contents: Vec<HexPanelContent> = state
            .iter()
            .map(|(_, p)| p.content)
            .collect();
        assert!(contents.contains(&HexPanelContent::Matrix));
        assert!(contents.contains(&HexPanelContent::Inspector));
    }

    #[test]
    fn hex_panel_from_content() {
        let panel: HexPanel = HexPanelContent::Matrix.into();
        assert_eq!(panel.content, HexPanelContent::Matrix);
    }
}
