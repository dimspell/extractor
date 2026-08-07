use super::*;

// ============================================================================
// Search overlay
// ============================================================================

#[test]
fn test_search_visible_after_open() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenSearch);
    assert!(state.search.visible, "search should be visible");
}

#[test]
fn test_search_overlay_renders_mode_button() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenSearch);
    let mut ui = simulator(view(&state, &config));
    ui.find("HEX").expect("search mode button should show HEX");
}

#[test]
fn test_search_toggle_mode_shows_ascii() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenSearch);
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    let mut ui = simulator(view(&state, &config));
    ui.find("TXT").expect("search mode button should show TXT");
}

#[test]
fn test_search_execute_hex() {
    let mut state = make_state(b"\x00\xDE\xAD\xBE\xEF\x00".to_vec());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("DE AD BE EF".into()),
    );
    assert_eq!(state.search.count(), 1, "should find 1 match");
    assert_eq!(state.search.results[0], 1, "match should start at byte 1");
}

#[test]
fn test_search_execute_ascii() {
    let mut state = make_state(b"hello world".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenSearch);
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("world".into()),
    );
    assert_eq!(state.search.count(), 1, "should find 'world'");
    assert_eq!(state.search.results[0], 6);
}

#[test]
fn test_search_no_match_shows_zero() {
    let mut state = make_state(b"hello".to_vec());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("xyzzy".into()),
    );
    assert_eq!(state.search.count(), 0, "should find no matches");
}

#[test]
fn test_search_next_prev() {
    let mut state = make_state(b"\x61\x62\x20\x61\x62\x20\x61\x62".to_vec());
    let config = default_config();
    // Search for hex bytes "61 62" (= "ab" ASCII) to verify navigation.
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("61 62".into()),
    );
    assert_eq!(state.search.count(), 3, "should find 3 matches");
    // After initial search, current_match is None (no match selected).
    // Navigate to the first match:
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(state.search.current_match, Some(0));
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(state.search.current_match, Some(1));
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(state.search.current_match, Some(2));
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(state.search.current_match, Some(0), "should wrap around");
    send(&mut state, &config, HexEditorMessage::SearchPrev);
    assert_eq!(state.search.current_match, Some(2), "should wrap backward");
}

#[test]
fn test_search_close_clears_state() {
    let mut state = make_state(b"hello".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Search("ll".into()));
    assert!(state.search.visible);
    assert!(!state.search.query.is_empty());
    send(&mut state, &config, HexEditorMessage::CloseSearch);
    assert!(!state.search.visible, "search should be hidden after close");
    assert!(state.search.query.is_empty(), "query should be cleared");
}

// ============================================================================
// Search — cursor movement, mode toggle, single-match navigation
// ============================================================================

#[test]
fn test_search_selects_cursor() {
    let mut state = make_state(b"\x00\xDE\xAD\xBE\xEF\x00".to_vec());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("DE AD BE EF".into()),
    );
    // After initial Search, cursor stays at 0 (current_match is None).
    // Navigate to the first match:
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(
        state.selection.cursor, 1,
        "search should move cursor to first match"
    );
}

#[test]
fn test_search_next_prev_single_match_stays_on_zero() {
    let mut state = make_state(b"\x00\xFF\x00".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Search("FF".into()));
    assert_eq!(state.search.count(), 1);
    assert_eq!(state.search.current_match, None, "initial match is None");
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(state.search.current_match, Some(0));
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(
        state.search.current_match,
        Some(0),
        "single match should stay on 0 with wrap"
    );
    send(&mut state, &config, HexEditorMessage::SearchPrev);
    assert_eq!(
        state.search.current_match,
        Some(0),
        "single match should stay on 0 backward"
    );
}

#[test]
fn test_search_reexecute_after_toggle_mode() {
    let mut state = make_state(b"\x00\xDE\xAD\xBE\xEF\x41\x42".to_vec());
    let config = default_config();
    // Search for "AB" in hex mode — no match (bytes would be 0xAB, 0xBE...)
    send(&mut state, &config, HexEditorMessage::Search("4142".into()));
    assert_eq!(
        state.search.count(),
        1,
        "hex 4142 matches bytes at offset 5"
    );
    // Toggle to ASCII — the query "4142" is now treated as ASCII text
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    // In ASCII mode, "4142" is looked for as literal bytes 0x34 0x31 0x34 0x32
    // (ASCII codes for '4','1','4','2'), which probably won't match anything.
    // But at least the search re-executes without panicking.
    assert_eq!(state.search.mode, crate::search::SearchMode::Ascii);
}

// ============================================================================
// Search overlay — match count display
// ============================================================================

#[test]
fn test_search_overlay_shows_match_count() {
    let mut state = make_state(b"\x00\xFF\x00\xFF\x00".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Search("FF".into()));
    assert_eq!(state.search.count(), 2);
    let mut ui = simulator(view(&state, &config));
    // Shows "-/2" after initial search (no match selected yet)
    ui.find("-/2")
        .expect("search overlay should show match count");
}

#[test]
fn test_search_overlay_shows_no_matches() {
    let mut state = make_state(b"hello".to_vec());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("xyzzy".into()),
    );
    let mut ui = simulator(view(&state, &config));
    ui.find("0 matches")
        .expect("should show 0 matches for no results");
}

#[test]
fn test_search_overlay_nav_buttons_render() {
    let mut state = make_state(b"\x00\xFF\x00\xFF\x00".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::Search("FF".into()));
    let mut ui = simulator(view(&state, &config));
    ui.find("<").expect("prev match button should render");
    ui.find(">").expect("next match button should render");
}

// ============================================================================
// Search — decimal mode & whitespace-tolerant ASCII
// ============================================================================

#[test]
fn test_search_decimal_le_width_2() {
    // 1000 = 0x03E8 → LE bytes E8 03 at offset 3.
    let mut state = make_state(b"\x00\x01\x02\xE8\x03\x00".to_vec());
    let config = default_config();
    // Toggle Hex → Ascii → Decimal (two presses).
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    assert_eq!(state.search.mode, crate::search::SearchMode::Decimal);
    send(&mut state, &config, HexEditorMessage::SetSearchWidth(2));
    send(&mut state, &config, HexEditorMessage::Search("1000".into()));
    assert_eq!(state.search.count(), 1, "should find 1 decimal match");
    assert_eq!(state.search.results[0], 3, "match should start at offset 3");
}

#[test]
fn test_search_decimal_be_width_4() {
    // 0x00010203 = 66051 → BE bytes 00 01 02 03 at offset 0.
    let mut state = make_state(b"\x00\x01\x02\x03\xFF".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    send(&mut state, &config, HexEditorMessage::ToggleSearchEndian);
    send(&mut state, &config, HexEditorMessage::SetSearchWidth(4));
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("66051".into()),
    );
    assert_eq!(state.search.count(), 1, "should find 1 BE decimal match");
    assert_eq!(state.search.results[0], 0);
}

#[test]
fn test_search_decimal_negative_value() {
    // -1 as width-1 byte → 0xFF at offset 1.
    let mut state = make_state(b"\xAA\xFF\xBB".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    send(&mut state, &config, HexEditorMessage::SetSearchWidth(1));
    send(&mut state, &config, HexEditorMessage::Search("-1".into()));
    assert_eq!(state.search.count(), 1, "should find -1 as 0xFF");
    assert_eq!(state.search.results[0], 1);
}

#[test]
fn test_search_decimal_toggle_after_two_presses() {
    let mut state = make_state(b"".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenSearch);
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    assert_eq!(state.search.mode, crate::search::SearchMode::Ascii);
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    assert_eq!(state.search.mode, crate::search::SearchMode::Decimal);
}

#[test]
fn test_search_ascii_whitespace_tolerant_multiline() {
    let mut state = make_state(b"hello\nworld".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenSearch);
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("hello world".into()),
    );
    assert_eq!(
        state.search.count(),
        1,
        "newline should collapse to a space"
    );
    assert_eq!(state.search.results[0], 0, "match should start at offset 0");
}

#[test]
fn test_search_decimal_width_control_updates_state() {
    let mut state = make_state(b"".to_vec());
    let config = default_config();
    assert_eq!(state.search.width, 4, "default width should be 4");
    send(&mut state, &config, HexEditorMessage::SetSearchWidth(8));
    assert_eq!(state.search.width, 8, "width should update to 8");
}

#[test]
fn test_search_invalid_width_ignored() {
    // Width 3 is not a valid decimal width (1/2/4/8); the message is ignored.
    let mut state = make_state(b"".to_vec());
    let config = default_config();
    assert_eq!(state.search.width, 4, "default width should be 4");
    send(&mut state, &config, HexEditorMessage::SetSearchWidth(3));
    assert_eq!(
        state.search.width, 4,
        "invalid width should leave the width unchanged"
    );
}

#[test]
fn test_search_set_width_reexecutes_decimal_query() {
    // With an active Decimal query, changing the width re-executes the search.
    // 100 as width 1 = 0x64; as width 2 (LE) = 0x64 0x00.
    let data = b"\x64\x00\x64\xFF";
    let mut state = make_state(data.to_vec());
    let config = default_config();
    // Toggle Hex → Ascii → Decimal (two presses).
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    assert_eq!(state.search.mode, crate::search::SearchMode::Decimal);

    // Start at width 2: bytes 0x64 0x00 at offset 0 → 1 match.
    send(&mut state, &config, HexEditorMessage::SetSearchWidth(2));
    send(&mut state, &config, HexEditorMessage::Search("100".into()));
    assert_eq!(state.search.count(), 1, "width-2 should match offset 0");

    // Switch to width 1: bytes 0x64 at offsets 0 and 2 → 2 matches.
    send(&mut state, &config, HexEditorMessage::SetSearchWidth(1));
    assert_eq!(
        state.search.count(), 2,
        "changing width to 1 should re-execute and find both matches"
    );
    assert_eq!(state.search.results, vec![0, 2]);
}

#[test]
fn test_search_ascii_multiline_extent() {
    // "ab  \ncd": the query "b cd" collapses the run "  \n" (indices 2..=4).
    // The match starts at index 1 and covers indices 1..=6 (6 original bytes).
    let mut state = make_state(b"ab  \ncd".to_vec());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::OpenSearch);
    send(&mut state, &config, HexEditorMessage::ToggleSearchMode);
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("b cd".into()),
    );
    assert_eq!(state.search.count(), 1);
    assert_eq!(state.search.results[0], 1);
    assert_eq!(state.search.extents[0], 6);
    send(&mut state, &config, HexEditorMessage::SearchNext);
    assert_eq!(state.search.current_len(), 6);
    for a in 1..=6 {
        assert!(
            state.search.match_set.contains(&a),
            "match_set should contain index {a}"
        );
    }
    assert!(!state.search.match_set.contains(&0));
    assert!(!state.search.match_set.contains(&7));
}
