use super::*;
use crate::RepeatPatternDialog;

// ============================================================================
// Repeated pattern groups — creation, annotation, rename, collapse, remove
// ============================================================================

// ---------------------------------------------------------------------------
// CommitRepeatedPattern — annotation prefill
// ---------------------------------------------------------------------------

#[test]
fn commit_repeated_pattern_prefills_annotations() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    // Open repeat dialog
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

    // Each pattern should have an auto-prefilled annotation
    assert_eq!(
        state.patterns[0].annotation.as_deref(),
        Some("Monster[0]")
    );
    assert_eq!(
        state.patterns[1].annotation.as_deref(),
        Some("Monster[1]")
    );
    assert_eq!(
        state.patterns[2].annotation.as_deref(),
        Some("Monster[2]")
    );

    // Each should be in the group
    let gid = state.groups[0].id;
    assert!(state.patterns.iter().all(|p| p.group_id == Some(gid)));
}

#[test]
fn commit_repeated_pattern_empty_label_uses_default() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    state.repeat_pattern = Some(RepeatPatternDialog {
        block_start: 0x00,
        block_size: 0x10,
        draft: "2".to_string(),
        label_draft: "".to_string(),
        error: None,
    });
    send(&mut state, &config, HexEditorMessage::CommitRepeatedPattern);

    assert_eq!(state.groups.len(), 1);
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
fn commit_repeated_pattern_whitespace_label_uses_default() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    state.repeat_pattern = Some(RepeatPatternDialog {
        block_start: 0x00,
        block_size: 0x10,
        draft: "1".to_string(),
        label_draft: "   ".to_string(),
        error: None,
    });
    send(&mut state, &config, HexEditorMessage::CommitRepeatedPattern);

    assert_eq!(state.groups[0].label, "Unnamed group");
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

    // 5 blocks of 0x20 → blocks 0-31 and 32-49 fit; block 3 at 64 > max_addr
    assert_eq!(state.patterns.len(), 2, "only 2 blocks fit in 50 bytes");
    assert_eq!(state.patterns[0].end, 31);
    assert_eq!(state.patterns[1].end, 49, "last block should clamp at max_addr");
}

#[test]
fn commit_repeated_pattern_invalid_count_shows_error() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    state.repeat_pattern = Some(RepeatPatternDialog {
        block_start: 0x00,
        block_size: 0x10,
        draft: "not_a_number".to_string(),
        label_draft: "Monster".to_string(),
        error: None,
    });
    send(&mut state, &config, HexEditorMessage::CommitRepeatedPattern);

    // Should not create patterns, dialog should have an error
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
    create_single_pattern(&mut state, &config, 10, 20);
    let id = state.patterns[0].id;

    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(id, "my note".to_string()),
    );

    assert_eq!(
        state.patterns[0].annotation.as_deref(),
        Some("my note")
    );
}

#[test]
fn set_annotation_empty_clears_annotation() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    create_single_pattern(&mut state, &config, 10, 20);
    let id = state.patterns[0].id;

    // Set annotation
    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(id, "note".to_string()),
    );
    assert!(state.patterns[0].annotation.is_some());

    // Clear via empty string
    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(id, "".to_string()),
    );
    assert!(
        state.patterns[0].annotation.is_none(),
        "annotation should be cleared"
    );
}

#[test]
fn set_annotation_whitespace_clears_annotation() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    create_single_pattern(&mut state, &config, 10, 20);
    let id = state.patterns[0].id;

    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(id, "   ".to_string()),
    );
    assert!(
        state.patterns[0].annotation.is_none(),
        "whitespace-only annotation should be cleared"
    );
}

#[test]
fn set_annotation_recomputes_row_annotations() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    create_single_pattern(&mut state, &config, 10, 20);
    let id = state.patterns[0].id;

    // Before: no annotations
    assert!(
        state.row_annotations.is_empty(),
        "row_annotations should be empty before setting annotation"
    );

    // Set annotation
    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(id, "tag".to_string()),
    );
    assert!(
        !state.row_annotations.is_empty(),
        "row_annotations should be recomputed after setting annotation"
    );
    // Pattern covers bytes 10-20, bpr=16 → rows at row_start 0 and 16
    assert!(
        state.row_annotations.contains_key(&0),
        "row_annotations should contain entry for first row (byte 0..15)"
    );
    assert!(
        state.row_annotations.contains_key(&16),
        "row_annotations should contain entry for second row (byte 16..31)"
    );

    // Clear annotation
    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(id, "".to_string()),
    );
    assert!(
        state.row_annotations.is_empty(),
        "row_annotations should be empty after clearing annotation"
    );
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
    // No panic, no change
    assert!(state.patterns.is_empty());
}

// ---------------------------------------------------------------------------
// CommitRenameGroup — annotation auto-update
// ---------------------------------------------------------------------------

fn create_group(
    state: &mut HexEditorState,
    config: &HexEditorConfig,
    label: &str,
    base: u64,
    size: u64,
    count: u32,
) {
    state.repeat_pattern = Some(RepeatPatternDialog {
        block_start: base,
        block_size: size,
        draft: count.to_string(),
        label_draft: label.to_string(),
        error: None,
    });
    send(state, config, HexEditorMessage::CommitRepeatedPattern);
}

fn create_single_pattern(
    state: &mut HexEditorState,
    config: &HexEditorConfig,
    start: u64,
    end: u64,
) {
    send(state, config, HexEditorMessage::SelectAt(start));
    send(state, config, HexEditorMessage::ExtendTo(end));
    send(state, config, HexEditorMessage::CreatePattern);
}

#[test]
fn rename_group_updates_child_annotations() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x20, 3);

    let gid = state.groups[0].id;

    // Begin rename
    send(
        &mut state,
        &config,
        HexEditorMessage::BeginRenameGroup(gid),
    );
    assert_eq!(state.renaming_group, Some(gid));
    assert_eq!(state.renaming_group_draft, "Monster");

    // Edit draft
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("Enemy".to_string()),
    );

    // Commit
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    assert_eq!(state.groups[0].label, "Enemy");
    assert!(state.renaming_group.is_none(), "rename should be cleared");
    assert!(state.renaming_group_draft.is_empty(), "draft should be cleared");

    // Annotations should be updated
    assert_eq!(state.patterns[0].annotation.as_deref(), Some("Enemy[0]"));
    assert_eq!(state.patterns[1].annotation.as_deref(), Some("Enemy[1]"));
    assert_eq!(state.patterns[2].annotation.as_deref(), Some("Enemy[2]"));
    assert!(
        state.status_msg.contains("Enemy"),
        "status should mention new name"
    );
}

#[test]
fn rename_group_to_empty_uses_unnamed_and_updates_annotations() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 2);

    let gid = state.groups[0].id;
    send(
        &mut state,
        &config,
        HexEditorMessage::BeginRenameGroup(gid),
    );
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
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 2);

    // Manually change the second pattern's annotation
    let pid2 = state.patterns[1].id;
    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(pid2, "custom note".to_string()),
    );

    let gid = state.groups[0].id;
    send(
        &mut state,
        &config,
        HexEditorMessage::BeginRenameGroup(gid),
    );
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("Enemy".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    // Auto-generated annotation should update
    assert_eq!(state.patterns[0].annotation.as_deref(), Some("Enemy[0]"));
    // Manual annotation should be preserved
    assert_eq!(
        state.patterns[1].annotation.as_deref(),
        Some("custom note")
    );
}

#[test]
fn rename_group_substring_old_to_new() {
    // "Mon" → "Monster": prefix replacement should work
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Mon", 0x10, 0x10, 2);

    let gid = state.groups[0].id;
    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("Monster".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    assert_eq!(state.groups[0].label, "Monster");
    assert_eq!(state.patterns[0].annotation.as_deref(), Some("Monster[0]"));
    assert_eq!(state.patterns[1].annotation.as_deref(), Some("Monster[1]"));
}

#[test]
fn rename_group_substring_new_to_old() {
    // "Monster" → "Mon": prefix replacement
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 2);

    let gid = state.groups[0].id;
    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("Mon".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    assert_eq!(state.groups[0].label, "Mon");
    assert_eq!(state.patterns[0].annotation.as_deref(), Some("Mon[0]"));
    assert_eq!(state.patterns[1].annotation.as_deref(), Some("Mon[1]"));
}

#[test]
fn rename_group_other_groups_unaffected() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    // Create two groups
    create_group(&mut state, &config, "GroupA", 0x10, 0x10, 2);
    let gid_a = state.groups[0].id;
    create_group(&mut state, &config, "GroupB", 0x40, 0x10, 2);
    let gid_b = state.groups[1].id;

    // Rename GroupA → RenamedA
    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid_a));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("RenamedA".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    // GroupB labels and annotations should be unchanged
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
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    // Create a group and an ungrouped pattern
    create_group(&mut state, &config, "GroupA", 0x10, 0x10, 2);
    let gid = state.groups[0].id;
    create_single_pattern(&mut state, &config, 0x80, 0x8F);
    let ungrouped_id = state.patterns[2].id;

    // Set annotation on ungrouped pattern
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

    // Ungrouped pattern annotation should be untouched
    assert_eq!(
        state.patterns[2].annotation.as_deref(),
        Some("standalone")
    );
}

#[test]
fn rename_group_non_auto_annotation_preserved() {
    // An annotation that starts with "Monster[" but doesn't follow the
    // `{label}[{digits}]` pattern should NOT be updated.
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 1);
    let gid = state.groups[0].id;

    // Manually set annotation to something that starts with "Monster["
    // but isn't auto-generated (no digits after [)
    let pid = state.patterns[0].id;
    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(
            pid,
            "Monster[foobar]".to_string(),
        ),
    );

    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("Enemy".to_string()),
    );
    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    // Annotation should remain unchanged (not digits between brackets)
    assert_eq!(
        state.patterns[0].annotation.as_deref(),
        Some("Monster[foobar]")
    );
}

#[test]
fn rename_group_non_match_annotation_preserved() {
    // Annotation that starts differently from old_label[ should not be touched
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 1);
    let gid = state.groups[0].id;

    // Manually set annotation to something completely different
    let pid = state.patterns[0].id;
    send(
        &mut state,
        &config,
        HexEditorMessage::SetPatternAnnotation(
            pid,
            "custom annotation".to_string(),
        ),
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
        Some("custom annotation")
    );
}

#[test]
fn rename_group_cancel_does_not_change_anything() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 2);

    let gid = state.groups[0].id;

    send(&mut state, &config, HexEditorMessage::BeginRenameGroup(gid));
    send(
        &mut state,
        &config,
        HexEditorMessage::SetRenameGroupDraft("Enemy".to_string()),
    );
    // Cancel instead of commit
    send(&mut state, &config, HexEditorMessage::CancelRenameGroup);

    assert_eq!(state.groups[0].label, "Monster", "label should be unchanged");
    assert!(state.renaming_group.is_none());
    assert!(state.renaming_group_draft.is_empty());

    // Annotations unchanged
    assert_eq!(
        state.patterns[0].annotation.as_deref(),
        Some("Monster[0]")
    );
    assert_eq!(
        state.patterns[1].annotation.as_deref(),
        Some("Monster[1]")
    );
}

#[test]
fn rename_group_whitespace_label_trims_and_uses_default() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 2);

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
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    // Set renaming state to a gid that doesn't exist
    state.renaming_group = Some(999);
    state.renaming_group_draft = "Ghost".to_string();

    send(&mut state, &config, HexEditorMessage::CommitRenameGroup);

    // Should not panic, renames nothing
    assert!(state.groups.is_empty());
    assert!(state.patterns.is_empty());
}

// ---------------------------------------------------------------------------
// TogglePatternGroup — collapse/expand
// ---------------------------------------------------------------------------

#[test]
fn toggle_group_collapse_expand() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 2);

    let gid = state.groups[0].id;

    // Initially not collapsed
    assert!(!state.collapsed_groups.contains(&gid));

    // Toggle to collapse
    send(
        &mut state,
        &config,
        HexEditorMessage::TogglePatternGroup(gid),
    );
    assert!(state.collapsed_groups.contains(&gid));

    // Toggle to expand
    send(
        &mut state,
        &config,
        HexEditorMessage::TogglePatternGroup(gid),
    );
    assert!(!state.collapsed_groups.contains(&gid));
}

#[test]
fn toggle_nonexistent_group_does_not_panic() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    // Should not panic — toggling inserts to collapsed set even for
    // nonexistent groups (harmless; just a toggle without side effects)
    send(
        &mut state,
        &config,
        HexEditorMessage::TogglePatternGroup(999),
    );
    // The id was inserted (toggle behavior), but no patterns exist
    assert!(state.patterns.is_empty());
}

// ---------------------------------------------------------------------------
// RemovePatternGroup — removes group and all children
// ---------------------------------------------------------------------------

#[test]
fn remove_group_removes_all_child_patterns() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 3);
    // Also add an ungrouped pattern that should survive
    create_single_pattern(&mut state, &config, 0x80, 0x8F);

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
    assert!(
        !state.collapsed_groups.contains(&gid),
        "collapsed state should be cleaned up"
    );
    assert!(
        state.status_msg.contains("Removed group and 3 pattern(s)"),
        "should report correct removed count: got {}",
        state.status_msg
    );
}

#[test]
fn remove_group_recomputes_lookups() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 2);

    let gid = state.groups[0].id;
    // pattern_by_addr should be populated
    assert!(!state.pattern_by_addr.is_empty());
    assert!(!state.row_annotations.is_empty());

    send(
        &mut state,
        &config,
        HexEditorMessage::RemovePatternGroup(gid),
    );

    assert!(
        state.pattern_by_addr.is_empty(),
        "pattern lookup should be rebuilt"
    );
    assert!(
        state.row_annotations.is_empty(),
        "row annotations should be recomputed"
    );
}

#[test]
fn remove_nonexistent_group_is_noop() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    // Should not panic
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
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 3);

    let gid = state.groups[0].id;
    let original_color = state.groups[0].color_idx;

    send(
        &mut state,
        &config,
        HexEditorMessage::CycleGroupColor(gid),
    );

    let expected_color = (original_color + 1) % 16;
    assert_eq!(state.groups[0].color_idx, expected_color);

    // All children should have the new color
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
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    // Should not panic
    send(
        &mut state,
        &config,
        HexEditorMessage::CycleGroupColor(999),
    );
}

// ---------------------------------------------------------------------------
// CyclePatternColor — individual pattern only
// ---------------------------------------------------------------------------

#[test]
fn cycle_pattern_color() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    create_single_pattern(&mut state, &config, 10, 20);
    let id = state.patterns[0].id;
    let original_color = state.patterns[0].color_idx;

    send(
        &mut state,
        &config,
        HexEditorMessage::CyclePatternColor(id),
    );

    assert_eq!(
        state.patterns[0].color_idx,
        (original_color + 1) % 16
    );
}

#[test]
fn cycle_pattern_color_nonexistent_id_is_noop() {
    let mut state = make_state((0..64).collect());
    let config = default_config();

    // Should not panic
    send(
        &mut state,
        &config,
        HexEditorMessage::CyclePatternColor(999),
    );
}

// ---------------------------------------------------------------------------
// SetRenameGroupDraft — updates draft
// ---------------------------------------------------------------------------

#[test]
fn set_rename_group_draft_updates_string() {
    let mut state = make_state((0..64).collect());
    let config = default_config();
    create_group(&mut state, &config, "Group", 0x10, 0x10, 1);
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

    // repeat_pattern is None
    send(&mut state, &config, HexEditorMessage::CommitRepeatedPattern);

    // Should not panic, no patterns created
    assert!(state.patterns.is_empty());
    assert!(state.groups.is_empty());
}

#[test]
fn commit_repeated_pattern_zero_count_does_nothing() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    state.repeat_pattern = Some(RepeatPatternDialog {
        block_start: 0x00,
        block_size: 0x10,
        draft: "0".to_string(),
        label_draft: "Empty".to_string(),
        error: None,
    });
    send(&mut state, &config, HexEditorMessage::CommitRepeatedPattern);

    // count < 1 is rejected by parse_repeat_count() → no group, no patterns
    assert_eq!(state.patterns.len(), 0, "zero count should create no patterns");
    assert_eq!(state.groups.len(), 0, "group should NOT be created for invalid count");
    // Dialog still open with error
    assert!(state.repeat_pattern.is_some());
    assert!(state.repeat_pattern.unwrap().error.is_some());
}

// ---------------------------------------------------------------------------
// Orphan group auto-cleanup
// ---------------------------------------------------------------------------

#[test]
fn remove_last_pattern_in_group_removes_group() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 1);

    assert_eq!(state.groups.len(), 1, "should have 1 group");
    let pid = state.patterns[0].id;

    send(&mut state, &config, HexEditorMessage::RemovePattern(pid));

    assert!(state.groups.is_empty(), "group should be removed when its last pattern is removed");
    assert!(state.patterns.is_empty(), "pattern should be gone");
}

#[test]
fn remove_some_but_not_all_patterns_keeps_group() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 3);

    assert_eq!(state.groups.len(), 1, "should have 1 group");
    assert_eq!(state.patterns.len(), 3, "should have 3 patterns");
    let pid = state.patterns[0].id;

    send(&mut state, &config, HexEditorMessage::RemovePattern(pid));

    assert_eq!(state.groups.len(), 1, "group should persist when patterns remain");
    assert_eq!(state.patterns.len(), 2, "only one pattern removed");
    assert_eq!(state.groups[0].label, "Monster", "group label preserved");
}

#[test]
fn remove_all_patterns_from_multiple_groups_cleans_up() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    // Create two groups, each with 1 pattern
    create_group(&mut state, &config, "GroupA", 0x10, 0x10, 1);
    create_group(&mut state, &config, "GroupB", 0x30, 0x10, 1);

    assert_eq!(state.groups.len(), 2);
    let pid_a = state.patterns[0].id;
    let pid_b = state.patterns[1].id;

    send(&mut state, &config, HexEditorMessage::RemovePattern(pid_a));
    assert_eq!(state.groups.len(), 1, "GroupA removed, GroupB stays");
    assert_eq!(state.groups[0].label, "GroupB");

    send(&mut state, &config, HexEditorMessage::RemovePattern(pid_b));
    assert!(state.groups.is_empty(), "all groups removed");
}

#[test]
fn orphan_cleanup_does_not_affect_patterns_not_in_groups() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Monster", 0x10, 0x10, 1);

    // Also create a standalone pattern
    state.repeat_pattern = None;
    send(&mut state, &config, HexEditorMessage::SelectAt(0x50));
    send(&mut state, &config, HexEditorMessage::ExtendTo(0x5F));
    send(&mut state, &config, HexEditorMessage::CreatePattern);

    assert_eq!(state.patterns.len(), 2, "grouped + standalone pattern");
    let gid = state.groups[0].id;
    let grouped_pid = state.patterns.iter().find(|p| p.group_id == Some(gid)).unwrap().id;

    send(&mut state, &config, HexEditorMessage::RemovePattern(grouped_pid));

    assert!(state.groups.is_empty(), "group removed");
    assert_eq!(state.patterns.len(), 1, "standalone pattern remains");
    assert!(state.patterns[0].group_id.is_none());
}

// ---------------------------------------------------------------------------
// Pattern color_idx stored in pattern_by_addr
// ---------------------------------------------------------------------------

#[test]
fn pattern_by_addr_stores_color_idx() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    // Create a pattern — it gets color_idx = 0 (first pattern)
    send(&mut state, &config, HexEditorMessage::SelectAt(0x10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(0x1F));
    send(&mut state, &config, HexEditorMessage::CreatePattern);

    let pat = &state.patterns[0];
    for (addr, (id, color_idx)) in &state.pattern_by_addr {
        assert_eq!(*id, pat.id, "address {addr}: pattern id matches");
        assert_eq!(*color_idx, pat.color_idx, "address {addr}: color_idx matches pattern's color_idx");
    }
    assert!(!state.pattern_by_addr.is_empty());
}

#[test]
fn pattern_by_addr_color_idx_survives_removal_and_add() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    // Create 3 patterns to get color_idx = 0, 1, 2
    for (i, start) in [0x10, 0x30, 0x50].iter().enumerate() {
        send(&mut state, &config, HexEditorMessage::SelectAt(*start));
        send(&mut state, &config, HexEditorMessage::ExtendTo(start + 0x0F));
        send(&mut state, &config, HexEditorMessage::CreatePattern);
        assert_eq!(state.patterns.last().unwrap().color_idx, i as u8,
            "pattern {i} should get color_idx {i}");
    }

    // Remove pattern 1 (the middle one)
    let mid_id = state.patterns[1].id;
    send(&mut state, &config, HexEditorMessage::RemovePattern(mid_id));

    // Remaining patterns should still have their original color_idx
    assert_eq!(state.patterns[0].color_idx, 0);
    assert_eq!(state.patterns[1].color_idx, 2);
    for (addr, (id, color_idx)) in &state.pattern_by_addr {
        let pat = state.pattern_by_id(*id).unwrap();
        assert_eq!(*color_idx, pat.color_idx,
            "address {addr}: color_idx {color_idx} matches pattern {id}");
    }
}

// ---------------------------------------------------------------------------
// Regression: color cycling rebuilds pattern_by_addr
// ---------------------------------------------------------------------------

#[test]
fn cycle_pattern_color_rebuilds_lookup() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    send(&mut state, &config, HexEditorMessage::SelectAt(0x10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(0x1F));
    send(&mut state, &config, HexEditorMessage::CreatePattern);
    let pid = state.patterns[0].id;
    let original = state.patterns[0].color_idx;

    send(&mut state, &config, HexEditorMessage::CyclePatternColor(pid));

    let new_color = state.patterns[0].color_idx;
    assert_ne!(new_color, original, "color should cycle");
    for (addr, (_id, ci)) in &state.pattern_by_addr {
        assert_eq!(*ci, new_color,
            "pattern_by_addr at {addr} should reflect new color_idx {new_color}");
    }
}

#[test]
fn cycle_group_color_rebuilds_lookup_for_all_children() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Group", 0x10, 0x10, 3);

    let gid = state.groups[0].id;
    let original = state.groups[0].color_idx;

    send(&mut state, &config, HexEditorMessage::CycleGroupColor(gid));

    let new_color = state.groups[0].color_idx;
    assert_ne!(new_color, original, "group color should cycle");
    // All child patterns in pattern_by_addr should have the new color
    for (addr, (_id, ci)) in &state.pattern_by_addr {
        assert_eq!(*ci, new_color,
            "all patterns should have the new group color at {addr}");
    }
}

// ---------------------------------------------------------------------------
// Regression: rename group recomputes row_annotations
// ---------------------------------------------------------------------------

#[test]
fn commit_rename_group_recomputes_row_annotations() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    // Create a single pattern and assign it to a group manually
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

    // Verify annotation is in row_annotations
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

    // row_annotations should now have the new annotation
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
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();
    create_group(&mut state, &config, "Group", 0x10, 0x10, 1);

    let gid = state.groups[0].id;
    // Collapse the group
    send(&mut state, &config, HexEditorMessage::TogglePatternGroup(gid));
    assert!(state.collapsed_groups.contains(&gid), "group should be collapsed");

    // Remove the only pattern — group should be cleaned up
    let pid = state.patterns[0].id;
    send(&mut state, &config, HexEditorMessage::RemovePattern(pid));

    assert!(state.groups.is_empty(), "orphan group removed");
    assert!(!state.collapsed_groups.contains(&gid),
        "collapsed_groups should not contain stale orphaned group id");
}

// ---------------------------------------------------------------------------
// Regression: clear_patterns cleans up active_patterns
// ---------------------------------------------------------------------------

#[test]
fn clear_patterns_cleans_up_active_patterns() {
    let mut state = make_state((0..=255u8).collect());
    let config = default_config();

    send(&mut state, &config, HexEditorMessage::SelectAt(0x10));
    send(&mut state, &config, HexEditorMessage::ExtendTo(0x1F));
    send(&mut state, &config, HexEditorMessage::CreatePattern);

    // Manually seed active_patterns (normally set via cursor-move logic)
    state.active_patterns.insert(999);
    assert!(!state.active_patterns.is_empty(), "precondition: active_patterns populated");

    send(&mut state, &config, HexEditorMessage::ClearAllPatterns);

    assert!(state.active_patterns.is_empty(), "active_patterns should be cleared");
}
