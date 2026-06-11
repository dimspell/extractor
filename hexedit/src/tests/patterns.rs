use super::*;

// ============================================================================
// Patterns
// ============================================================================

#[test]
fn test_create_pattern_needs_range() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    // Single selection (no range)
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    assert_eq!(state.patterns.len(), 0, "should not create pattern");
    assert!(
        state.status_msg.contains("Select a range"),
        "should show hint to select range"
    );
}

#[test]
fn test_create_pattern_from_range() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(20));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    assert_eq!(state.patterns.len(), 1, "should create one pattern");
    assert_eq!(state.patterns[0].start, 10);
    assert_eq!(state.patterns[0].end, 20);
    assert!(
        state.status_msg.contains("Pattern created"),
        "should confirm creation"
    );
}

#[test]
fn test_remove_pattern_by_id() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(5));
    send(&mut state, &config, HexEditorMessage::ExtendTo(15));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    let id = state.patterns[0].id;
    send(&mut state, &config, HexEditorMessage::RemovePattern(id));
    assert_eq!(state.patterns.len(), 0);
    assert!(state.pattern_by_addr.is_empty());
}

#[test]
fn test_remove_pattern_at_address() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(5));
    send(&mut state, &config, HexEditorMessage::ExtendTo(15));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    // Remove the pattern by clicking on an address within it
    send(&mut state, &config, HexEditorMessage::RemovePatternAt(10));
    assert_eq!(state.patterns.len(), 0, "pattern should be removed");
}

#[test]
fn test_clear_all_patterns() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(0));
    send(&mut state, &config, HexEditorMessage::ExtendTo(10));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    send(&mut state, &config, HexEditorMessage::SelectAt(20));
    send(&mut state, &config, HexEditorMessage::ExtendTo(30));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    assert_eq!(state.patterns.len(), 2);
    send(&mut state, &config, HexEditorMessage::ClearAllPatterns);
    assert_eq!(state.patterns.len(), 0);
    assert!(state.status_msg.contains("All patterns cleared"));
}

#[test]
fn test_pattern_list_toggle() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    assert!(!state.show_pattern_list, "pattern list hidden by default");
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    assert!(state.show_pattern_list, "pattern list now visible");
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    assert!(!state.show_pattern_list, "pattern list hidden again");
}

#[test]
fn test_pattern_list_empty_view() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    let mut ui = simulator(view(&state, &config));
    ui.find("Patterns (0)")
        .expect("pattern list header should show 0");
    ui.find("No patterns defined")
        .expect("empty pattern message");
}

#[test]
fn test_pattern_list_with_patterns() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(20));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    send(&mut state, &config, HexEditorMessage::TogglePatternList);
    let mut ui = simulator(view(&state, &config));
    ui.find("Patterns (1)")
        .expect("pattern list header should show 1");
}

#[test]
fn test_navigate_to_pattern_moves_cursor() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::SelectAt(10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(20));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    let id = state.patterns[0].id;
    // Navigate away
    send(&mut state, &config, HexEditorMessage::SelectAt(0));
    assert_eq!(state.selection.cursor, 0);
    // Navigate to pattern
    send(&mut state, &config, HexEditorMessage::NavigateToPattern(id));
    assert_eq!(state.selection.cursor, 10, "cursor should go to pattern start");
}

#[test]
fn test_context_menu_addr_set_on_right_click() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    assert!(state.context_menu_addr.is_none());
    send(&mut state, &config, HexEditorMessage::RightClickAt(42));
    assert_eq!(state.context_menu_addr, Some(42));
}

// ============================================================================
// Patterns — extended
// ============================================================================

#[test]
fn test_pattern_context_menu_on_existing_pattern() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    // Create a pattern
    send(&mut state, &config, HexEditorMessage::SelectAt(10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(20));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    // Right-click on address within pattern
    send(&mut state, &config, HexEditorMessage::RightClickAt(15));
    assert_eq!(state.context_menu_addr, Some(15));
    // The RightClickAt message itself doesn't check patterns — that's done in view.
    // We just verify the address was stored.
}

#[test]
fn test_remove_pattern_at_nonexistent_address_is_noop() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::RemovePatternAt(999));
    assert_eq!(state.patterns.len(), 0);
}

#[test]
fn test_remove_pattern_at_context_menu_removes_pattern() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    // Create a pattern
    send(&mut state, &config, HexEditorMessage::SelectAt(10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(20));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    assert_eq!(state.patterns.len(), 1);
    // Set up the context-menu address
    send(&mut state, &config, HexEditorMessage::RightClickAt(15));
    assert_eq!(state.context_menu_addr, Some(15));
    // Remove through context-menu path
    send(&mut state, &config, HexEditorMessage::RemovePatternAtContextMenu);
    assert_eq!(state.patterns.len(), 0, "pattern should be removed via context menu");
}

#[test]
fn test_remove_pattern_at_context_menu_clears_addr() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::RightClickAt(42));
    assert_eq!(state.context_menu_addr, Some(42));
    // Remove at a non-pattern address (harmless no-op)
    send(&mut state, &config, HexEditorMessage::RemovePatternAtContextMenu);
    assert!(state.context_menu_addr.is_none(), "context_menu_addr should be cleared after handling");
}

#[test]
fn test_remove_pattern_at_context_menu_noop_when_no_addr() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    // context_menu_addr is None, no right-click happened
    assert!(state.context_menu_addr.is_none());
    // Should not panic or crash
    send(&mut state, &config, HexEditorMessage::RemovePatternAtContextMenu);
    assert_eq!(state.patterns.len(), 0);
}
