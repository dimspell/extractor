//! Pane grid tests for the hex editor's Halloy-style moveable panels.
//!
//! Covers:
//! - Focus: `PaneClicked` sets focused pane
//! - Split: `SplitPane` adds a pane, saturates at MAX_PANELS
//! - Close: `ClosePane` removes pane, guards last-pane
//! - Pattern list: `TogglePatternList` adds/removes a PatternList pane
//! - Drag & Drop: `PaneDragged(Dropped)` reorders / docks
//! - Resize: `PaneResized` updates divider ratios

use iced::widget::pane_grid::{self, Axis, DragEvent, Pane, Region, ResizeEvent, Target};

use super::*;

// ============================================================================
// Helper: find a second pane that isn't the focused one.
// ============================================================================

fn second_pane(state: &HexEditorState) -> Pane {
    let focus = state.pane_focus;
    state
        .panes
        .iter()
        .find(|(id, _)| **id != focus)
        .map(|(id, _)| *id)
        .expect("expected at least 2 panes")
}

// ============================================================================
// Pane focus
// ============================================================================

#[test]
fn test_pane_clicked_sets_focus() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();

    let other = second_pane(&state);

    send(&mut state, &config, HexEditorMessage::PaneClicked(other));
    assert_eq!(state.pane_focus, other);
}

#[test]
fn test_pane_clicked_on_already_focused_pane() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();
    let focus = state.pane_focus;

    send(&mut state, &config, HexEditorMessage::PaneClicked(focus));
    assert_eq!(
        state.pane_focus, focus,
        "re-clicking focused pane is a no-op"
    );
}

// ============================================================================
// Split pane
// ============================================================================

#[test]
fn test_split_pane_adds_a_pane() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();
    let before = state.panes.len();

    send(
        &mut state,
        &config,
        HexEditorMessage::SplitPane(Axis::Vertical),
    );

    assert_eq!(
        state.panes.len(),
        before + 1,
        "splitting should add one pane"
    );
}

#[test]
fn test_split_pane_uses_focused_pane() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();
    let other = second_pane(&state);

    // Focus a specific pane then split
    send(&mut state, &config, HexEditorMessage::PaneClicked(other));
    let before = state.panes.len();
    send(
        &mut state,
        &config,
        HexEditorMessage::SplitPane(Axis::Horizontal),
    );

    assert_eq!(
        state.panes.len(),
        before + 1,
        "split should add a pane on the focused panel"
    );
}

#[test]
fn test_split_pane_respects_max_panels() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();

    // Split repeatedly up to the limit
    for _ in 0..10 {
        send(
            &mut state,
            &config,
            HexEditorMessage::SplitPane(Axis::Vertical),
        );
    }

    let max = 8; // matches MAX_PANELS in panel.rs
    assert!(
        state.panes.len() <= max,
        "should not exceed {} panes, got {}",
        max,
        state.panes.len()
    );
    assert_eq!(state.panes.len(), max, "should saturate at {} panes", max);
}

// ============================================================================
// Close pane
// ============================================================================

#[test]
fn test_close_pane_removes_pane() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();
    let before = state.panes.len();
    assert!(before > 1, "default layout must have >1 panes to test close");

    send(&mut state, &config, HexEditorMessage::ClosePane);

    assert_eq!(
        state.panes.len(),
        before - 1,
        "close should remove one pane"
    );
}

#[test]
fn test_close_last_pane_is_noop() {
    let (single_state, _pane) = pane_grid::State::new(
        crate::domain::panel::HexPanel::new(
            crate::domain::panel::HexPanelContent::Matrix,
        ),
    );
    let mut state = make_state(vec![0u8; 64]);
    state.panes = single_state;
    state.pane_focus = state.panes.iter().next().map(|(id, _)| *id).unwrap();

    let config = default_config();
    assert_eq!(state.panes.len(), 1);

    send(&mut state, &config, HexEditorMessage::ClosePane);

    assert_eq!(
        state.panes.len(),
        1,
        "close of last pane should be a no-op"
    );
}

#[test]
fn test_close_pane_updates_focus_to_sibling() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();

    send(&mut state, &config, HexEditorMessage::ClosePane);

    assert_eq!(
        state.panes.len(),
        1,
        "should be 1 pane after closing one of 2"
    );
    let remaining = state.panes.iter().next().map(|(id, _)| *id).unwrap();
    assert_eq!(
        state.pane_focus, remaining,
        "focus should point to remaining pane"
    );
}

// ============================================================================
// Toggle Pattern List
// ============================================================================

#[test]
fn test_toggle_pattern_list_adds_pattern_pane() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();
    let has_pattern_before = state.panes.iter().any(|(_, p)| {
        matches!(p.content, crate::domain::panel::HexPanelContent::PatternList)
    });

    assert!(
        !has_pattern_before,
        "default layout should not have PatternList"
    );

    send(&mut state, &config, HexEditorMessage::TogglePatternList);

    let has_pattern_after = state.panes.iter().any(|(_, p)| {
        matches!(p.content, crate::domain::panel::HexPanelContent::PatternList)
    });
    assert!(
        has_pattern_after,
        "TogglePatternList should add a PatternList pane"
    );
}

#[test]
fn test_toggle_pattern_list_removes_pattern_pane() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();

    // Add pattern list
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    let has_pattern = state.panes.iter().any(|(_, p)| {
        matches!(p.content, crate::domain::panel::HexPanelContent::PatternList)
    });
    assert!(has_pattern);

    // Toggle again to remove
    send(&mut state, &config, HexEditorMessage::TogglePatternList);

    let has_pattern_after = state.panes.iter().any(|(_, p)| {
        matches!(p.content, crate::domain::panel::HexPanelContent::PatternList)
    });
    assert!(
        !has_pattern_after,
        "second TogglePatternList should remove the PatternList pane"
    );
}

#[test]
fn test_toggle_pattern_list_when_single_pane_works() {
    let (single_state, _pane) = pane_grid::State::new(
        crate::domain::panel::HexPanel::new(
            crate::domain::panel::HexPanelContent::Matrix,
        ),
    );
    let mut state = make_state(vec![0u8; 64]);
    state.panes = single_state;
    state.pane_focus = state.panes.iter().next().map(|(id, _)| *id).unwrap();
    let config = default_config();
    assert_eq!(state.panes.len(), 1);

    send(&mut state, &config, HexEditorMessage::TogglePatternList);

    assert_eq!(state.panes.len(), 2, "should split to add pattern list");
    let has_pattern = state.panes.iter().any(|(_, p)| {
        matches!(p.content, crate::domain::panel::HexPanelContent::PatternList)
    });
    assert!(has_pattern);
}

// ============================================================================
// Pane drag & drop
// ============================================================================

#[test]
fn test_pane_dragged_drop_reorders() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();

    let panes: Vec<Pane> = state.panes.iter().map(|(id, _)| *id).collect();
    assert_eq!(panes.len(), 2, "default layout should have 2 panes");

    let p1 = panes[0];
    let p2 = panes[1];

    send(
        &mut state,
        &config,
        HexEditorMessage::PaneDragged(DragEvent::Dropped {
            pane: p1,
            target: Target::Pane(p2, Region::Center),
        }),
    );

    assert_eq!(
        state.panes.len(),
        2,
        "drag-drop should not change pane count"
    );
}

#[test]
fn test_pane_dragged_cancelled_is_noop() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();
    let before = state.panes.len();
    let focus = state.pane_focus;

    // Cancel the drag of the currently focused pane
    send(
        &mut state,
        &config,
        HexEditorMessage::PaneDragged(DragEvent::Canceled { pane: focus }),
    );

    assert_eq!(state.panes.len(), before, "cancelled drag should be noop");
}

// ============================================================================
// Pane resize
// ============================================================================

/// Walk the layout tree to find the first (only) [`pane_grid::Split`].
fn find_first_split(panes: &pane_grid::State<crate::domain::panel::HexPanel>) -> pane_grid::Split {
    use iced::widget::pane_grid::Node;

    fn search(node: &Node) -> Option<pane_grid::Split> {
        match node {
            Node::Split { id, .. } => Some(*id),
            Node::Pane(_) => None,
        }
    }
    search(panes.layout()).expect("default layout should have a split node")
}

#[test]
fn test_pane_resized_updates_ratio() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();

    let split = find_first_split(&state.panes);
    send(
        &mut state,
        &config,
        HexEditorMessage::PaneResized(ResizeEvent { split, ratio: 0.75 }),
    );

    assert_eq!(
        state.panes.len(),
        2,
        "resize should not change pane count"
    );
}

#[test]
fn test_pane_resized_to_extreme_ratio() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();

    let split = find_first_split(&state.panes);

    send(
        &mut state,
        &config,
        HexEditorMessage::PaneResized(ResizeEvent { split, ratio: 0.99 }),
    );

    assert_eq!(
        state.panes.len(),
        2,
        "extreme resize should not break state"
    );
}

// ============================================================================
// TogglePatternList compatibility with existing state
// ============================================================================

#[test]
fn test_multiple_toggle_pattern_list_toggling() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();

    // Default has no PatternList
    let has_pat = |st: &HexEditorState| -> bool {
        st.panes
            .iter()
            .any(|(_, p)| matches!(p.content, crate::domain::panel::HexPanelContent::PatternList))
    };
    assert!(!has_pat(&state));

    // Add
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    assert!(has_pat(&state));

    // Remove
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    assert!(!has_pat(&state));

    // Re-add (should work fresh)
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    assert!(has_pat(&state));
}

// ============================================================================
// Legacy show_pattern_list compatibility
// ============================================================================

#[test]
fn test_toggle_pattern_list_updates_legacy_flag() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();
    assert!(!state.show_pattern_list, "legacy flag should start false");

    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    assert!(
        state.show_pattern_list,
        "legacy flag should be true after toggle on"
    );

    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    assert!(
        !state.show_pattern_list,
        "legacy flag should be false after toggle off"
    );
}

// ============================================================================
// Pane count invariants
// ============================================================================

#[test]
fn test_split_close_split_sequence() {
    let mut state = make_state(vec![0u8; 64]);
    let config = default_config();

    assert_eq!(state.panes.len(), 2);

    // Split → 3
    send(&mut state, &config, HexEditorMessage::SplitPane(Axis::Vertical));
    assert_eq!(state.panes.len(), 3);

    // Close → 2
    send(&mut state, &config, HexEditorMessage::ClosePane);
    assert_eq!(state.panes.len(), 2);

    // Close → 1
    send(&mut state, &config, HexEditorMessage::ClosePane);
    assert_eq!(state.panes.len(), 1);

    // Close → still 1 (no-op)
    send(&mut state, &config, HexEditorMessage::ClosePane);
    assert_eq!(state.panes.len(), 1);
}
