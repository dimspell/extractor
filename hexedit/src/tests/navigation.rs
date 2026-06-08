use super::*;

// ============================================================================
// Navigation
// ============================================================================

#[test]
fn test_select_at_sets_cursor() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(42));
    assert_eq!(state.selection.cursor, 42);
    assert!(state.selection.is_single(), "selection should be single");
}

#[test]
fn test_extend_to_creates_range() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(30));
    assert_eq!(state.selection.start(), 10);
    assert_eq!(state.selection.end(), 30);
}

#[test]
fn test_navigation_right() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Nav {
            dir: NavDir::Right,
            extend: false,
        },
    );
    assert_eq!(state.selection.cursor, 1);
}

#[test]
fn test_navigation_down() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Nav {
            dir: NavDir::Down,
            extend: false,
        },
    );
    assert_eq!(state.selection.cursor, 16);
}

#[test]
fn test_navigation_with_extend() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Nav {
            dir: NavDir::Right,
            extend: true,
        },
    );
    // cursor moved to 1, but anchor is still at 0
    assert_eq!(state.selection.cursor, 1);
    assert_eq!(state.selection.anchor, 0);
    assert!(!state.selection.is_single(), "should extend selection");
}

#[test]
fn test_navigation_at_bounds() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    // Navigate left at cursor 0 should saturate.
    send(
        &mut state,
        &config,
        HexEditorMessage::Nav {
            dir: NavDir::Left,
            extend: false,
        },
    );
    assert_eq!(state.selection.cursor, 0, "left at 0 should stay at 0");

    // Navigate to document end.
    send(
        &mut state,
        &config,
        HexEditorMessage::Nav {
            dir: NavDir::DocumentEnd,
            extend: false,
        },
    );
    assert_eq!(state.selection.cursor, 255, "document end should be at max_addr");
}

#[test]
fn test_navigation_on_empty_file_does_nothing() {
    let mut state = make_state(vec![]);
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Nav {
            dir: NavDir::Right,
            extend: false,
        },
    );
    assert_eq!(state.selection.cursor, 0, "empty file navigation should be no-op");
}

// ============================================================================
// Navigation — extended
// ============================================================================

#[test]
fn test_nav_page_down() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Nav { dir: NavDir::PageDown, extend: false });
    // PAGE_ROWS = 24, so cursor moves from 0 to 24*16 = 384, clamped to 255
    assert_eq!(state.selection.cursor, 255, "PageDown should saturate at max_addr");
}

#[test]
fn test_nav_page_up() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    // Go to end first
    send(&mut state, &config, HexEditorMessage::Nav { dir: NavDir::DocumentEnd, extend: false });
    assert_eq!(state.selection.cursor, 255);
    // Page up from 255: 255.saturating_sub(24*16) = 255-384 = 0
    send(&mut state, &config, HexEditorMessage::Nav { dir: NavDir::PageUp, extend: false });
    assert_eq!(state.selection.cursor, 0, "PageUp from end should saturate at 0");
}

#[test]
fn test_nav_document_start() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Nav { dir: NavDir::DocumentEnd, extend: false });
    assert_eq!(state.selection.cursor, 255);
    send(&mut state, &config, HexEditorMessage::Nav { dir: NavDir::DocumentStart, extend: false });
    assert_eq!(state.selection.cursor, 0);
}

#[test]
fn test_nav_down_saturates_at_end() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    // 256 bytes, 16 BPR = 16 rows. Last row starts at 240. Go there.
    send(&mut state, &config, HexEditorMessage::SelectAt(240));
    send(&mut state, &config, HexEditorMessage::Nav { dir: NavDir::Down, extend: false });
    // Down from 240 → 256, saturates at max_addr = 255
    assert_eq!(state.selection.cursor, 255, "Down from last row should saturate at max_addr");
}

#[test]
fn test_extend_to_backwards() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(20));
    send(&mut state, &config, HexEditorMessage::ExtendTo(5));
    assert_eq!(state.selection.start(), 5);
    assert_eq!(state.selection.end(), 20);
    assert_eq!(state.selection.cursor, 5);
}
