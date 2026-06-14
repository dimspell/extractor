//! Pattern list layout — sorts all patterns by start address and assigns
//! Git-log-style branch-connector glyphs for contiguous group runs.
//!
//! # Visual metaphor
//!
//! Patterns in the same group that appear as a contiguous run in address-sorted
//! order get branch connectors:
//!
//! ```text
//! Monster ────────────────────────────────────────────────────►
//! ├─ 0x00000010─0x0000002F  32 B  [header]
//! ├─ 0x00000030─0x0000004F  32 B  [body]
//! └─ 0x00000050─0x0000006F  32 B  [footer]
//!
//! ●─ 0x00000080─0x0000008F  16 B  [checksum]
//! ```
//!
//! If a group's patterns are interleaved by patterns from other groups, they
//! form separate runs — each run gets its own branch connector sequence
//! (├ … └).  Full multi-lane (git-graph) rendering is not yet implemented.

use std::collections::BTreeSet;

use crate::pattern::{Pattern, RepeatedPatternGroup};

/// Glyph shown in the left gutter of a pattern row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GutterGlyph {
    /// `├` — first in a contiguous group run (more patterns follow in same run).
    GroupFirst,
    /// `│` — continuation of a contiguous group run.
    GroupMiddle,
    /// `└` — last in a contiguous group run.
    GroupLast,
    /// `●` — ungrouped pattern, or a solo group pattern that has no adjacent
    /// siblings in the address-sorted order (interleaved by other groups).
    Solo,
}

/// A pattern row ready for rendering, with its visual metadata.
///
/// For collapsed groups a single *stub* row is emitted (``collapsed: true``) so
/// the view can still render the group header with its ▶ toggle — otherwise the
/// user would lose the ability to re-expand the group.
#[derive(Debug, Clone)]
pub struct PatternRow {
    pub pattern_id: usize,
    pub glyph: GutterGlyph,
    /// If this is the first pattern in a group run or a collapsed-group stub,
    /// carries the group label so the view can render a section header.
    /// `None` for subsequent patterns in the same run and for solo patterns.
    pub group_label: Option<String>,
    pub group_id: Option<usize>,
    pub color_idx: u8,
    /// `true` for a collapsed-group stub — the view should render only the
    /// group header (with ▶) and skip the full pattern row.
    pub collapsed: bool,
}

/// Internal sort entry — either a visible pattern or a collapsed-group stub
/// that is inserted at the correct address-sorted position.
struct Entry<'a> {
    start: u64,
    id: usize,
    kind: EntryKind<'a>,
}

enum EntryKind<'a> {
    Visible(&'a Pattern),
    CollapsedStub {
        first: &'a Pattern,
        label: String,
        color_idx: u8,
    },
}

/// Compute sorted visible rows for the pattern list panel.
///
/// 1. Visible patterns (not in collapsed groups) are sorted by `(start, id)`.
/// 2. Each collapsed group produces a single *stub* entry sorted at the
///    position of its first pattern.
/// 3. Contiguous runs of visible patterns sharing the same group get branch
///    connectors (`├`, `│`, `└`).
/// 4. Ungrouped or isolated patterns get the `●` solo glyph.
pub fn compute_pattern_rows(
    patterns: &[Pattern],
    groups: &[RepeatedPatternGroup],
    collapsed_groups: &BTreeSet<usize>,
) -> Vec<PatternRow> {
    // ── Build entries ────────────────────────────────────────────────────

    let mut entries: Vec<Entry> = Vec::with_capacity(patterns.len() + groups.len());

    // Visible patterns
    for p in patterns {
        let hidden = p.group_id.is_some_and(|gid| collapsed_groups.contains(&gid));
        if !hidden {
            entries.push(Entry {
                start: p.start,
                id: p.id,
                kind: EntryKind::Visible(p),
            });
        }
    }

    // Collapsed-group stubs (one per collapsed group, sorted at first pattern)
    for group in groups {
        if collapsed_groups.contains(&group.id) {
            let first = patterns
                .iter()
                .filter(|p| p.group_id == Some(group.id))
                .min_by_key(|p| (p.start, p.id));
            if let Some(first) = first {
                entries.push(Entry {
                    start: first.start,
                    id: first.id,
                    kind: EntryKind::CollapsedStub {
                        first,
                        label: group.label.clone(),
                        color_idx: group.color_idx,
                    },
                });
            }
        }
    }

    // ── Sort all entries by (start, id) ──────────────────────────────────
    entries.sort_by(|a, b| a.start.cmp(&b.start).then(a.id.cmp(&b.id)));

    // ── Walk entries and emit PatternRows ────────────────────────────────
    let mut rows = Vec::with_capacity(entries.len());
    let mut i = 0;

    while i < entries.len() {
        match &entries[i].kind {
            EntryKind::CollapsedStub { first, label, color_idx } => {
                rows.push(PatternRow {
                    pattern_id: first.id,
                    glyph: GutterGlyph::GroupFirst,
                    group_label: Some(label.clone()),
                    group_id: first.group_id,
                    color_idx: *color_idx,
                    collapsed: true,
                });
                i += 1;
            }
            EntryKind::Visible(pat) => {
                match pat.group_id {
                    Some(gid) => {
                        // Contiguous run of visible patterns in the same group
                        let mut j = i + 1;
                        while j < entries.len() {
                            match &entries[j].kind {
                                EntryKind::Visible(p) if p.group_id == Some(gid) => j += 1,
                                _ => break,
                            }
                        }
                        let run_len = j - i;

                        let group_label = groups
                            .iter()
                            .find(|g| g.id == gid)
                            .map(|g| g.label.clone());

                        for k in 0..run_len {
                            let EntryKind::Visible(p) = &entries[i + k].kind else {
                                unreachable!()
                            };
                            let glyph = match run_len {
                                1 => GutterGlyph::GroupFirst,
                                _ if k == 0 => GutterGlyph::GroupFirst,
                                _ if k == run_len - 1 => GutterGlyph::GroupLast,
                                _ => GutterGlyph::GroupMiddle,
                            };
                            rows.push(PatternRow {
                                pattern_id: p.id,
                                glyph,
                                group_label: if k == 0 {
                                    group_label.clone()
                                } else {
                                    None
                                },
                                group_id: Some(gid),
                                color_idx: p.color_idx,
                                collapsed: false,
                            });
                        }
                        i = j;
                    }
                    None => {
                        rows.push(PatternRow {
                            pattern_id: pat.id,
                            glyph: GutterGlyph::Solo,
                            group_label: None,
                            group_id: None,
                            color_idx: pat.color_idx,
                            collapsed: false,
                        });
                        i += 1;
                    }
                }
            }
        }
    }

    rows
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Sorting
    // ------------------------------------------------------------------

    #[test]
    fn empty_patterns_returns_empty() {
        let rows = compute_pattern_rows(&[], &[], &BTreeSet::new());
        assert!(rows.is_empty());
    }

    #[test]
    fn ungrouped_patterns_sort_by_start_then_id() {
        let patterns = vec![
            Pattern::new(10, 0x30, 0x3F, 0),
            Pattern::new(20, 0x10, 0x1F, 1),
            Pattern::new(30, 0x20, 0x2F, 2),
        ];
        let rows = compute_pattern_rows(&patterns, &[], &BTreeSet::new());
        assert_eq!(rows.len(), 3);
        // Sorted by (start, id)
        assert_eq!(rows[0].pattern_id, 20); // 0x10
        assert_eq!(rows[1].pattern_id, 30); // 0x20
        assert_eq!(rows[2].pattern_id, 10); // 0x30
        assert!(rows.iter().all(|r| r.glyph == GutterGlyph::Solo));
    }

    #[test]
    fn same_address_tiebreaks_by_id() {
        let patterns = vec![
            Pattern::new(5, 0x10, 0x1F, 0),
            Pattern::new(3, 0x10, 0x1F, 1), // same start, lower id
            Pattern::new(7, 0x10, 0x1F, 2), // same start, higher id
        ];
        let rows = compute_pattern_rows(&patterns, &[], &BTreeSet::new());
        assert_eq!(rows.len(), 3);
        // Sorted by (start, id) → id order: 3, 5, 7
        assert_eq!(rows[0].pattern_id, 3);
        assert_eq!(rows[1].pattern_id, 5);
        assert_eq!(rows[2].pattern_id, 7);
    }

    // ------------------------------------------------------------------
    // Branch connectors
    // ------------------------------------------------------------------

    #[test]
    fn contiguous_group_run_gets_connectors() {
        let group = RepeatedPatternGroup::new(0, "Test".into(), 0);
        let patterns = vec![
            Pattern::grouped(0, 0x10, 0x1F, 0, 0),
            Pattern::grouped(1, 0x20, 0x2F, 1, 0),
            Pattern::grouped(2, 0x30, 0x3F, 2, 0),
        ];
        let rows = compute_pattern_rows(&patterns, &[group], &BTreeSet::new());
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].glyph, GutterGlyph::GroupFirst);
        assert_eq!(rows[1].glyph, GutterGlyph::GroupMiddle);
        assert_eq!(rows[2].glyph, GutterGlyph::GroupLast);

        // Only the first pattern carries the group label
        assert!(rows[0].group_label.is_some());
        assert!(rows[1].group_label.is_none());
        assert!(rows[2].group_label.is_none());
    }

    #[test]
    fn single_pattern_in_group_gets_first_glyph() {
        let group = RepeatedPatternGroup::new(0, "Lone".into(), 0);
        let patterns = vec![Pattern::grouped(0, 0x10, 0x1F, 0, 0)];
        let rows = compute_pattern_rows(&patterns, &[group], &BTreeSet::new());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].glyph, GutterGlyph::GroupFirst);
        assert!(rows[0].group_label.is_some());
    }

    #[test]
    fn solo_ungrouped_patterns_get_solo_glyph() {
        let group = RepeatedPatternGroup::new(0, "G".into(), 0);
        let patterns = vec![
            Pattern::grouped(0, 0x10, 0x1F, 0, 0), // group, run of 1
            Pattern::new(1, 0x20, 0x2F, 1),         // solo
            Pattern::grouped(2, 0x30, 0x3F, 2, 1),  // another group, run of 1
        ];
        // Two groups, each with 1 pattern → each gets GroupFirst
        // Middle one is solo
        let rows = compute_pattern_rows(
            &patterns,
            &[group, RepeatedPatternGroup::new(1, "H".into(), 1)],
            &BTreeSet::new(),
        );
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].glyph, GutterGlyph::GroupFirst); // group 0 pat
        assert_eq!(rows[0].group_id, Some(0));
        assert_eq!(rows[1].glyph, GutterGlyph::Solo); // ungrouped
        assert_eq!(rows[2].glyph, GutterGlyph::GroupFirst); // group 1 pat
        assert_eq!(rows[2].group_id, Some(1));
    }

    #[test]
    fn interleaved_groups_become_separate_runs() {
        let group_a = RepeatedPatternGroup::new(0, "A".into(), 0);
        let group_b = RepeatedPatternGroup::new(1, "B".into(), 1);
        let patterns = vec![
            Pattern::grouped(0, 0x10, 0x1F, 0, 0), // A[0]
            Pattern::grouped(1, 0x20, 0x2F, 1, 1), // B[0] — interleaves
            Pattern::grouped(2, 0x30, 0x3F, 2, 0), // A[1]
        ];
        // A[0] forms run of 1 (B[0] breaks continuity)
        // B[0] forms run of 1
        // A[1] forms run of 1
        let rows = compute_pattern_rows(
            &patterns,
            &[group_a, group_b],
            &BTreeSet::new(),
        );
        assert_eq!(rows.len(), 3);
        // Each run has length 1 → GroupFirst for all
        assert_eq!(rows[0].glyph, GutterGlyph::GroupFirst);
        assert_eq!(rows[0].group_id, Some(0));
        assert_eq!(rows[1].glyph, GutterGlyph::GroupFirst);
        assert_eq!(rows[1].group_id, Some(1));
        assert_eq!(rows[2].glyph, GutterGlyph::GroupFirst);
        assert_eq!(rows[2].group_id, Some(0));
    }

    // ------------------------------------------------------------------
    // Collapsed groups — must still produce a stub row so the toggle stays
    // visible in the UI.  The stub appears at the first pattern's sort
    // position with `collapsed: true`.
    // ------------------------------------------------------------------

    #[test]
    fn collapsed_group_produces_stub_row() {
        let group = RepeatedPatternGroup::new(0, "Collapsed".into(), 0);
        let patterns = vec![
            Pattern::grouped(0, 0x10, 0x1F, 0, 0), // group 0
            Pattern::new(1, 0x30, 0x3F, 1),          // solo
        ];
        let mut collapsed = BTreeSet::new();
        collapsed.insert(0);
        let rows = compute_pattern_rows(&patterns, &[group], &collapsed);
        // Expect two rows: collapsed-group stub + solo pattern
        assert_eq!(rows.len(), 2);
        // Stub — sorted at 0x10 (first pattern's start)
        assert_eq!(rows[0].group_id, Some(0));
        assert!(rows[0].group_label.is_some());
        assert!(rows[0].collapsed, "stub should be marked collapsed");
        // Solo — stays at 0x30
        assert_eq!(rows[1].pattern_id, 1);
        assert_eq!(rows[1].glyph, GutterGlyph::Solo);
        assert!(!rows[1].collapsed);
    }

    #[test]
    fn collapsed_group_stub_uses_group_color() {
        let group = RepeatedPatternGroup::new(0, "G".into(), 7);
        let patterns = vec![Pattern::grouped(0, 0x10, 0x1F, 0, 0)];
        let mut collapsed = BTreeSet::new();
        collapsed.insert(0);
        let rows = compute_pattern_rows(&patterns, &[group], &collapsed);
        assert_eq!(rows.len(), 1);
        // Stub should use the GROUP's color_idx, not the pattern's
        assert_eq!(rows[0].color_idx, 7);
    }

    #[test]
    fn collapsed_group_stubs_interleave_with_visible_patterns() {
        let g0 = RepeatedPatternGroup::new(0, "G0".into(), 0);
        let g1 = RepeatedPatternGroup::new(1, "G1".into(), 1);
        let patterns = vec![
            Pattern::grouped(0, 0x10, 0x1F, 0, 0), // G0
            Pattern::grouped(1, 0x20, 0x2F, 1, 1), // G1
            Pattern::new(2, 0x80, 0x8F, 2),          // solo
        ];
        let mut collapsed = BTreeSet::new();
        collapsed.insert(0); // G0 collapsed, G1 visible
        let rows = compute_pattern_rows(&patterns, &[g0, g1], &collapsed);
        // Order: G0 stub (0x10), G1[0] (0x20), solo (0x80)
        assert_eq!(rows.len(), 3);
        assert!(rows[0].collapsed);
        assert_eq!(rows[0].group_id, Some(0));
        assert!(!rows[1].collapsed);
        assert_eq!(rows[1].group_id, Some(1));
        assert!(!rows[2].collapsed);
        assert_eq!(rows[2].group_id, None);
    }

    // ------------------------------------------------------------------
    // Group labels
    // ------------------------------------------------------------------

    #[test]
    fn group_label_comes_from_groups_list() {
        let group = RepeatedPatternGroup::new(42, "MyGroup".into(), 3);
        let patterns = vec![Pattern::grouped(0, 0x10, 0x1F, 0, 42)];
        let rows = compute_pattern_rows(&patterns, &[group], &BTreeSet::new());
        assert_eq!(rows[0].group_label.as_deref(), Some("MyGroup"));
    }

    #[test]
    fn missing_group_id_in_groups_list_label_is_none() {
        // Pattern references group 99, but no group with that id exists
        let patterns = vec![Pattern::grouped(0, 0x10, 0x1F, 0, 99)];
        let rows = compute_pattern_rows(&patterns, &[], &BTreeSet::new());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].group_id, Some(99));
        assert!(rows[0].group_label.is_none(), "no matching group → no label");
        assert_eq!(rows[0].glyph, GutterGlyph::GroupFirst);
    }

    // ------------------------------------------------------------------
    // Color index passthrough
    // ------------------------------------------------------------------

    #[test]
    fn color_idx_is_preserved() {
        let patterns = vec![
            Pattern::new(0, 0x10, 0x1F, 7),
            Pattern::grouped(1, 0x20, 0x2F, 3, 0),
        ];
        let group = RepeatedPatternGroup::new(0, "G".into(), 0);
        let rows = compute_pattern_rows(&patterns, &[group], &BTreeSet::new());
        assert_eq!(rows[0].color_idx, 7);
        assert_eq!(rows[1].color_idx, 3);
    }
}
