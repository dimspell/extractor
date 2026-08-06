use super::*;
use crate::selection::Selection;

// ============================================================================
// Extend file — dialog open/close, drafts, commit, address-drift bookkeeping
// ============================================================================

/// Helper: park the cursor at `addr`, open the dialog, fill in the drafts and
/// commit in one shot.
fn commit_extend(
    state: &mut HexEditorState,
    config: &HexEditorConfig,
    addr: u64,
    count: &str,
    pattern: &str,
) {
    state.selection = Selection::single(addr);
    send(state, config, HexEditorMessage::BeginExtend);
    send(
        state,
        config,
        HexEditorMessage::SetExtendCount(count.to_string()),
    );
    send(
        state,
        config,
        HexEditorMessage::SetExtendPattern(pattern.to_string()),
    );
    send(state, config, HexEditorMessage::CommitExtend);
}

#[test]
fn begin_extend_opens_dialog() {
    let mut state = make_state(vec![1, 2, 3]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginExtend);
    assert!(state.extend_dialog.is_some(), "dialog should open");
    let dlg = state.extend_dialog.as_ref().unwrap();
    assert_eq!(dlg.count_draft, "", "count draft starts empty");
    assert_eq!(dlg.pattern_draft, "00", "pattern defaults to zero-fill");
    assert!(dlg.error.is_none());
}

#[test]
fn begin_extend_empty_file_shows_status() {
    let mut state = make_state(vec![]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginExtend);
    assert!(state.extend_dialog.is_none(), "dialog should stay closed");
    assert_eq!(state.status_msg, "Cannot extend an empty file");
}

#[test]
fn begin_extend_does_not_move_cursor() {
    let mut state = make_state((0..16).collect());
    let config = default_config();
    // Right-click sets only context_menu_addr, not the cursor.
    send(&mut state, &config, HexEditorMessage::RightClickAt(9));
    let before = state.selection.cursor;
    send(&mut state, &config, HexEditorMessage::BeginExtend);
    assert_eq!(
        state.selection.cursor, before,
        "opening the dialog must not move the cursor"
    );
    // Dismissing the dialog leaves the cursor untouched (no parking side effect).
    send(&mut state, &config, HexEditorMessage::CloseExtend);
    assert_eq!(state.selection.cursor, before);
}

#[test]
fn commit_extend_inserts_at_right_clicked_byte() {
    let mut state = make_state(vec![1, 2, 3, 4, 5]);
    let config = default_config();
    // Right-click byte 2 (cursor stays at 0 from make_state).
    send(&mut state, &config, HexEditorMessage::RightClickAt(2));
    send(&mut state, &config, HexEditorMessage::BeginExtend);
    send(
        &mut state,
        &config,
        HexEditorMessage::SetExtendCount("1".into()),
    );
    send(
        &mut state,
        &config,
        HexEditorMessage::SetExtendPattern("FF".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitExtend);
    assert_eq!(state.provider.as_slice(), &[1, 2, 0xFF, 3, 4, 5]);
    // The cursor parks on the inserted byte at commit time.
    assert_eq!(state.selection.cursor, 2);
}

#[test]
fn commit_extend_rejects_clicked_past_eof() {
    let mut state = make_state(vec![1, 2, 3]);
    let config = default_config();
    // Diff-pane right-clicks can report addresses past baseline EOF
    // (max_addr = 2 here) — reject instead of clamping to max_addr.
    send(&mut state, &config, HexEditorMessage::RightClickAt(5));
    send(&mut state, &config, HexEditorMessage::BeginExtend);
    send(
        &mut state,
        &config,
        HexEditorMessage::SetExtendCount("2".into()),
    );
    send(
        &mut state,
        &config,
        HexEditorMessage::SetExtendPattern("FF".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitExtend);
    assert_eq!(state.status_msg, "Cannot extend: clicked past end of file");
    assert_eq!(state.provider.as_slice(), &[1, 2, 3], "nothing inserted");
    assert!(state.extend_dialog.is_some(), "dialog stays open");
}

#[test]
fn set_drafts_update_dialog_and_clear_error() {
    let mut state = make_state(vec![1, 2, 3]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginExtend);
    state.extend_dialog.as_mut().unwrap().error = Some("boom".into());
    send(
        &mut state,
        &config,
        HexEditorMessage::SetExtendCount("8".into()),
    );
    send(
        &mut state,
        &config,
        HexEditorMessage::SetExtendPattern("AA".into()),
    );
    let dlg = state.extend_dialog.as_ref().unwrap();
    assert_eq!(dlg.count_draft, "8");
    assert_eq!(dlg.pattern_draft, "AA");
    assert!(dlg.error.is_none(), "editing should clear the error");
}

#[test]
fn commit_extend_inserts_mid_file() {
    let mut state = make_state(vec![1, 2, 3, 4, 5]);
    let config = default_config();
    commit_extend(&mut state, &config, 2, "3", "FF");
    assert_eq!(
        state.provider.as_slice(),
        &[1, 2, 0xFF, 0xFF, 0xFF, 3, 4, 5]
    );
    assert!(state.extend_dialog.is_none(), "dialog closes on success");
    assert_eq!(state.provider.len(), 8);
}

#[test]
fn commit_extend_inserts_at_zero() {
    let mut state = make_state(vec![1, 2, 3]);
    let config = default_config();
    commit_extend(&mut state, &config, 0, "2", "AA");
    assert_eq!(state.provider.as_slice(), &[0xAA, 0xAA, 1, 2, 3]);
}

#[test]
fn commit_extend_appends_at_eof() {
    let mut state = make_state(vec![1, 2, 3, 4, 5]);
    let config = default_config();
    // addr == len is a valid append (no shift).
    commit_extend(&mut state, &config, 5, "2", "AA");
    assert_eq!(state.provider.as_slice(), &[1, 2, 3, 4, 5, 0xAA, 0xAA]);
}

#[test]
fn commit_extend_cursor_past_eof_shows_status() {
    let mut state = make_state(vec![1, 2, 3]);
    let config = default_config();
    state.selection = Selection::single(10);
    send(&mut state, &config, HexEditorMessage::BeginExtend);
    send(
        &mut state,
        &config,
        HexEditorMessage::SetExtendCount("2".into()),
    );
    send(
        &mut state,
        &config,
        HexEditorMessage::SetExtendPattern("FF".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitExtend);
    assert_eq!(
        state.status_msg,
        "Cannot extend: cursor is past end of file"
    );
    assert_eq!(state.provider.as_slice(), &[1, 2, 3], "nothing inserted");
    assert!(state.extend_dialog.is_some(), "dialog stays open");
}

#[test]
fn commit_extend_repeats_pattern_across_count() {
    let mut state = make_state(vec![0; 8]);
    let config = default_config();
    commit_extend(&mut state, &config, 0, "7", "01 02");
    assert_eq!(
        state.provider.as_slice(),
        &[1, 2, 1, 2, 1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn commit_extend_truncates_long_pattern_to_count() {
    let mut state = make_state(vec![0, 0, 0]);
    let config = default_config();
    commit_extend(&mut state, &config, 0, "2", "AA BB CC");
    assert_eq!(state.provider.as_slice(), &[0xAA, 0xBB, 0, 0, 0]);
}

#[test]
fn commit_extend_rebases_patterns() {
    let mut state = make_state(vec![0; 20]);
    let config = default_config();
    state.add_pattern(0, 1); // before insert point — unchanged
    state.add_pattern(3, 5); // at insert point — shifts forward
    state.add_pattern(7, 9); // after insert point — shifts forward
    commit_extend(&mut state, &config, 3, "2", "FF");
    assert_eq!(state.patterns[0].start, 0);
    assert_eq!(state.patterns[0].end, 1);
    assert_eq!(state.patterns[1].start, 5);
    assert_eq!(state.patterns[1].end, 7);
    assert_eq!(state.patterns[2].start, 9);
    assert_eq!(state.patterns[2].end, 11);
}

#[test]
fn commit_extend_rebases_straddling_pattern() {
    let mut state = make_state(vec![0; 20]);
    let config = default_config();
    state.add_pattern(2, 4); // straddles insert point (2 < 3 <= 4) — start kept, tail absorbs
    state.add_pattern(7, 9); // fully after insert point — both shift
    state.add_pattern(0, 1); // fully before insert point — untouched
    commit_extend(&mut state, &config, 3, "2", "FF");
    assert_eq!(state.patterns[0].start, 2, "straddling start unchanged");
    assert_eq!(
        state.patterns[0].end, 6,
        "straddling tail absorbs inserted bytes"
    );
    assert_eq!(state.patterns[1].start, 9);
    assert_eq!(state.patterns[1].end, 11);
    assert_eq!(state.patterns[2].start, 0);
    assert_eq!(state.patterns[2].end, 1);
}

#[test]
fn commit_extend_clears_search_results() {
    let mut state = make_state(vec![0x10, 0x20, 0x10, 0x20, 0x10]);
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::Search("10 20".into()),
    );
    assert!(state.search.has_results(), "search should have matches");
    assert!(!state.search.match_set.is_empty());
    assert!(state.search.is_visible());
    commit_extend(&mut state, &config, 0, "2", "FF");
    assert!(state.search.results.is_empty(), "results cleared");
    assert!(state.search.match_set.is_empty(), "match set cleared");
    assert!(state.search.current_match.is_none());
    assert_eq!(state.search.query_len, 0);
    assert!(
        state.search.is_visible(),
        "extend must not force-close the search overlay"
    );
}

#[test]
fn commit_extend_sets_selection_to_inserted_range() {
    let mut state = make_state(vec![1, 2, 3, 4, 5]);
    let config = default_config();
    commit_extend(&mut state, &config, 2, "3", "00");
    assert_eq!(state.selection.anchor, 2);
    assert_eq!(state.selection.cursor, 4);
}

#[test]
fn commit_extend_invalid_count_keeps_dialog_and_sets_error() {
    let mut state = make_state(vec![1, 2, 3]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginExtend);
    send(
        &mut state,
        &config,
        HexEditorMessage::SetExtendCount("0".into()),
    );
    send(
        &mut state,
        &config,
        HexEditorMessage::SetExtendPattern("FF".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitExtend);
    assert!(state.extend_dialog.is_some(), "dialog stays open");
    let dlg = state.extend_dialog.as_ref().unwrap();
    assert_eq!(dlg.error.as_deref(), Some("Count must be at least 1 byte"));
    assert_eq!(state.provider.as_slice(), &[1, 2, 3], "nothing inserted");
}

#[test]
fn commit_extend_invalid_pattern_keeps_dialog_and_sets_error() {
    let mut state = make_state(vec![1, 2, 3]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginExtend);
    send(
        &mut state,
        &config,
        HexEditorMessage::SetExtendCount("4".into()),
    );
    send(
        &mut state,
        &config,
        HexEditorMessage::SetExtendPattern("XYZ".into()),
    );
    send(&mut state, &config, HexEditorMessage::CommitExtend);
    assert!(state.extend_dialog.is_some(), "dialog stays open");
    let dlg = state.extend_dialog.as_ref().unwrap();
    assert_eq!(dlg.error.as_deref(), Some("Invalid hex input: \"XYZ\""));
    assert_eq!(state.provider.as_slice(), &[1, 2, 3], "nothing inserted");
}

#[test]
fn close_extend_dismisses() {
    let mut state = make_state(vec![1, 2, 3]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginExtend);
    assert!(state.extend_dialog.is_some());
    send(&mut state, &config, HexEditorMessage::CloseExtend);
    assert!(state.extend_dialog.is_none());
    assert_eq!(state.provider.as_slice(), &[1, 2, 3], "no bytes inserted");
}

#[test]
fn commit_extend_marks_tail_vanilla_diff_and_dirty() {
    let mut state = make_state(vec![1, 2, 3, 4, 5]);
    state.vanilla = Some(vec![1, 2, 3, 4, 5]);
    let config = default_config();
    commit_extend(&mut state, &config, 2, "2", "00");
    // Inserted bytes (2,3) differ from vanilla, and the shifted tail (old
    // 3,4,5 now at 4,5,6) differs too — the whole tail is marked.
    assert_eq!(
        state.vanilla_diff,
        (2..7).collect::<BTreeSet<u64>>(),
        "tail at/after the insert point must differ from vanilla"
    );
    // The dirty set covers the inserted offsets.
    assert!(state.provider.dirty().contains(&2));
    assert!(state.provider.dirty().contains(&3));
}

#[test]
fn commit_extend_cancels_active_edit_mode() {
    let mut state = make_state(vec![1, 2, 3, 4, 5]);
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::BeginEdit(3));
    assert!(state.edit_mode.is_some(), "edit mode active before extend");
    commit_extend(&mut state, &config, 2, "2", "00");
    assert!(
        state.edit_mode.is_none(),
        "extend must cancel in-matrix edit mode (the draft addr has shifted)"
    );
}

#[test]
fn commit_extend_twice_accumulates() {
    let mut state = make_state(vec![1, 2, 3, 4, 5]);
    let config = default_config();
    commit_extend(&mut state, &config, 2, "2", "AA");
    assert_eq!(state.provider.as_slice(), &[1, 2, 0xAA, 0xAA, 3, 4, 5]);
    assert_eq!(
        state.selection.cursor, 3,
        "selection covers first inserted range"
    );
    // Second extend at the same addr inserts before the previously inserted
    // bytes — they shift forward along with the tail.
    commit_extend(&mut state, &config, 2, "1", "BB");
    assert_eq!(
        state.provider.as_slice(),
        &[1, 2, 0xBB, 0xAA, 0xAA, 3, 4, 5]
    );
    assert_eq!(state.selection.anchor, 2);
    assert_eq!(
        state.selection.cursor, 2,
        "selection covers second inserted range"
    );
}
