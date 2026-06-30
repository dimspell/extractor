use super::*;
use crate::RepeatPatternDialog;
use crate::domain::pattern::{Pattern, RepeatedPatternGroup};

// ============================================================================
// Optimized helpers — direct state construction (no message pipeline)
// ============================================================================

/// Directly construct a pattern without going through the message pipeline.
/// Caller must call `rebuild_lookups()` (or a mutation that triggers it) before
/// accessing `pattern_by_addr` or `row_annotations`.
fn seed_pattern(state: &mut HexEditorState, start: u64, end: u64) -> usize {
    let id = state.next_pattern_id;
    state.next_pattern_id += 1;
    let color_idx = (state.patterns.len() % 16) as u8;
    state.patterns.push(Pattern::new(id, start, end, color_idx));
    id
}

/// Directly construct a group with repeated patterns and auto-annotations.
/// Clamps block_end at `state.max_addr()` and stops when `block_start > max`.
fn seed_group(
    state: &mut HexEditorState,
    label: &str,
    count: u32,
    base: u64,
    size: u64,
) -> usize {
    let gid = state.next_group_id;
    state.next_group_id += 1;
    let color_idx = (state.groups.len() % 16) as u8;
    state
        .groups
        .push(RepeatedPatternGroup::new(gid, label.to_string(), color_idx));

    let max_addr = state.max_addr();
    for i in 0..count {
        let block_start = base + i as u64 * size;
        if block_start > max_addr {
            break;
        }
        let block_end = (block_start + size - 1).min(max_addr);
        let pid = state.next_pattern_id;
        state.next_pattern_id += 1;
        let mut pat = Pattern::grouped(pid, block_start, block_end, color_idx, gid);
        pat.annotation = Some(format!("{}[{}]", label, i));
        state.patterns.push(pat);
    }
    gid
}

/// Rebuild both `pattern_by_addr` and `row_annotations` after direct mutations.
fn rebuild_lookups(state: &mut HexEditorState) {
    state.rebuild_pattern_lookup();
    state.recompute_row_annotations();
}

// ============================================================================
// Repeated pattern groups — creation, annotation, rename, collapse, remove
// ============================================================================

// ---------------------------------------------------------------------------
// CommitRepeatedPattern — annotation prefill
// ---------------------------------------------------------------------------

#[test]
fn commit_repeated_pattern_prefills_annotations() {
    let mut state = make_state((0..128).collect());
    let config = default_config();

    state.repeat_pattern = Some(RepeatPatternDialog {
        block_start: 0x10,
        block_size: 0x20,
        draft: "3".to_string(),
        label_draft: "Monster".to_string(),
        error: None,
    });
    send(&mut state, &config, HexEditorMessage::CommitRepeatedPattern);

    assert_eq!(state.patterns.len(), 3, "should create 3 patterns");
    assert_eq!(state.groups.len(), 1, "should create 1 group");
    assert_eq!(state.groups[0].label, "Monster");

    assert_eq!(state.patterns[0].annotation.as_deref(), Some("Monster[0]"));
    assert_eq!(state.patterns[1].annotation.as_deref(), Some("Monster[1]"));
    assert_eq!(state.patterns[2].annotation.as_deref(), Some("Monster[2]"));

    let gid = state.groups[0].id;
    assert!(state.patterns.iter().all(|p| p.group_id == Some(gid)));
}

#[test]
fn commit_repeated_pattern_empty_or_whitespace_label_uses_default() {
    for label_draft in ["", "   "] {
        let mut state = make_state((0..64).collect());
        let config = default_config();
        state.repeat_pattern = Some(RepeatPatternDialog {
            block_start: 0x00,
            block_size: 0x10,
            draft: "1".to_string(),
            label_draft: label_draft.to_string(),
            error: None,
        });
        send(&mut state, &config, HexEditorMessage::CommitRepeatedPattern);

        assert_eq!(state.groups.len(), 1);
        assert_eq!(state.groups[0].label, "Unnamed group");
        assert_eq!(
            state.patterns[0].annotation.as_deref(),
            Some("Unnamed group[0]")
        );
    }
}

#[test]
fn commit_repeated_pattern_clamps_at_max_addr() {
    let mut state = make_state((0..50).collect());
    let config = default_config();

    state.repeat_pattern = Some(RepeatPatternDialog {
        block_start: 0x00,
        block_size: 0x20,
        draft: "5".to_string(),
        label_draft: "Clamped".to_string(),
        error: None,
    });
    send(&mut state, &config, HexEditorMessage::CommitRepeatedPattern);

    assert_eq!(state.patterns.len(), 2, "only 2 blocks fit in 50 bytes");
    assert_eq!(state.patterns[0].end, 31);
    assert_eq!(
        state.patterns[1].end, 49,
        "last block should clamp at max_addr"
    );
}

#[test]
fn commit_repeated_pattern_invalid_count_shows_error() {
    let mut state = make_state((0..64).collect());
    let config = default_config();

    state.repeat_pattern = Some(RepeatPatternDialog {
        block_start: 0x00,
        block_size: 0x10,
        draft: "not_a_number".to_string(),
        label_draft: "Monster".to_string(),
        error: None,
    });
    send(&mut state, &config, HexEditorMessage::CommitRepeatedPattern);

    assert_eq!(state.patterns.len(), 0, "no patterns with invalid count");
    assert!(state.repeat_pattern.is_some(), "dialog should remain open");
    assert!(
        state.repeat_pattern.as_ref().unwrap().error.is_some(),
        "dialog should have error message"
    );
}

// ---------------------------------------------------------------------------
// SetPatternAnnotation — set, clear, recompute
// ---------------------------------------------------------------------------

#[test]
fn set_annotation_creates_annotation() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_pattern(&mut state, 10, 20);
    rebuild_lookups(&mut state);
    let id = state.patterns[0].id;

    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(id, "my note".to_string()),
    );

    assert_eq!(state.patterns[0].annotation.as_deref(), Some("my note"));
}

#[test]
fn set_annotation_empty_clears_annotation() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_pattern(&mut state, 10, 20);
    rebuild_lookups(&mut state);
    let id = state.patterns[0].id;

    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(id, "note".to_string()),
    );
    assert!(state.patterns[0].annotation.is_some());

    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(id, "".to_string()),
    );
    assert!(state.patterns[0].annotation.is_none());
}

#[test]
fn set_annotation_whitespace_clears_annotation() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_pattern(&mut state, 10, 20);
    rebuild_lookups(&mut state);
    let id = state.patterns[0].id;

    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(id, "   ".to_string()),
    );
    assert!(state.patterns[0].annotation.is_none());
}

#[test]
fn set_annotation_recomputes_row_annotations() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_pattern(&mut state, 10, 20);
    rebuild_lookups(&mut state);
    let id = state.patterns[0].id;

    assert!(
        state.row_annotations.is_empty(),
        "row_annotations should be empty before annotation (seed_pattern sets none)"
    );

    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(id, "tag".to_string()),
    );
    assert!(
        !state.row_annotations.is_empty(),
        "row_annotations should be recomputed after setting annotation"
    );
    // bpr=16 → rows at row_start 0 and 16
    assert!(state.row_annotations.contains_key(&0));
    assert!(state.row_annotations.contains_key(&16));

    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(id, "".to_string()),
    );
    assert!(state.row_annotations.is_empty());
}

#[test]
fn set_annotation_nonexistent_id_is_noop() {
    let mut state = make_state((0..64).collect());
    let config = default_config();

    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(999, "ghost".to_string()),
    );
    assert!(state.patterns.is_empty());
}

// ---------------------------------------------------------------------------
// CommitRenameGroup — annotation auto-update
// ---------------------------------------------------------------------------

#[test]
fn rename_group_updates_child_annotations() {
    let mut state = make_state((0..128).collect());
    let config = default_config();
    seed_group(&mut state, "Monster", 3, 0x10, 0x20);
    rebuild_lookups(&mut state);

    let gid = state.groups[0].id;

    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
    assert_eq!(state.renaming_group, Some(gid));
    assert_eq!(state.renaming_group_draft, "Monster");

    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("Enemy".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    assert_eq!(state.groups[0].label, "Enemy");
    assert!(state.renaming_group.is_none());
    assert!(state.renaming_group_draft.is_empty());

    assert_eq!(state.patterns[0].annotation.as_deref(), Some("Enemy[0]"));
    assert_eq!(state.patterns[1].annotation.as_deref(), Some("Enemy[1]"));
    assert_eq!(state.patterns[2].annotation.as_deref(), Some("Enemy[2]"));
    assert!(state.status_msg.contains("Enemy"));
}

#[test]
fn rename_group_to_empty_uses_unnamed_and_updates_annotations() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_group(&mut state, "Monster", 2, 0x10, 0x10);
    rebuild_lookups(&mut state);

    let gid = state.groups[0].id;
    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    assert_eq!(state.groups[0].label, "Unnamed group");
    assert_eq!(
        state.patterns[0].annotation.as_deref(),
        Some("Unnamed group[0]")
    );
    assert_eq!(
        state.patterns[1].annotation.as_deref(),
        Some("Unnamed group[1]")
    );
}

#[test]
fn rename_group_preserves_manual_annotations() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_group(&mut state, "Monster", 2, 0x10, 0x10);
    rebuild_lookups(&mut state);

    let pid2 = state.patterns[1].id;
    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(pid2, "custom note".to_string()),
    );

    let gid = state.groups[0].id;
    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("Enemy".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    assert_eq!(state.patterns[0].annotation.as_deref(), Some("Enemy[0]"));
    assert_eq!(state.patterns[1].annotation.as_deref(), Some("custom note"));
}

#[test]
fn rename_group_substring_rename_works_both_directions() {
    for (old_label, new_label) in [("Mon", "Monster"), ("Monster", "Mon")] {
        let mut state = make_state((0..64).collect());
        let config = default_config();
        seed_group(&mut state, old_label, 2, 0x10, 0x10);
        rebuild_lookups(&mut state);

        let gid = state.groups[0].id;
        send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
        send(
            &mut state,
            &config,
            HexEditorMessage::SetRenameGroupDraft(new_label.to_string()),
        );
        send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

        assert_eq!(state.groups[0].label, new_label);
        assert_eq!(
            state.patterns[0].annotation.as_deref(),
            Some(&format!("{new_label}[0]")[..])
        );
        assert_eq!(
            state.patterns[1].annotation.as_deref(),
            Some(&format!("{new_label}[1]")[..])
        );
    }
}

#[test]
fn rename_group_other_groups_unaffected() {
    let mut state = make_state((0..128).collect());
    let config = default_config();

    seed_group(&mut state, "GroupA", 2, 0x10, 0x10);
    let gid_a = state.groups[0].id;
    seed_group(&mut state, "GroupB", 2, 0x40, 0x10);
    let gid_b = state.groups[1].id;
    rebuild_lookups(&mut state);

    send(
        &mut state,
        &config,
        HexEditorMessage::BeginRenameGroup(gid_a),
    );
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("RenamedA".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    assert_eq!(state.groups[1].label, "GroupB");
    let group_b_patterns: Vec<_> = state
        .patterns
        .iter()
        .filter(|p| p.group_id == Some(gid_b))
        .collect();
    assert_eq!(group_b_patterns[0].annotation.as_deref(), Some("GroupB[0]"));
    assert_eq!(group_b_patterns[1].annotation.as_deref(), Some("GroupB[1]"));
}

#[test]
fn rename_group_ungrouped_unaffected() {
    let mut state = make_state((0..192).collect());
    let config = default_config();

    seed_group(&mut state, "GroupA", 2, 0x10, 0x10);
    let gid = state.groups[0].id;
    seed_pattern(&mut state, 0x80, 0x8F);
    rebuild_lookups(&mut state);
    let ungrouped_id = state.patterns[2].id;

    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(ungrouped_id, "standalone".to_string()),
    );

    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("Renamed".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    assert_eq!(state.patterns[2].annotation.as_deref(), Some("standalone"));
}

#[test]
fn rename_group_preserves_non_auto_annotations() {
    for (initial_annotation, desc) in [
        ("Monster[foobar]", "non-digit in brackets"),
        ("custom annotation", "different prefix"),
    ] {
        let mut state = make_state((0..64).collect());
        let config = default_config();
        seed_group(&mut state, "Monster", 1, 0x10, 0x10);
        rebuild_lookups(&mut state);

        let gid = state.groups[0].id;
        let pid = state.patterns[0].id;
        send(
            &mut state,
            &config,
            HexEditorMessage::SetPatternAnnotation(pid, initial_annotation.to_string()),
        );

        send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
        send(
            &mut state,
            &config,
            HexEditorMessage::SetRenameGroupDraft("Enemy".to_string()),
        );
        send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

        assert_eq!(
            state.patterns[0].annotation.as_deref(),
            Some(initial_annotation),
            "{desc}"
        );
    }
}

#[test]
fn rename_group_cancel_does_not_change_anything() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_group(&mut state, "Monster", 2, 0x10, 0x10);
    rebuild_lookups(&mut state);

    let gid = state.groups[0].id;

    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("Enemy".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CancelRenameGroup);

    assert_eq!(state.groups[0].label, "Monster");
    assert!(state.renaming_group.is_none());
    assert!(state.renaming_group_draft.is_empty());

    assert_eq!(state.patterns[0].annotation.as_deref(), Some("Monster[0]"));
    assert_eq!(state.patterns[1].annotation.as_deref(), Some("Monster[1]"));
}

#[test]
fn rename_group_whitespace_label_trims_and_uses_default() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_group(&mut state, "Monster", 2, 0x10, 0x10);
    rebuild_lookups(&mut state);

    let gid = state.groups[0].id;
    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("  ".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    assert_eq!(state.groups[0].label, "Unnamed group");
    assert_eq!(
        state.patterns[0].annotation.as_deref(),
        Some("Unnamed group[0]")
    );
    assert_eq!(
        state.patterns[1].annotation.as_deref(),
        Some("Unnamed group[1]")
    );
}

#[test]
fn rename_group_nonexistent_id_is_noop() {
    let mut state = make_state((0..64).collect());
    let config = default_config();

    state.renaming_group = Some(999);
    state.renaming_group_draft = "Ghost".to_string();

    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    assert!(state.groups.is_empty());
    assert!(state.patterns.is_empty());
}

// ---------------------------------------------------------------------------
// TogglePatternGroup — collapse/expand
// ---------------------------------------------------------------------------

#[test]
fn toggle_group_collapse_expand() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_group(&mut state, "Monster", 2, 0x10, 0x10);
    rebuild_lookups(&mut state);

    let gid = state.groups[0].id;

    assert!(!state.collapsed_groups.contains(&gid));

    send(
        &mut state,
        &config,
        HexEditorMessage::TogglePatternGroup(gid),
    );
    assert!(state.collapsed_groups.contains(&gid));

    send(
        &mut state,
        &config,
        HexEditorMessage::TogglePatternGroup(gid),
    );
    assert!(!state.collapsed_groups.contains(&gid));
}

#[test]
fn toggle_nonexistent_group_does_not_panic() {
    let mut state = make_state((0..64).collect());
    let config = default_config();

    send(
        &mut state,
        &config,
        HexEditorMessage::TogglePatternGroup(999),
    );
    assert!(state.patterns.is_empty());
}

// ---------------------------------------------------------------------------
// RemovePatternGroup — removes group and all children
// ---------------------------------------------------------------------------

#[test]
fn remove_group_removes_all_child_patterns() {
    let mut state = make_state((0..192).collect());
    let config = default_config();
    seed_group(&mut state, "Monster", 3, 0x10, 0x10);
    seed_pattern(&mut state, 0x80, 0x8F);
    rebuild_lookups(&mut state);

    let gid = state.groups[0].id;
    assert_eq!(state.patterns.len(), 4);
    assert_eq!(state.groups.len(), 1);

    send(
        &mut state,
        &config,
        HexEditorMessage::RemovePatternGroup(gid),
    );

    assert_eq!(state.patterns.len(), 1, "ungrouped pattern should remain");
    assert_eq!(state.groups.len(), 0, "group should be removed");
    assert!(!state.collapsed_groups.contains(&gid));
    assert!(
        state.status_msg.contains("Removed group and 3 pattern(s)"),
        "should report correct removed count: got {}",
        state.status_msg
    );
}

#[test]
fn remove_group_recomputes_lookups() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_group(&mut state, "Monster", 2, 0x10, 0x10);
    rebuild_lookups(&mut state);

    let gid = state.groups[0].id;
    assert!(!state.pattern_by_addr.is_empty());
    assert!(!state.row_annotations.is_empty());

    send(
        &mut state,
        &config,
        HexEditorMessage::RemovePatternGroup(gid),
    );

    assert!(state.pattern_by_addr.is_empty(), "pattern lookup should be rebuilt");
    assert!(state.row_annotations.is_empty(), "row annotations should be recomputed");
}

#[test]
fn remove_nonexistent_group_is_noop() {
    let mut state = make_state((0..64).collect());
    let config = default_config();

    send(
        &mut state,
        &config,
        HexEditorMessage::RemovePatternGroup(999),
    );
    assert!(state.groups.is_empty());
    assert!(state.patterns.is_empty());
}

// ---------------------------------------------------------------------------
// CycleGroupColor — updates all children
// ---------------------------------------------------------------------------

#[test]
fn cycle_group_color_updates_all_children() {
    let mut state = make_state((0..128).collect());
    let config = default_config();
    seed_group(&mut state, "Monster", 3, 0x10, 0x10);
    rebuild_lookups(&mut state);

    let gid = state.groups[0].id;
    let original_color = state.groups[0].color_idx;

    send(&mut state, &config, HexEditorMessage::CycleGroupColor(gid));

    let expected_color = (original_color + 1) % 16;
    assert_eq!(state.groups[0].color_idx, expected_color);

    for pat in &state.patterns {
        assert_eq!(
            pat.color_idx, expected_color,
            "pattern {} should have updated color",
            pat.id
        );
    }
}

#[test]
fn cycle_group_color_nonexistent_id_is_noop() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(&mut state, &config, HexEditorMessage::CycleGroupColor(999));
}

// ---------------------------------------------------------------------------
// CyclePatternColor — individual pattern only
// ---------------------------------------------------------------------------

#[test]
fn cycle_pattern_color() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_pattern(&mut state, 10, 20);
    rebuild_lookups(&mut state);
    let id = state.patterns[0].id;
    let original_color = state.patterns[0].color_idx;

    send(&mut state, &config, HexEditorMessage::CyclePatternColor(id));

    assert_eq!(state.patterns[0].color_idx, (original_color + 1) % 16);
}

#[test]
fn cycle_pattern_color_nonexistent_id_is_noop() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    send(
        &mut state,
        &config,
        HexEditorMessage::CyclePatternColor(999),
    );
    assert!(state.patterns.is_empty());
}

// ---------------------------------------------------------------------------
// SetRenameGroupDraft — updates draft
// ---------------------------------------------------------------------------

#[test]
fn set_rename_group_draft_updates_string() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_group(&mut state, "Group", 1, 0x10, 0x10);
    rebuild_lookups(&mut state);
    let gid = state.groups[0].id;

    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));

    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("NewName".to_string()),
    );
    assert_eq!(state.renaming_group_draft, "NewName");

    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("".to_string()),
    );
    assert_eq!(state.renaming_group_draft, "");
}

// ---------------------------------------------------------------------------
// CommitRepeatedPattern — edge cases
// ---------------------------------------------------------------------------

#[test]
fn commit_repeated_pattern_without_dialog_is_noop() {
    let mut state = make_state((0..64).collect());
    let config = default_config();

    send(&mut state, &config, HexEditorMessage::CommitRepeatedPattern);

    assert!(state.patterns.is_empty());
    assert!(state.groups.is_empty());
}

#[test]
fn commit_repeated_pattern_zero_count_does_nothing() {
    let mut state = make_state((0..64).collect());
    let config = default_config();

    state.repeat_pattern = Some(RepeatPatternDialog {
        block_start: 0x00,
        block_size: 0x10,
        draft: "0".to_string(),
        label_draft: "Empty".to_string(),
        error: None,
    });
    send(&mut state, &config, HexEditorMessage::CommitRepeatedPattern);

    assert_eq!(state.patterns.len(), 0, "zero count should create no patterns");
    assert_eq!(state.groups.len(), 0, "group should NOT be created for invalid count");
    assert!(state.repeat_pattern.is_some());
    assert!(state.repeat_pattern.unwrap().error.is_some());
}

// ---------------------------------------------------------------------------
// Orphan group auto-cleanup
// ---------------------------------------------------------------------------

#[test]
fn remove_last_pattern_in_group_removes_group() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_group(&mut state, "Monster", 1, 0x10, 0x10);
    rebuild_lookups(&mut state);

    assert_eq!(state.groups.len(), 1);
    let pid = state.patterns[0].id;

    send(&mut state, &config, HexEditorMessage::RemovePattern(pid));

    assert!(state.groups.is_empty(), "group should be removed when its last pattern is removed");
    assert!(state.patterns.is_empty(), "pattern should be gone");
}

#[test]
fn remove_some_but_not_all_patterns_keeps_group() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_group(&mut state, "Monster", 3, 0x10, 0x10);
    rebuild_lookups(&mut state);

    assert_eq!(state.groups.len(), 1);
    assert_eq!(state.patterns.len(), 3);
    let pid = state.patterns[0].id;

    send(&mut state, &config, HexEditorMessage::RemovePattern(pid));

    assert_eq!(state.groups.len(), 1, "group should persist when patterns remain");
    assert_eq!(state.patterns.len(), 2, "only one pattern removed");
    assert_eq!(state.groups[0].label, "Monster", "group label preserved");
}

#[test]
fn remove_all_patterns_from_multiple_groups_cleans_up() {
    let mut state = make_state((0..64).collect());
    let config = default_config();

    seed_group(&mut state, "GroupA", 1, 0x10, 0x10);
    let pid_a = state.patterns[0].id;
    seed_group(&mut state, "GroupB", 1, 0x30, 0x10);
    let pid_b = state.patterns[1].id;
    rebuild_lookups(&mut state);

    assert_eq!(state.groups.len(), 2);

    send(&mut state, &config, HexEditorMessage::RemovePattern(pid_a));
    assert_eq!(state.groups.len(), 1, "GroupA removed, GroupB stays");
    assert_eq!(state.groups[0].label, "GroupB");

    send(&mut state, &config, HexEditorMessage::RemovePattern(pid_b));
    assert!(state.groups.is_empty(), "all groups removed");
}

#[test]
fn orphan_cleanup_does_not_affect_patterns_not_in_groups() {
    let mut state = make_state((0..128).collect());
    let config = default_config();
    seed_group(&mut state, "Monster", 1, 0x10, 0x10);
    let gid = state.groups[0].id;

    // Create standalone pattern via message pipeline (tests that path too).
    // Set repeat_pattern = None to avoid state pollution from seed_group's
    // internal annotation logic.
    state.repeat_pattern = None;
    send(&mut state, &config, HexEditorMessage::SelectAt(0x50));
    send(&mut state, &config, HexEditorMessage::ExtendTo(0x5F));
    send(&mut state, &config, HexEditorMessage::CreatePattern);

    assert_eq!(state.patterns.len(), 2, "grouped + standalone pattern");
    let grouped_pid = state
        .patterns
        .iter()
        .find(|p| p.group_id == Some(gid))
        .unwrap()
        .id;

    send(
        &mut state,
        &config,
        HexEditorMessage::RemovePattern(grouped_pid),
    );

    assert!(state.groups.is_empty(), "group removed");
    assert_eq!(state.patterns.len(), 1, "standalone pattern remains");
    assert!(state.patterns[0].group_id.is_none());
}

// ---------------------------------------------------------------------------
// Pattern color_idx stored in pattern_by_addr
// ---------------------------------------------------------------------------

#[test]
fn pattern_by_addr_stores_color_idx() {
    let mut state = make_state((0..128).collect());

    seed_pattern(&mut state, 0x10, 0x1F);
    rebuild_lookups(&mut state);

    let pat = &state.patterns[0];
    for (addr, (id, color_idx)) in &state.pattern_by_addr {
        assert_eq!(*id, pat.id, "address {addr}: pattern id matches");
        assert_eq!(
            *color_idx, pat.color_idx,
            "address {addr}: color_idx matches pattern's color_idx"
        );
    }
    assert!(!state.pattern_by_addr.is_empty());
}

#[test]
fn pattern_by_addr_color_idx_survives_removal_and_add() {
    let mut state = make_state((0..128).collect());
    let config = default_config();

    // Create 3 patterns (color_idx = 0, 1, 2) via the message pipeline
    for (i, start) in [0x10, 0x30, 0x50].iter().enumerate() {
        send(&mut state, &config, HexEditorMessage::SelectAt(*start));
        send(
            &mut state,
            &config,
            HexEditorMessage::ExtendTo(start + 0x0F),
        );
        send(&mut state, &config, HexEditorMessage::CreatePattern);
        assert_eq!(
            state.patterns.last().unwrap().color_idx,
            i as u8,
            "pattern {i} should get color_idx {i}"
        );
    }

    let mid_id = state.patterns[1].id;
    send(&mut state, &config, HexEditorMessage::RemovePattern(mid_id));

    assert_eq!(state.patterns[0].color_idx, 0);
    assert_eq!(state.patterns[1].color_idx, 2);
    for (addr, (id, color_idx)) in &state.pattern_by_addr {
        let pat = state.pattern_by_id(*id).unwrap();
        assert_eq!(
            *color_idx, pat.color_idx,
            "address {addr}: color_idx {color_idx} matches pattern {id}"
        );
    }
}

// ---------------------------------------------------------------------------
// Regression: color cycling rebuilds pattern_by_addr
// ---------------------------------------------------------------------------

#[test]
fn cycle_pattern_color_rebuilds_lookup() {
    let mut state = make_state((0..64).collect());
    let config = default_config();

    seed_pattern(&mut state, 0x10, 0x1F);
    rebuild_lookups(&mut state);
    let pid = state.patterns[0].id;
    let original = state.patterns[0].color_idx;

    send(
        &mut state,
        &config,
        HexEditorMessage::CyclePatternColor(pid),
    );

    let new_color = state.patterns[0].color_idx;
    assert_ne!(new_color, original, "color should cycle");
    for (addr, (_id, ci)) in &state.pattern_by_addr {
        assert_eq!(
            *ci, new_color,
            "pattern_by_addr at {addr} should reflect new color_idx {new_color}"
        );
    }
}

#[test]
fn cycle_group_color_rebuilds_lookup_for_all_children() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_group(&mut state, "Group", 3, 0x10, 0x10);
    rebuild_lookups(&mut state);

    let gid = state.groups[0].id;
    let original = state.groups[0].color_idx;

    send(&mut state, &config, HexEditorMessage::CycleGroupColor(gid));

    let new_color = state.groups[0].color_idx;
    assert_ne!(new_color, original, "group color should cycle");
    for (addr, (_id, ci)) in &state.pattern_by_addr {
        assert_eq!(
            *ci, new_color,
            "all patterns should have the new group color at {addr}"
        );
    }
}

// ---------------------------------------------------------------------------
// Regression: rename group recomputes row_annotations
// ---------------------------------------------------------------------------

#[test]
fn commit_rename_group_recomputes_row_annotations() {
    let mut state = make_state((0..64).collect());
    let config = default_config();

    // Build a single pattern + group manually (not via seed_group or pipeline)
    send(&mut state, &config, HexEditorMessage::SelectAt(0x10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(0x1F));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    let group = crate::domain::pattern::RepeatedPatternGroup::new(
        state.next_group_id,
        "Monster".to_string(),
        0,
    );
    let gid = group.id;
    state.next_group_id += 1;
    state.groups.push(group);
    state.patterns[0].group_id = Some(gid);
    state.patterns[0].annotation = Some("Monster[0]".to_string());
    state.rebuild_pattern_lookup();
    state.recompute_row_annotations();

    // Verify old annotation is in row_annotations
    let has_annotation = state
        .row_annotations
        .values()
        .any(|segments| segments.iter().any(|(_, ann)| ann == "Monster[0]"));
    assert!(has_annotation, "row_annotations should contain old annotation");

    // Now rename the group
    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("Enemy".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    let has_new = state
        .row_annotations
        .values()
        .any(|segments| segments.iter().any(|(_, ann)| ann == "Enemy[0]"));
    assert!(has_new, "row_annotations should contain updated annotation after rename");
    let has_old = state
        .row_annotations
        .values()
        .any(|segments| segments.iter().any(|(_, ann)| ann == "Monster[0]"));
    assert!(!has_old, "row_annotations should not contain stale old annotation");
}

// ---------------------------------------------------------------------------
// Regression: collapsed_groups cleaned up after orphan removal
// ---------------------------------------------------------------------------

#[test]
fn remove_pattern_cleans_up_collapsed_groups() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    seed_group(&mut state, "Group", 1, 0x10, 0x10);
    rebuild_lookups(&mut state);

    let gid = state.groups[0].id;
    send(
        &mut state,
        &config,
        HexEditorMessage::TogglePatternGroup(gid),
    );
    assert!(state.collapsed_groups.contains(&gid), "group should be collapsed");

    let pid = state.patterns[0].id;
    send(&mut state, &config, HexEditorMessage::RemovePattern(pid));

    assert!(state.groups.is_empty(), "orphan group removed");
    assert!(!state.collapsed_groups.contains(&gid), "collapsed_groups should not contain stale orphaned group id");
}

// ---------------------------------------------------------------------------
// Regression: clear_patterns cleans up active_patterns
// ---------------------------------------------------------------------------

#[test]
fn clear_patterns_cleans_up_active_patterns() {
    let mut state = make_state((0..64).collect());
    let config = default_config();

    seed_pattern(&mut state, 0x10, 0x1F);
    rebuild_lookups(&mut state);

    state.active_patterns.insert(999);
    assert!(!state.active_patterns.is_empty(), "precondition: active_patterns populated");

    send(&mut state, &config, HexEditorMessage::ClearAllPatterns);

    assert!(state.active_patterns.is_empty(), "active_patterns should be cleared");
}
